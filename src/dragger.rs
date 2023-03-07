use macroquad::prelude::*;
#[path = "board.rs"] mod board;
pub use board::*;


pub struct Dragger {
    pub piece: Option<Piece>,
    pub dragging: bool,
    pub mousex: f32,
    pub mousey: f32,
    pub init_row: isize,
    pub init_col: isize
}

impl Dragger {
    pub fn new() -> Self {
        Self {
            piece: None,
            dragging: false,
            mousex: 0.0,
            mousey: 0.0,
            init_row: 0,
            init_col: 0
        }
    }
    pub async fn draw(&mut self) {
        let piece: &mut Piece; 
        match &mut self.piece {
            Some(val) => {piece = val},
            None => {return}
        };
        let im = load_texture(&piece.texture_path(128)).await.unwrap();
        draw_texture(
            im, 
            self.mousex - im.width() / 2.0,
            self.mousey - im.height() / 2.0,
            Color::new(1.0, 1.0, 1.0, 1.0),
        )
    }
    pub fn update_mouse(&mut self, position: (f32, f32)) {
        (self.mousex, self.mousey) = position;
    }
    pub fn save_initial(&mut self, position: (f32, f32), tilesize: f32) {
        self.init_row = (position.1 / tilesize) as isize;
        self.init_col = (position.0 / tilesize) as isize;
    }
    pub fn begin_drag(&mut self, piece: &Piece) {
        self.piece = Some(*piece);
        self.dragging = true;
    }
    pub fn end_drag(&mut self) {
        self.piece = None;
        self.dragging = false;
    }
}