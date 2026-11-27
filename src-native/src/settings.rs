use crate::{audio::EqSettings, player::Repeat};
use eframe::egui::{self, Color32};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub palette: String,
    pub wallpaper: Option<String>,
    pub wallpaper_opacity: f32,
    pub volume: f32,
    pub shuffle: bool,
    pub repeat: Repeat,
    pub auto_related: bool,
    pub eq: EqSettings,
    pub layout: String,
    pub rpm: f32,
    pub vinyl_color: [u8; 3],
    pub skin: String,
    pub cover_position: String,
    pub tonearm: bool,
    pub font_size: f32,
    pub dim_inactive: f32,
    pub background: String,
    pub island: bool,
    pub stealth: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            palette: "catppuccin-mocha".into(),
            wallpaper: None,
            wallpaper_opacity: 0.15,
            volume: 0.8,
            shuffle: false,
            repeat: Repeat::All,
            auto_related: true,
            eq: EqSettings::default(),
            layout: "vinyl".into(),
            rpm: 33.,
            vinyl_color: [203, 166, 247],
            skin: "marble".into(),
            cover_position: "full".into(),
            tonearm: true,
            font_size: 36.,
            dim_inactive: 0.5,
            background: "apple-glass".into(),
            island: false,
            stealth: true,
        }
    }
}
impl Settings {
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = crate::lyrics::read_limited(path, 1024 * 1024)?;
        let mut s: Self = serde_json::from_slice(&bytes)
            .map_err(|_| "Không đọc được cài đặt native. Tệp gốc được giữ nguyên.")?;
        s.volume = if s.volume.is_finite() {
            s.volume.clamp(0., 1.)
        } else {
            0.8
        };
        s.rpm = s.rpm.clamp(20., 78.);
        s.font_size = s.font_size.clamp(16., 72.);
        s.wallpaper_opacity = s.wallpaper_opacity.clamp(0., 0.6);
        Ok(s)
    }
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        if path.exists() {
            std::fs::copy(path, path.with_extension("json.bak")).map_err(|e| e.to_string())?;
        }
        crate::lyrics::atomic_json(path, self, false)
    }
}
#[derive(Clone, Deserialize)]
pub struct Palette {
    pub id: String,
    pub family: String,
    pub name: String,
    pub scheme: String,
    pub colors: HashMap<String, String>,
}
pub fn palettes() -> Vec<Palette> {
    serde_json::from_str(include_str!("themes.json")).expect("Embedded theme data must be valid")
}
pub fn color(hex: &str) -> Color32 {
    if hex.len() == 7
        && let Ok(n) = u32::from_str_radix(&hex[1..], 16)
    {
        return Color32::from_rgb((n >> 16) as u8, (n >> 8) as u8, n as u8);
    }
    Color32::LIGHT_GRAY
}
pub fn apply(ctx: &egui::Context, palette: &Palette) {
    let get = |key: &str| palette.colors.get(key).map_or(Color32::GRAY, |h| color(h));
    let mut visuals = if palette.scheme == "light" {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    visuals.panel_fill = get("base");
    visuals.window_fill = get("surface");
    visuals.extreme_bg_color = get("sidebar");
    visuals.faint_bg_color = get("surface");
    visuals.override_text_color = Some(get("text"));
    visuals.selection.bg_fill = get("accent");
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, readable(get("text"), get("accent")));
    visuals.widgets.inactive.bg_fill = get("surface");
    visuals.widgets.hovered.bg_fill = get("overlay");
    visuals.widgets.active.bg_fill = get("overlay");
    visuals.widgets.inactive.fg_stroke.color = get("text");
    visuals.widgets.hovered.fg_stroke.color = get("text");
    visuals.widgets.active.fg_stroke.color = get("text");
    visuals.warn_fg_color = readable(Color32::from_rgb(249, 226, 175), get("base"));
    visuals.error_fg_color = readable(get("danger"), get("base"));
    ctx.set_visuals(visuals);
}
fn luminance(c: Color32) -> f32 {
    let f = |x: u8| {
        let x = x as f32 / 255.;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    };
    f(c.r()) * 0.2126 + f(c.g()) * 0.7152 + f(c.b()) * 0.0722
}
fn contrast(a: Color32, b: Color32) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
pub fn readable(mut c: Color32, bg: Color32) -> Color32 {
    let target = if contrast(Color32::WHITE, bg) > contrast(Color32::BLACK, bg) {
        Color32::WHITE
    } else {
        Color32::BLACK
    };
    for _ in 0..100 {
        if contrast(c, bg) >= 4.5 {
            break;
        }
        c = Color32::from_rgb(
            (c.r() as f32 * 0.9 + target.r() as f32 * 0.1).round() as u8,
            (c.g() as f32 * 0.9 + target.g() as f32 * 0.1).round() as u8,
            (c.b() as f32 * 0.9 + target.b() as f32 * 0.1).round() as u8,
        );
    }
    c
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_themes_and_contrast() {
        let p = palettes();
        assert_eq!(p.len(), 31);
        for p in p {
            let bg = color(&p.colors["base"]);
            assert!(contrast(readable(color(&p.colors["accent"]), bg), bg) >= 4.5);
        }
    }
}
