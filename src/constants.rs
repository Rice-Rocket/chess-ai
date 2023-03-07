use macroquad::prelude::Color;


pub const ROWS: usize = 8;
pub const COLS: usize = 8;
pub const DEFAULT_THEME: usize = 1;
pub const BG_COLOR: Color = Color::new(0.13, 0.125, 0.13, 1.0);

// pub const ZOBRIST_FILE: String = String::from("internal/zobrist.bin");
pub struct Constants {
    pub zobrist_file: String
}
impl Constants {
    pub fn new() -> Self {
        Self {
            zobrist_file: "internal/zobrist.bin".to_string(),
        }
    }
}