use macroquad::prelude::*;
use macroquad::audio::{load_sound, Sound};
#[path = "constants.rs"] mod constants;
use constants::DEFAULT_THEME;

#[derive(Clone, Copy)]
pub struct ColorPair {
    pub light: Color,
    pub dark: Color
}

impl ColorPair {
    fn new(light: Color, dark: Color) -> Self {
        Self {
            light: light,
            dark: dark
        }
    }
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub bg: ColorPair,
    pub trace: ColorPair,
    pub moves: ColorPair,
    pub title_color: Color
}

impl Theme {
    fn new(light_bg: Color, dark_bg: Color, light_trace: Color, dark_trace: Color, light_moves: Color, dark_moves: Color, title_color: Color) -> Self {
        Self {
            bg: ColorPair::new(light_bg, dark_bg),
            trace: ColorPair::new(light_trace, dark_trace),
            moves: ColorPair::new(light_moves, dark_moves),
            title_color: title_color
        }
    }
}



pub struct Config {
    pub idx: usize,
    pub theme: Theme,
    pub themes: Vec<Theme>,
    pub font: Font,
    pub move_sound: Sound,
    pub capture_sound: Sound
}

impl Config {
    pub async fn new() -> Self {
        let green = Theme::new(
            Color::from_rgba(234, 235, 200, 255), Color::from_rgba(119, 154, 88, 255),
            Color::from_rgba(244, 247, 116, 255), Color::from_rgba(172, 19, 51, 255),
            Color::from_rgba(200, 100, 100, 255), Color::from_rgba(200, 70, 70, 255),
            Color::from_rgba(230, 230, 230, 255)
        );
        let brown = Theme::new(
            Color::from_rgba(235, 209, 166, 255), Color::from_rgba(165, 117, 80, 255),
            Color::from_rgba(245, 234, 100, 255), Color::from_rgba(209, 185, 59, 255),
            Color::from_rgba(200, 100, 100, 255), Color::from_rgba(200, 70, 70, 255),
            Color::from_rgba(230, 230, 230, 255)
        );
        let blue = Theme::new(
            Color::from_rgba(229, 228, 200, 255), Color::from_rgba(60, 95, 135, 255),
            Color::from_rgba(123, 187, 227, 255), Color::from_rgba(43, 119, 191, 255),
            Color::from_rgba(200, 100, 100, 255), Color::from_rgba(200, 70, 70, 255),
            Color::from_rgba(230, 230, 230, 255)
        );
        let gray = Theme::new(
            Color::from_rgba(234, 235, 200, 255), Color::from_rgba(119, 154, 88, 255),
            Color::from_rgba(99, 126, 143, 255), Color::from_rgba(82, 102, 128, 255),
            Color::from_rgba(200, 100, 100, 255), Color::from_rgba(200, 70, 70, 255),
            Color::from_rgba(230, 230, 230, 255)
        );
        let themes = Vec::from([green, brown, blue, gray]);
        Self {
            idx: DEFAULT_THEME,
            theme: themes[DEFAULT_THEME],
            themes: themes,
            font: load_ttf_font("assets/Monaco.ttf").await.unwrap(),
            move_sound: load_sound("assets/sounds/move.wav").await.unwrap(),
            capture_sound: load_sound("assets/sounds/capture.wav").await.unwrap(),
        }
    }
    pub fn change_theme(&mut self) {
        self.idx = (self.idx + 1) % self.themes.len();
        self.theme = self.themes[self.idx];
    }
}