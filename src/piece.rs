#[path = "team.rs"] mod team;
pub use team::Team;

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None
}

impl PieceType {
    pub fn as_string(&self) -> String {
        match *self {
            Self::Pawn => "pawn".to_string(),
            Self::Knight => "knight".to_string(),
            Self::Bishop => "bishop".to_string(),
            Self::Rook => "rook".to_string(),
            Self::Queen => "queen".to_string(),
            Self::King => "king".to_string(),
            Self::None => "".to_string()
        }
    }
    pub fn as_int(&self) -> i32 {
        match *self {
            Self::Pawn => 1,
            Self::Knight => 2,
            Self::Bishop => 3,
            Self::Rook => 4,
            Self::Queen => 5,
            Self::King => 6,
            Self::None => 0
        }
    }
    pub fn value_mg(&self) -> i32 {
        match *self {
            Self::Pawn => 124,
            Self::Knight => 781,
            Self::Bishop => 825,
            Self::Rook => 1276,
            Self::Queen => 2538,
            Self::King => 0,
            Self::None => 0
        }
    }
    pub fn value_eg(&self) -> i32 {
        match *self {
            Self::Pawn => 206,
            Self::Knight => 854,
            Self::Bishop => 915,
            Self::Rook => 1380,
            Self::Queen => 2682,
            Self::King => 0,
            Self::None => 0
        }
    }
}


#[derive(Eq, Hash, Clone, Copy)]
pub struct Piece {
    pub ptype: PieceType,
    pub uid: isize,
    pub color: Team,
    pub value_mg: i32,
    pub value_eg: i32,
    pub has_moved: bool,

    pub row: usize,
    pub col: usize,

    pub en_passant: bool,
    pub dir: isize,
}

impl PartialEq for Piece {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
    fn ne(&self, other: &Self) -> bool {
        self.uid != other.uid
    }
}

impl Piece {
    pub fn new(ptype: PieceType, color: Team, uid: isize, row: usize, col: usize) -> Self {
        Self {
            ptype: ptype,
            uid: uid,
            color: color,
            value_mg: ptype.value_mg(),
            value_eg: ptype.value_eg(),
            has_moved: false,

            row: row,
            col: col,

            en_passant: false,
            dir: if color == Team::White { -1 } else { 1 }
        }
    }
    // pub fn set_texture(&mut self, size: usize) {
    //     self.texture_path = format!("../assets/images/imgs-{}px/{}_{}.png", size, self.name, self.color.as_string());
    // }
    pub fn texture_path(&self, size: usize) -> String {
        format!("assets/images/imgs-{}px/{}_{}.png", size, self.color.as_string(), self.ptype.as_string())
    }
    pub fn make_moved(&mut self) {
        self.has_moved = true;
    }
    pub fn copy(&self) -> Self {
        Self {
            ptype: self.ptype,
            uid: self.uid,
            color: self.color,
            value_mg: self.value_mg,
            value_eg: self.value_eg,
            has_moved: self.has_moved,
            en_passant: self.en_passant,
            row: self.row,
            col: self.col,
            dir: self.dir
        }
    }
}


