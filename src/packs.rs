//! Data-only artwork packs. No game state or rules are accessible to a pack.
use crate::{game::Dir, storage, theme};
use eframe::egui::{self, Color32, Pos2, Rect, TextureHandle, Vec2};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

const MAX_FILE: u64 = 8 * 1024 * 1024;
const MAX_DIM: u32 = 2048;
pub const LATCH: &str = "builtin:latch";
pub const ORIGINAL: &str = "builtin:original";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Atlas {
    pub image: String,
    /// Pixel rectangles [left, top, width, height], one square per frame.
    pub frames: Vec<[u32; 4]>,
    #[serde(default)]
    pub key_color: Option<String>,
    #[serde(default = "tolerance")]
    pub key_tolerance: u8,
}
fn tolerance() -> u8 {
    70
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub name: String,
    pub author: String,
    pub license: String,
    pub icon: String,
    pub player: Atlas,
    pub pursuers: Atlas,
    pub normal: [usize; 4],
    pub vulnerable: [usize; 4],
    #[serde(default = "animation_rate")]
    pub fps: f32,
    #[serde(default = "sprite_scale")]
    pub scale: f32,
}
fn animation_rate() -> f32 {
    9.
}
fn sprite_scale() -> f32 {
    1.05
}
#[derive(Debug)]
pub struct Pixels {
    pub size: [usize; 2],
    pub rgba: Vec<u8>,
}
pub struct PackData {
    pub manifest: Manifest,
    pub player: Pixels,
    pub pursuers: Pixels,
    pub icon: Pixels,
}
#[derive(Clone, Copy)]
pub enum PursuerState {
    Normal,
    Vulnerable,
    Returning,
}
pub struct Artwork {
    pub name: String,
    pub author: String,
    pub license: String,
    manifest: Manifest,
    player: TextureHandle,
    pursuers: TextureHandle,
    pub icon: TextureHandle,
    pub icon_data: egui::IconData,
}
#[derive(Clone, Debug)]
pub struct Choice {
    pub key: String,
    pub name: String,
    pub error: Option<String>,
}

