
#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum Team {
    White,
    Black,
    None
}


impl Team {
    pub fn other(&self) -> Self {
        match *self {
            Self::White => Self::Black,
            Self::Black => Self::White,
            Self::None => Self::None
        }
    }
    pub fn as_string(&self) -> String {
        match *self {
            Self::White => "White".to_string(),
            Self::Black => "Black".to_string(),
            Self::None => "".to_string()
        }
    }
    pub fn as_bool(&self) -> bool {
        match *self {
            Self::White => true,
            Self::Black => true,
            Self::None => false
        }
    }
    pub fn as_int(&self) -> i32 {
        match *self {
            Self::White => 1,
            Self::Black => 0,
            Self::None => 0
        }
    }
}