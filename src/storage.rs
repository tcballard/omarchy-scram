use crate::game::Game;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

pub fn xdg(key: &str, fallback: &str) -> PathBuf {
    std::env::var_os(key)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(fallback)
        })
}
// Keep this storage identity through the rename; both executables share the lock.
pub const DATA_DIRECTORY: &str = "omarchy-munch";
pub fn state_dir() -> PathBuf {
    xdg("XDG_STATE_HOME", ".local/state").join(DATA_DIRECTORY)
}
pub fn read_bounded(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File exceeds size limit",
        ));
    }
    Ok(bytes)
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Missing parent directory"))?;
    fs::create_dir_all(parent)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| e.error)?;
    File::open(parent)?.sync_all()
}
pub struct SessionLock {
    _file: File,
}
impl SessionLock {
    pub fn acquire(dir: &Path) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(dir.join("session.lock"))?;
        file.try_lock_exclusive().map_err(|_| {
            io::Error::other(
                "Omarchy Scram is already using this save folder. Close the other window first.",
            )
        })?;
        Ok(Self { _file: file })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Appearance {
    Omarchy,
    Charcoal,
    Ivory,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub sound: bool,
    pub appearance: Appearance,
    pub reduce_motion: bool,
    pub relaxed: bool,
    pub character_pack: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            sound: false,
            appearance: Appearance::Omarchy,
            reduce_motion: false,
            relaxed: false,
            character_pack: "builtin:latch".into(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Save {
    pub version: u32,
    pub game: Game,
    pub settings: Settings,
    pub best: [u32; 2],
    pub last_score: Option<u32>,
}
impl Default for Save {
    fn default() -> Self {
        Self {
            version: 1,
            game: Game::default(),
            settings: Settings::default(),
            best: [0; 2],
            last_score: None,
        }
    }
}
impl Save {
    pub fn record(&mut self) {
        let i = self.game.relaxed as usize;
        self.best[i] = self.best[i].max(self.game.score);
    }
    pub fn write(&mut self, dir: &Path) -> io::Result<()> {
        self.record();
        atomic_write(&dir.join("session.json"), &serde_json::to_vec_pretty(self)?)
    }
    pub fn restart(&mut self, dir: &Path) -> io::Result<()> {
        let mut next = self.clone();
        next.record();
        atomic_write(
            &dir.join("previous-session.json"),
            &serde_json::to_vec_pretty(&next)?,
        )?;
        next.last_score = Some(next.game.score);
        next.game = Game::new(next.settings.relaxed);
        next.write(dir)?;
        *self = next;
        Ok(())
    }
}
pub struct Loaded {
    pub save: Save,
    pub notice: String,
    pub writable: bool,
}
pub fn load(dir: &Path) -> Loaded {
    let path = dir.join("session.json");
    let read = read_bounded(&path, 1_000_000);
    if matches!(&read,Err(e) if e.kind()==io::ErrorKind::NotFound) {
        return Loaded {
            save: Save::default(),
            notice: String::new(),
            writable: true,
        };
    }
    if let Ok(bytes) = &read {
        if let Ok(raw) = serde_json::from_slice::<serde_json::Value>(bytes) {
            if raw
                .get("version")
                .and_then(|v| v.as_u64())
                .is_some_and(|v| v != 1)
            {
                return Loaded { save: Save::default(), notice: "This save was made by a different version. You can play, but saving is disabled to protect it.".into(), writable: false };
            }
        }
    }
    let decoded =
        read.and_then(|bytes| serde_json::from_slice::<Save>(&bytes).map_err(io::Error::other));
    match decoded {
        Ok(save) if save.version != 1 => Loaded{save:Save::default(),notice:"This save was made by a different version. You can play, but saving is disabled to protect it.".into(),writable:false},
        Ok(save) if save.game.validate()=>Loaded{save,notice:String::new(),writable:true},
        Err(e) if e.kind()!=io::ErrorKind::InvalidData && e.kind()!=io::ErrorKind::Other =>Loaded{save:Save::default(),notice:format!("Cannot read saved game: {e}. Saving is disabled."),writable:false},
        _=>{
            let suffix=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
            let backup=dir.join(format!("session-damaged-{suffix}.json"));
            match fs::rename(&path,&backup) {
                Ok(())=>Loaded{save:Save::default(),notice:"The saved game was damaged. Its original file has been kept beside your new save.".into(),writable:true},
                Err(e)=>Loaded{save:Save::default(),notice:format!("The damaged save could not be preserved: {e}. Saving is disabled."),writable:false},
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn save_round_trip_and_archive() {
        let d = tempfile::tempdir().unwrap();
        let mut s = Save::default();
        s.game.score = 120;
        s.settings.sound = true;
        s.write(d.path()).unwrap();
        assert_eq!(load(d.path()).save, s);
        s.restart(d.path()).unwrap();
        assert_eq!(s.game.score, 0);
        assert_eq!(s.best[0], 120);
        assert!(d.path().join("previous-session.json").exists());
    }
    #[test]
    fn malformed_save_is_preserved() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("session.json"), b"broken").unwrap();
        let l = load(d.path());
        assert!(l.writable);
        assert!(!l.notice.is_empty());
        let file = fs::read_dir(d.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert_eq!(fs::read(file).unwrap(), b"broken");
    }
    #[test]
    fn newer_version_is_never_overwritten() {
        let d = tempfile::tempdir().unwrap();
        let s = Save {
            version: 2,
            ..Save::default()
        };
        let b = serde_json::to_vec(&s).unwrap();
        fs::write(d.path().join("session.json"), &b).unwrap();
        assert!(!load(d.path()).writable);
        assert_eq!(fs::read(d.path().join("session.json")).unwrap(), b);
    }
    #[test]
    fn future_schema_is_preserved_before_decoding() {
        let d = tempfile::tempdir().unwrap();
        let bytes = br#"{"version":2,"different_schema":true}"#;
        fs::write(d.path().join("session.json"), bytes).unwrap();
        assert!(!load(d.path()).writable);
        assert_eq!(fs::read(d.path().join("session.json")).unwrap(), bytes);
    }
    #[test]
    fn failed_restart_keeps_current_run() {
        let d = tempfile::tempdir().unwrap();
        let mut save = Save::default();
        save.game.score = 450;
        let original = save.clone();
        fs::create_dir(d.path().join("session.json")).unwrap();
        assert!(save.restart(d.path()).is_err());
        assert_eq!(save, original);
    }
    #[test]
    fn lock_excludes_concurrent_writers() {
        let d = tempfile::tempdir().unwrap();
        let l = SessionLock::acquire(d.path()).unwrap();
        assert!(SessionLock::acquire(d.path()).is_err());
        drop(l);
        assert!(SessionLock::acquire(d.path()).is_ok());
    }
    #[test]
    fn relaxed_scores_stay_separate() {
        let mut s = Save::default();
        s.game.score = 100;
        s.record();
        s.game = Game::new(true);
        s.game.score = 250;
        s.record();
        assert_eq!(s.best, [100, 250]);
    }
}