pub fn directory() -> PathBuf {
    storage::xdg("XDG_DATA_HOME", ".local/share")
        .join(storage::DATA_DIRECTORY)
        .join("packs")
}
fn valid_folder(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn safe_file(root: &Path, name: &str) -> Result<PathBuf, String> {
    let path = Path::new(name);
    if name.is_empty()
        || name.len() > 200
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("Asset paths must be relative, without parent-directory segments.".into());
    }
    let base = root.canonicalize().map_err(|e| e.to_string())?;
    let resolved = root
        .join(path)
        .canonicalize()
        .map_err(|e| format!("Cannot open {name}: {e}"))?;
    if !resolved.starts_with(&base) || !resolved.is_file() {
        return Err(format!("{name} must be a file inside this pack."));
    }
    Ok(resolved)
}
pub fn parse(text: &str) -> Result<Manifest, String> {
    let m: Manifest = toml::from_str(text).map_err(|e| format!("Invalid pack.toml: {e}"))?;
    if m.schema != 1 {
        return Err(format!(
            "Unsupported pack schema {} (supported: 1).",
            m.schema
        ));
    }
    for (name, value) in [
        ("name", &m.name),
        ("author", &m.author),
        ("license", &m.license),
    ] {
        if value.trim().is_empty() || value.len() > 100 || value.chars().any(char::is_control) {
            return Err(format!("{name} must contain 1–100 readable characters."));
        }
    }
    if !m.fps.is_finite() || !(1.0..=24.).contains(&m.fps) {
        return Err("fps must be between 1 and 24.".into());
    }
    if !m.scale.is_finite() || !(0.6..=1.2).contains(&m.scale) {
        return Err("scale must be between 0.6 and 1.2.".into());
    }
    if m.player.frames.is_empty() || m.player.frames.len() > 12 {
        return Err("Provide 1–12 player frames.".into());
    }
    if m.pursuers.frames.is_empty() || m.pursuers.frames.len() > 32 {
        return Err("Provide 1–32 pursuer frames.".into());
    }
    if m.normal
        .iter()
        .chain(m.vulnerable.iter())
        .any(|&i| i >= m.pursuers.frames.len())
    {
        return Err("A pursuer frame index is outside its atlas.".into());
    }
    for atlas in [&m.player, &m.pursuers] {
        if atlas
            .key_color
            .as_ref()
            .is_some_and(|s| theme::hex(s).is_none())
        {
            return Err("key_color must be a #rrggbb colour.".into());
        }
        if atlas.key_tolerance > 100 {
            return Err("key_tolerance must be 0–100.".into());
        }
    }
    Ok(m)
}
fn decode(bytes: &[u8], atlas: Option<&Atlas>) -> Result<Pixels, String> {
    if bytes.len() as u64 > MAX_FILE {
        return Err("PNG exceeds 8 MiB.".into());
    }
    let reader =
        image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png);
    let (w, h) = reader
        .into_dimensions()
        .map_err(|e| format!("Invalid PNG: {e}"))?;
    if w == 0 || h == 0 || w > MAX_DIM || h > MAX_DIM {
        return Err("PNG dimensions must be 1–2048 pixels per side.".into());
    }
    if let Some(atlas) = atlas {
        for [x, y, fw, fh] in &atlas.frames {
            if *fw == 0
                || *fw != *fh
                || x.checked_add(*fw).is_none_or(|n| n > w)
                || y.checked_add(*fh).is_none_or(|n| n > h)
            {
                return Err("Every frame must be a square rectangle contained in its PNG.".into());
            }
        }
    }
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?
        .into_rgba8();
    let mut rgba = image.into_raw();
    // Conventional colour-key interpretation at texture load. Source PNG is unchanged.
    if let Some((key, tolerance)) = atlas.and_then(|a| {
        a.key_color
            .as_ref()
            .and_then(|s| theme::hex(s))
            .map(|k| (k, a.key_tolerance))
    }) {
        for px in rgba.as_chunks_mut::<4>().0 {
            let distance = px[0]
                .abs_diff(key.r())
                .max(px[1].abs_diff(key.g()))
                .max(px[2].abs_diff(key.b()));
            if distance <= tolerance {
                px[3] = 0;
            }
        }
    }
    if let Some(atlas) = atlas {
        for [x, y, fw, fh] in &atlas.frames {
            let visible = (*y..y + fh)
                .any(|yy| (*x..x + fw).any(|xx| rgba[((yy * w + xx) * 4 + 3) as usize] > 32));
            if !visible {
                return Err("An atlas frame is completely transparent.".into());
            }
        }
    }
    Ok(Pixels {
        size: [w as usize, h as usize],
        rgba,
    })
}
fn load_with(
    m: Manifest,
    mut get: impl FnMut(&str) -> Result<Vec<u8>, String>,
) -> Result<PackData, String> {
    let player = decode(&get(&m.player.image)?, Some(&m.player))?;
    let pursuers = decode(&get(&m.pursuers.image)?, Some(&m.pursuers))?;
    let icon = decode(&get(&m.icon)?, None)?;
    if icon.size[0] != icon.size[1] {
        return Err("The pack icon must be square.".into());
    }
    Ok(PackData {
        manifest: m,
        player,
        pursuers,
        icon,
    })
}
pub fn load_folder(root: &Path) -> Result<PackData, String> {
    let text =
        storage::read_bounded(&safe_file(root, "pack.toml")?, 32_768).map_err(|e| e.to_string())?;
    let m = parse(std::str::from_utf8(&text).map_err(|e| e.to_string())?)?;
    load_with(m, |file| {
        storage::read_bounded(&safe_file(root, file)?, MAX_FILE).map_err(|e| e.to_string())
    })
}
pub fn embedded() -> Result<PackData, String> {
    let m = parse(include_str!("../assets/packs/latch/pack.toml"))?;
    load_with(m, |file| match file {
        "player.png" => Ok(include_bytes!("../assets/packs/latch/player.png").to_vec()),
        "pursuers.png" => Ok(include_bytes!("../assets/packs/latch/pursuers.png").to_vec()),
        "icon.png" => Ok(include_bytes!("../assets/packs/latch/icon.png").to_vec()),
        _ => Err("Unknown embedded asset.".into()),
    })
}
pub fn load_choice_at(key: &str, root: &Path) -> Result<Option<PackData>, String> {
    match key {
        LATCH => embedded().map(Some),
        ORIGINAL => Ok(None),
        key if key.starts_with("folder:") => {
            let folder = &key[7..];
            if !valid_folder(folder) {
                return Err("Invalid pack folder name.".into());
            }
            let base = root
                .canonicalize()
                .map_err(|e| format!("Packs folder is unavailable: {e}"))?;
            let pack = root
                .join(folder)
                .canonicalize()
                .map_err(|e| format!("Pack '{folder}' is unavailable: {e}"))?;
            if !pack.starts_with(base) {
                return Err("Pack folder must remain inside the packs directory.".into());
            }
            load_folder(&pack).map(Some)
        }
        _ => Err("Unknown character pack.".into()),
    }
}
pub fn choices_at(root: &Path) -> Vec<Choice> {
    let mut choices = vec![
        Choice {
            key: LATCH.into(),
            name: "Latch".into(),
            error: None,
        },
        Choice {
            key: ORIGINAL.into(),
            name: "Original".into(),
            error: None,
        },
    ];
    let mut folders = BTreeMap::new();
    if let Ok(entries) = fs::read_dir(root) {
        for e in entries.take(128).flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if valid_folder(&name) && e.file_type().is_ok_and(|t| t.is_dir()) {
                folders.insert(name, e.path());
            }
        }
    }
    for (name, path) in folders {
        let result = safe_file(&path, "pack.toml")
            .and_then(|p| storage::read_bounded(&p, 32_768).map_err(|e| e.to_string()))
            .and_then(|b| String::from_utf8(b).map_err(|e| e.to_string()))
            .and_then(|s| parse(&s));
        match result {
            Ok(m) => choices.push(Choice {
                key: format!("folder:{name}"),
                name: m.name,
                error: None,
            }),
            Err(e) => choices.push(Choice {
                key: format!("folder:{name}"),
                name,
                error: Some(e),
            }),
        }
    }
    choices
}
pub fn export_template(root: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let temporary = tempfile::Builder::new()
        .prefix(".pack-staging-")
        .tempdir_in(root)
        .map_err(|e| e.to_string())?;
    let dir = temporary.path();
    let manifest = include_str!("../assets/packs/latch/pack.toml").replacen(
        "name = \"Latch\"",
        "name = \"My characters\"",
        1,
    );
    fs::write(dir.join("pack.toml"), manifest).map_err(|e| e.to_string())?;
    fs::write(
        dir.join("ARTWORK-PACKS.md"),
        include_str!("../docs/ARTWORK-PACKS.md"),
    )
    .map_err(|e| e.to_string())?;
    fs::write(dir.join("LICENSE"), include_str!("../LICENSE")).map_err(|e| e.to_string())?;
    for (name, bytes) in [
        (
            "player.png",
            include_bytes!("../assets/packs/latch/player.png").as_slice(),
        ),
        (
            "pursuers.png",
            include_bytes!("../assets/packs/latch/pursuers.png").as_slice(),
        ),
        (
            "icon.png",
            include_bytes!("../assets/packs/latch/icon.png").as_slice(),
        ),
    ] {
        fs::write(dir.join(name), bytes).map_err(|e| e.to_string())?;
    }
    // Never replace the artist's existing work.
    let destination = (1..=1000)
        .map(|i| {
            root.join(if i == 1 {
                "my-characters".into()
            } else {
                format!("my-characters-{i}")
            })
        })
        .find(|p| !p.exists())
        .ok_or("Too many exported templates.")?;
    fs::rename(dir, &destination).map_err(|e| e.to_string())?;
    Ok(destination)
}
impl Artwork {
    pub fn upload(ctx: &egui::Context, data: PackData) -> Self {
        let texture = |name: &str, p: &Pixels| {
            ctx.load_texture(
                name,
                egui::ColorImage::from_rgba_unmultiplied(p.size, &p.rgba),
                egui::TextureOptions::LINEAR.with_mipmap_mode(Some(egui::TextureFilter::Linear)),
            )
        };
        let player = texture("character-player", &data.player);
        let pursuers = texture("character-pursuers", &data.pursuers);
        let icon = texture("character-icon", &data.icon);
        Self {
            name: data.manifest.name.clone(),
            author: data.manifest.author.clone(),
            license: data.manifest.license.clone(),
            manifest: data.manifest,
            player,
            pursuers,
            icon,
            icon_data: egui::IconData {
                rgba: data.icon.rgba,
                width: data.icon.size[0] as u32,
                height: data.icon.size[1] as u32,
            },
        }
    }
    pub fn player(
        &self,
        p: &egui::Painter,
        c: Pos2,
        side: f32,
        dir: Dir,
        time: f32,
        animated: bool,
    ) {
        let frame = if animated {
            ((time * self.manifest.fps) as usize) % self.manifest.player.frames.len()
        } else {
            0
        };
        draw(
            p,
            &self.player,
            self.manifest.player.frames[frame],
            c,
            side * self.manifest.scale,
            dir,
        );
    }
    pub fn pursuer(&self, p: &egui::Painter, c: Pos2, side: f32, id: usize, state: PursuerState) {
        let frame = if matches!(state, PursuerState::Normal) {
            self.manifest.normal[id]
        } else {
            self.manifest.vulnerable[id]
        };
        draw(
            p,
            &self.pursuers,
            self.manifest.pursuers.frames[frame],
            c,
            side * self.manifest.scale
                * if matches!(state, PursuerState::Returning) {
                    0.58
                } else {
                    1.
                },
            Dir::Right,
        );
    }
    pub fn preview(&self, ui: &mut egui::Ui) {
        let (r, _) = ui.allocate_exact_size(Vec2::new(240., 56.), egui::Sense::hover());
        let c = r.left_center() + Vec2::new(26., 0.);
        self.player(ui.painter(), c, 44., Dir::Right, 0., false);
        for i in 0..4 {
            self.pursuer(
                ui.painter(),
                c + Vec2::new(52. + i as f32 * 45., 0.),
                36.,
                i,
                PursuerState::Normal,
            );
        }
    }
}
fn draw(
    p: &egui::Painter,
    texture: &TextureHandle,
    frame: [u32; 4],
    center: Pos2,
    side: f32,
    dir: Dir,
) {
    let [x, y, w, h] = frame;
    let size = texture.size_vec2();
    let uv = Rect::from_min_max(
        Pos2::new(x as f32 / size.x, y as f32 / size.y),
        Pos2::new((x + w) as f32 / size.x, (y + h) as f32 / size.y),
    );
    let mut mesh = egui::Mesh::with_texture(texture.id());
    mesh.add_rect_with_uv(
        Rect::from_center_size(center, Vec2::splat(side)),
        uv,
        Color32::WHITE,
    );
    let angle = match dir {
        Dir::Right => 0.,
        Dir::Down => std::f32::consts::FRAC_PI_2,
        Dir::Left => std::f32::consts::PI,
        Dir::Up => -std::f32::consts::FRAC_PI_2,
    };
    let rotation = egui::emath::Rot2::from_angle(angle);
    for v in &mut mesh.vertices {
        v.pos = center + rotation * (v.pos - center);
    }
    p.add(egui::Shape::mesh(mesh));
}

