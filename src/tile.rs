use std::collections::HashMap;
#[path = "piece.rs"] mod piece;
pub use piece::*;


#[derive(Clone)]
pub struct Move {
    pub initial: Tile,
    pub end: Tile,
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} -> {:?}", self.initial, self.end)
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        (self.initial == other.initial) && (self.end == other.end)
    }
    fn ne(&self, other: &Self) -> bool {
        (self.initial != other.initial) || (self.end != other.end)
    }
}

impl Move {
    pub fn new(initial: Tile, end: Tile) -> Self {
        Self {
            initial: initial,
            end: end,
        }
    }
    pub fn copy(&self) -> Move {
        return Move {
            initial: self.initial.copy(),
            end: self.end.copy()
        };
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum MoveType {
    Castle,
    NoCastle
}

#[derive(Clone)]
pub struct Tile {
    pub row: isize,
    pub col: isize,
    pub bonuses: HashMap<(PieceType, Team), f32>,
    pub present_piece: Option<Piece>,
}

impl std::fmt::Debug for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_piece() {
            write!(f, "({}, {}, {})", self.row, self.col, self.piece().ptype.as_string())
        }
        else {
            write!(f, "({}, {})", self.row, self.col)
        }
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        (self.row == other.row) && (self.col == other.col)
    }
    fn ne(&self, other: &Self) -> bool {
        (self.row != other.row) || (self.col != other.col)
    }
}

impl Tile {
    pub fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            present_piece: None,
            bonuses: HashMap::new()
        }
    }
    pub fn init(&mut self, row: isize, col: isize, piece: Option<Piece>) {
        self.row = row;
        self.col = col;
        self.present_piece = piece;
    }
    pub fn piece(&self) -> &Piece {
        self.present_piece.as_ref().unwrap()
    }
    pub fn piece_mut(&mut self) -> &mut Piece {
        self.present_piece.as_mut().unwrap()
    }
    pub fn has_piece(&self) -> bool {
        match &self.present_piece {
            Some(_) => true,
            None => false
        }
    }
    pub fn is_empty(&self) -> bool {
        !self.has_piece()
    }
    pub fn has_team(&self, color: Team) -> bool {
        self.has_piece() && self.piece().color == color
    }
    pub fn has_rival(&self, color: Team) -> bool {
        self.has_piece() && self.piece().color != color
    }
    pub fn is_empty_or_rival(&self, color: Team) -> bool {
        self.is_empty() || self.has_rival(color)
    }
    pub fn copy(&self) -> Tile {
        return Tile {
            row: self.row,
            col: self.col,
            bonuses: self.bonuses.clone(),
            present_piece: self.present_piece.clone()
        };
    }
}