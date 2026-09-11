use crate::storage::{self, Appearance};
use eframe::egui::Color32;
use std::path::PathBuf;

pub fn hex(value: &str) -> Option<Color32> {
    if value.len() != 7
        || !value.starts_with('#')
        || !value[1..].bytes().all(|v| v.is_ascii_hexdigit())
    {
        return None;
    }
    let n = u32::from_str_radix(&value[1..], 16).ok()?;
    Some(Color32::from_rgb((n >> 16) as u8, (n >> 8) as u8, n as u8))
}
pub fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    let c = |x: u8, y: u8| (x as f32 * (1. - t) + y as f32 * t).round() as u8;
    Color32::from_rgb(c(a.r(), b.r()), c(a.g(), b.g()), c(a.b(), b.b()))
}
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let luminance = |c: Color32| {
        let f = |v: u8| {
            let s = v as f32 / 255.;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
    };
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
pub fn readable(c: Color32, bg: Color32, minimum: f32) -> Color32 {
    if contrast(c, bg) >= minimum {
        return c;
    }
    let target = if contrast(Color32::WHITE, bg) > contrast(Color32::BLACK, bg) {
        Color32::WHITE
    } else {
        Color32::BLACK
    };
    for step in 1..=100 {
        let adjusted = blend(c, target, step as f32 / 100.);
        if contrast(adjusted, bg) >= minimum {
            return adjusted;
        }
    }
    target
}
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    pub background: Color32,
    pub surface: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub field: Color32,
    pub wall: Color32,
    pub wall_fill: Color32,
    pub dot: Color32,
    pub player: Color32,
    pub pursuers: [Color32; 4],
    pub name: String,
}
impl Palette {
    pub fn new(bg: Color32, fg: Color32, accent: Color32, name: &str) -> Self {
        let text = readable(fg, bg, 4.5);
        let surface = bg;
        let field = blend(bg, Color32::BLACK, 0.09);
        let accent = readable(accent, field, 4.5);
        let wall = readable(blend(accent, field, 0.27), field, 3.);
        Self {
            background: bg,
            surface,
            text,
            muted: readable(blend(text, bg, 0.35), bg, 4.5),
            accent,
            field,
            wall,
            wall_fill: blend(field, accent, 0.10),
            dot: readable(blend(fg, accent, 0.2), field, 4.5),
            player: readable(hex("#eed18a").unwrap(), field, 4.5),
            pursuers: ["#df907e", "#beb1e1", "#80c2bb", "#dcaa6d"]
                .map(|c| readable(hex(c).unwrap(), field, 3.)),
            name: name.into(),
        }
    }
    pub fn load(appearance: Appearance) -> Self {
        if appearance == Appearance::Omarchy {
            if let Some(p) = Self::load_from(&Self::paths()) {
                return p;
            }
        }
        if appearance == Appearance::Ivory {
            Self::new(
                hex("#e4e8df").unwrap(),
                hex("#171c1a").unwrap(),
                hex("#4e673b").unwrap(),
                "Ivory",
            )
        } else {
            Self::new(
                hex("#171c1a").unwrap(),
                hex("#e4e8df").unwrap(),
                hex("#b3cb92").unwrap(),
                if appearance == Appearance::Omarchy {
                    "Omarchy · fallback"
                } else {
                    "Charcoal"
                },
            )
        }
    }
    fn paths() -> Vec<PathBuf> {
        vec![
            storage::xdg("XDG_STATE_HOME", ".local/state")
                .join("omarchy/current/theme/colors.toml"),
            storage::xdg("XDG_CONFIG_HOME", ".config").join("omarchy/current/theme/colors.toml"),
        ]
    }
    pub fn load_from(paths: &[PathBuf]) -> Option<Self> {
        paths.iter().find_map(|p| {
            let bytes = storage::read_bounded(p, 65536).ok()?;
            let table = std::str::from_utf8(&bytes)
                .ok()?
                .parse::<toml::Table>()
                .ok()?;
            let get = |k: &str| hex(table.get(k)?.as_str()?);
            Some(Self::new(
                get("background")?,
                get("foreground")?,
                get("accent").unwrap_or(hex("#b3cb92").unwrap()),
                "Omarchy",
            ))
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn light_dark_and_low_contrast_themes_stay_readable() {
        for bg in ["#171c1a", "#e4e8df", "#808080", "#ffffff", "#000000"] {
            let c = hex(bg).unwrap();
            let p = Palette::new(c, c, c, "test");
            assert!(contrast(p.text, p.background) >= 4.5);
            assert!(contrast(p.accent, p.field) >= 4.5);
            assert!(contrast(p.wall, p.field) >= 3.);
            assert!(contrast(p.player, p.field) >= 4.5);
        }
    }
    #[test]
    fn malformed_and_unicode_colors_are_rejected() {
        for s in ["#éaaaa", "#zzzzzz", "#12345", "1234567"] {
            assert_eq!(hex(s), None);
        }
    }
    #[test]
    fn invalid_theme_falls_through() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("a");
        let b = d.path().join("b");
        std::fs::write(&a, "broken").unwrap();
        std::fs::write(
            &b,
            "background = '#ffffff'\nforeground = '#111111'\naccent = '#557733'",
        )
        .unwrap();
        assert!(Palette::load_from(&[a, b]).is_some());
    }
}