#[cfg(test)]
mod tests {
    use super::*;
    fn png() -> Vec<u8> {
        let mut bytes = std::io::Cursor::new(Vec::new());
        let mut image = image::RgbaImage::from_pixel(4, 4, image::Rgba([240, 240, 220, 255]));
        image.put_pixel(0, 0, image::Rgba([251, 4, 250, 255]));
        image.put_pixel(1, 0, image::Rgba([20, 30, 40, 0]));
        image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
        bytes.into_inner()
    }
    fn manifest() -> Manifest {
        let mut m = parse(include_str!("../assets/packs/latch/pack.toml")).unwrap();
        m.player.frames = vec![[0, 0, 4, 4]];
        m.pursuers.frames = vec![[0, 0, 4, 4]; 8];
        m
    }
    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("pack.toml"),
            toml::to_string(&manifest()).unwrap(),
        )
        .unwrap();
        for name in ["player.png", "pursuers.png", "icon.png"] {
            fs::write(dir.path().join(name), png()).unwrap();
        }
        dir
    }
    #[test]
    fn embedded_atlases_validate_and_key_without_touching_source() {
        let data = embedded().unwrap();
        assert_eq!(data.manifest.name, "Latch");
        assert_eq!(data.player.size, [1536, 1024]);
        assert_eq!(data.player.rgba[3], 0);
        assert!(data
            .player
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[3] == 255));
        assert_eq!(data.pursuers.rgba[3], 0);
    }
    #[test]
    fn colour_key_and_existing_alpha_both_work() {
        let m = manifest();
        let p = decode(&png(), Some(&m.player)).unwrap();
        assert_eq!(
            &p.rgba[3..12].iter().step_by(4).copied().collect::<Vec<_>>(),
            &[0, 0, 255]
        );
        let mut atlas = m.player;
        atlas.key_color = None;
        assert_eq!(decode(&png(), Some(&atlas)).unwrap().rgba[3], 255);
    }
    #[test]
    fn bad_geometry_indices_schema_and_png_fail_clearly() {
        let mut m = manifest();
        m.player.frames[0] = [3, 0, 4, 4];
        assert!(decode(&png(), Some(&m.player))
            .unwrap_err()
            .contains("contained"));
        m.player.frames[0] = [u32::MAX, 0, 4, 4];
        assert!(decode(&png(), Some(&m.player)).is_err());
        m.normal[0] = 999;
        assert!(parse(&toml::to_string(&m).unwrap())
            .unwrap_err()
            .contains("index"));
        m.normal[0] = 0;
        m.schema = 2;
        assert!(parse(&toml::to_string(&m).unwrap())
            .unwrap_err()
            .contains("schema"));
        assert!(decode(b"not a PNG", None).is_err());
    }
    #[test]
    fn fully_invisible_frames_are_rejected() {
        let mut m = manifest();
        m.player.frames = vec![[0, 0, 1, 1]];
        assert!(decode(&png(), Some(&m.player))
            .unwrap_err()
            .contains("transparent"));
    }
    #[test]
    fn external_pack_loads_and_missing_assets_fail() {
        let dir = fixture();
        assert_eq!(load_folder(dir.path()).unwrap().manifest.name, "Latch");
        fs::remove_file(dir.path().join("pursuers.png")).unwrap();
        assert!(load_folder(dir.path())
            .err()
            .unwrap()
            .contains("pursuers.png"));
    }
    #[test]
    fn traversal_and_asset_symlink_escape_are_rejected() {
        let dir = fixture();
        assert!(safe_file(dir.path(), "../pack.toml").is_err());
        assert!(safe_file(dir.path(), "/tmp/test.png").is_err());
        assert!(load_choice_at("folder:../escape", dir.path()).is_err());
        #[cfg(unix)]
        {
            let other = fixture();
            std::os::unix::fs::symlink(
                other.path().join("icon.png"),
                dir.path().join("escape.png"),
            )
            .unwrap();
            assert!(safe_file(dir.path(), "escape.png").is_err());
            std::os::unix::fs::symlink(other.path(), dir.path().join("escape")).unwrap();
            assert!(load_choice_at("folder:escape", dir.path()).is_err());
            assert_eq!(choices_at(dir.path()).len(), 2);
        }
    }
    #[test]
    fn exporting_twice_keeps_artist_edits() {
        let root = tempfile::tempdir().unwrap();
        let first = export_template(root.path()).unwrap();
        fs::write(first.join("icon.png"), b"artist work").unwrap();
        let second = export_template(root.path()).unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read(first.join("icon.png")).unwrap(), b"artist work");
        let data = load_folder(&second).unwrap();
        assert_eq!(data.manifest.name, "My characters");
        assert_eq!(choices_at(root.path()).len(), 4);
    }
}
