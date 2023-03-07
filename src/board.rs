use std::collections::HashMap;
use std::fs::File;
use rand::{thread_rng, Rng};
use bincode::{serialize_into, deserialize_from};
use itertools::izip;

#[path = "tile.rs"] mod tile;
pub use tile::*;
#[path = "constants.rs"] mod constants;
pub use constants::*;
#[path = "utils.rs"] mod utils;
pub use utils::*;


pub struct Board {
    pub move_log: Vec<(Move, Piece, Option<Piece>, Option<(isize, isize)>, bool, MoveType)>,
    pub valid_moves: HashMap<Piece, Vec<Move>>,
    pub tiles: [[Tile; COLS]; ROWS],
    zobrist_keys: [[u64; 12]; (ROWS * COLS)],
    castle_moves: HashMap<Piece, Option<Move>>,
    castle_rooks: HashMap<Piece, [Option<Piece>; 2]>,

    checkmated: Team,
    stalemate: bool,
    game_stage: i32,
    cur_uid: isize
}

impl Board {
    pub fn new() -> Self {
        let mut tiles = [
            [Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new()],
            [Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new()],
            [Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new()],
            [Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new()],
            [Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new()],
            [Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new()],
            [Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new()],
            [Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new(), Tile::new()]
        ];
        for i in 0..ROWS {
            for j in 0..COLS {
                tiles[i][j].init(i as isize, j as isize, None);
            }
        }

        let mut cur_uid = add_pieces(&mut tiles, Team::White, 0);
        cur_uid = add_pieces(&mut tiles, Team::Black, cur_uid);
        psqt_bonuses(&mut tiles);
        Self {
            move_log: Vec::new(),
            valid_moves: HashMap::new(),
            tiles: tiles,
            zobrist_keys: create_zobrist_keys(),
            castle_moves: HashMap::new(),
            castle_rooks: HashMap::new(),

            checkmated: Team::None,
            stalemate: false,
            game_stage: 0,
            cur_uid: cur_uid
        }
    }
    pub fn evaluate(&self, perspective: Team) -> f32 {
        let eval = self.accumulate_material(perspective);
        return eval;
    }
    pub fn accumulate_material(&self, color: Team) -> f32 {
        let mut v = 0.0;
        for row in self.tiles.iter() {
            for tile in row.iter() {
                if tile.has_team(color) {
                    v += tile.piece().value_mg as f32;
                }
            };
        };
        return v;
    }
    pub fn is_terminal(&mut self) -> bool {
        self.in_checkmate(Team::Black) || self.in_checkmate(Team::White)
    }
    pub fn zobrist_hash(&self) -> u64 {
        let mut hash_val = 0;
        for (i, row) in self.tiles.iter().enumerate() {
            for (j, tile) in row.iter().enumerate() {
                if tile.has_piece() {
                    let piece_val = (tile.piece().ptype.as_int() + tile.piece().color.as_int() * 6 - 1) as usize;
                    hash_val ^= self.zobrist_keys[i * 8 + j][piece_val];
                }
            }
        }
        return hash_val;
    }
    pub fn clear_moves(&mut self) {
        self.valid_moves.clear();
    }
    pub fn simulate_move(&mut self, piece: Piece, action: Move) {
        self.tiles[action.initial.row as usize][action.initial.col as usize].present_piece = None;
        self.tiles[action.end.row as usize][action.end.col as usize].present_piece = Some(piece);
    }
    pub fn undo_simulate_move(&mut self, piece: Piece, action: Move) {
        self.tiles[action.end.row as usize][action.end.col as usize].present_piece = None;
        self.tiles[action.initial.row as usize][action.initial.col as usize].present_piece = Some(piece);
    }
    pub fn execute_move(&mut self, piece: &mut Piece, action: Move, simulation: bool, ignore_castle: bool) {
        let start = &action.initial;
        let end = &action.end;

        let mut removed_piece: Option<Piece> = None;
        let mut removed_pos: Option<(isize, isize)> = None;
        let mut was_castle: bool = false;
        let en_passant_empty = self.tiles[end.row as usize][end.col as usize].is_empty();

        if self.tiles[end.row as usize][end.col as usize].has_piece() {
            removed_piece = Some(*self.tiles[end.row as usize][end.col as usize].piece());
            removed_pos = Some((end.row, end.col));
        }
        
        self.tiles[start.row as usize][start.col as usize].present_piece = None;
        self.tiles[end.row as usize][end.col as usize].present_piece = Some(*piece);

        if piece.ptype == PieceType::Pawn {
            let diff = end.col - start.col;
            if (diff != 0) && en_passant_empty && (self.tiles[start.row as usize][(start.col + diff) as usize].piece().ptype == PieceType::Pawn) && self.tiles[start.row as usize][(start.col + diff) as usize].piece().en_passant {
                removed_piece = self.tiles[start.row as usize][(start.col + diff) as usize].present_piece;
                removed_pos = Some((start.row, start.col + diff));
                self.tiles[start.row as usize][(start.col + diff) as usize].present_piece = None;
            }
            self.check_promotion(*piece, end.copy());
        }

        if piece.ptype == PieceType::King {
            if self.is_castling(start.copy(), end.copy()) && !simulation && !ignore_castle {
                let diff = end.copy().col as isize - start.copy().col as isize;
                let mut rook;
                if diff < 0 {
                    rook = self.castle_rooks.get(&piece).unwrap()[0].unwrap();
                }
                else {
                    rook = self.castle_rooks.get(&piece).unwrap()[1].unwrap();
                }
                was_castle = true;
                let action = self.castle_moves.get(&rook).unwrap().clone().unwrap();
                self.execute_move(&mut rook, action, false, false);
            }
        }

        if was_castle {
            self.move_log.push((action.clone(), *piece, removed_piece, removed_pos, piece.has_moved, MoveType::Castle));
        }
        else {
            self.move_log.push((action.clone(), *piece, removed_piece, removed_pos, piece.has_moved, MoveType::NoCastle));
        }

        self.tiles[action.clone().end.row as usize][action.clone().end.col as usize].piece_mut().has_moved = true;
        self.tiles[action.clone().end.row as usize][action.clone().end.col as usize].piece_mut().row = action.clone().end.row as usize;
        self.tiles[action.clone().end.row as usize][action.clone().end.col as usize].piece_mut().col = action.clone().end.col as usize;
    }
    pub fn undo_last_move(&mut self) -> bool {
        if self.move_log.len() == 0 {
            return false;
        }

        let (action, mut piece, removed_piece, removed_pos, was_moved, movetype) = self.move_log.pop().unwrap();
        let mut was_castle = false;
        if movetype == MoveType::Castle {
            was_castle = true;
        };

        piece.has_moved = was_moved;
        self.tiles[action.end.row as usize][action.end.col as usize].present_piece = None;
        self.tiles[action.initial.row as usize][action.initial.col as usize].present_piece = Some(piece);

        if removed_pos.is_some() {
            self.tiles[removed_pos.unwrap().0 as usize][removed_pos.unwrap().1 as usize].present_piece = removed_piece;
        };
        if was_castle {
            self.undo_last_move();
        };
        return true;
    }
    pub fn check_promotion(&mut self, piece: Piece, end: Tile) {
        if (end.row == 0) || (end.row == 7) {
            self.tiles[end.row as usize][end.col as usize].present_piece = Some(Piece::new(PieceType::Queen, piece.color, self.cur_uid, piece.row, piece.col));
            self.cur_uid += 1;
        };
    }
    pub fn is_castling(&self, initial: Tile, end: Tile) -> bool {
        return initial.col.abs_diff(end.col) == 2;
    }
    pub fn set_en_passant(&mut self, piece: &mut Piece, action: Move) {
        for row in self.tiles.iter_mut() {
            for tile in row.iter_mut() {
                if tile.has_piece() && tile.piece().ptype == PieceType::Pawn {
                    tile.piece_mut().en_passant = false;
                }
            }
        }
        if (piece.ptype == PieceType::Pawn) && (action.clone().initial.row.abs_diff(action.clone().end.row) == 2) {
            self.tiles[action.clone().end.row as usize][action.clone().end.col as usize].piece_mut().en_passant = true;
        }
    }
    pub fn in_checkmate(&mut self, color: Team) -> bool {
        // self.calc_team_valid_moves(color);
        let mut pinned_count = 0;
        let mut count = 0;
        for row in self.tiles.iter() {
            for tile in row.iter() {
                if tile.has_team(color) {
                    match self.valid_moves.get(&tile.piece()) {
                        Some(moves) => {
                            if moves.len() == 0 {
                                pinned_count += 1;
                            }
                        },
                        None => {
                            pinned_count += 1;
                        }
                    }
                    count += 1;
                }
            };
        };
        return count == pinned_count;
    }
    pub fn get_pins_and_checks(&self, color: Team) -> (bool, Vec<(Tile, (isize, isize))>, Vec<(Tile, (isize, isize))>) {
        let mut pins = Vec::new();
        let mut checks = Vec::new();
        let mut in_check = false;
        let dirs = [(-1, 0), (0, -1), (1, 0), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)];
        let (mut start_row, mut start_col) = (0, 0);
        'outer: for (i, row) in self.tiles.iter().enumerate() {
            for (j, tile) in row.iter().enumerate() {
                if tile.has_team(color) && (tile.piece().ptype == PieceType::King) {
                    start_row = i;
                    start_col = j;
                    break 'outer;
                }
            }
        }

        for j in 0..dirs.len() {
            let mut possible_pin: Option<(Tile, (isize, isize))> = None;
            let d = dirs[j];
            for i in 1..8 {
                let (row, col) = ((start_row as isize + d.0 * i) as isize, (start_col as isize + d.1 * i) as isize);
                if inrange(row) && inrange(col) {
                    let end_tile = self.tiles[row as usize][col as usize].copy();
                    if end_tile.has_team(color) && (end_tile.piece().ptype != PieceType::King) {
                        if possible_pin.is_none() {
                            possible_pin = Some((end_tile, d));
                        }
                        else {
                            break;
                        }
                    }
                    else if end_tile.has_rival(color) {
                        if ((j <= 3) && (end_tile.piece().ptype == PieceType::Rook)) || (
                            (4 <= j) && (j <= 7) && (end_tile.piece().ptype == PieceType::Bishop)) || (
                            (i == 1) && (end_tile.piece().ptype == PieceType::Pawn) && (((color == Team::Black) && (6 <= j) && (j <= 7)) || ((color == Team::White) && (4 <= j) && (j <= 5)))) || (
                            end_tile.piece().ptype == PieceType::Queen) || ((i == 1) && (end_tile.piece().ptype == PieceType::King)) {
                                if possible_pin.is_none() {
                                    in_check = true;
                                    checks.push((end_tile, d));
                                    break;
                                }
                                else {
                                    pins.push(possible_pin.unwrap());
                                    break;
                                }
                        }
                        else {
                            break;
                        }
                    }
                }
                else {
                    break;
                }
            }
        }

        let knight_moves = [(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)];
        for m in knight_moves.iter() {
            let (row, col) = (start_row as isize + m.0, start_col as isize + m.1);
            if inrange(row) && inrange(col) {
                let end_tile = self.tiles[row as usize][col as usize].copy();
                if end_tile.has_rival(color) && (end_tile.piece().ptype == PieceType::Knight) {
                    in_check = true;
                    checks.push((end_tile, *m));
                }
            }
        }
        return (in_check, pins, checks);
    }
    pub fn is_valid(&self, piece: Piece, action: Move) -> bool {
        return self.valid_moves.contains_key(&piece) && self.valid_moves.get(&piece).unwrap().contains(&action);
    }
    pub fn add_valid_move(&mut self, piece: Piece, action: Move) {
        self.valid_moves.get_mut(&piece).unwrap().push(action);
    }
    pub fn calc_pawn_moves(&mut self, piece: Piece, row: isize, col: isize, pinned: bool, pin_direction: (isize, isize)) {
        // straight
        let steps = if piece.has_moved { 1 } else { 2 };
        let start = row + piece.dir;
        let end = row + (piece.dir * (1 + steps));
        let range: Vec<isize> = if piece.dir < 0 { (end + 1..start + 1).rev().collect() } else { (start..end).collect() };
        for valid_move_row in range {
            if !inrange(valid_move_row) || self.tiles[valid_move_row as usize][col as usize].has_piece() {
                break;
            }
            if !pinned || (pin_direction == (piece.dir, 0)) {
                let mut initial = Tile::new();
                let mut end = Tile::new();
                initial.init(row, col, None);
                end.init(valid_move_row, col, None);
                self.add_valid_move(piece, Move::new(initial, end));
            }
        }

        // diagonal
        let valid_move_row = row as isize + piece.dir;
        let valid_move_cols = [col - 1, col + 1];
        let x_shifts = [-1, 1];
        for (x_shift, move_col) in x_shifts.iter().zip(valid_move_cols.iter()) {
            if inrange(valid_move_row) && inrange(*move_col) {
                if self.tiles[valid_move_row as usize][*move_col as usize].has_rival(piece.color) && (!pinned || (pin_direction == (piece.dir, *x_shift))) {
                    let mut initial = Tile::new();
                    let mut end = Tile::new();
                    initial.init(row, col, None);
                    end.init(valid_move_row, *move_col, self.tiles[valid_move_row as usize][*move_col as usize].present_piece);
                    self.add_valid_move(piece, Move::new(initial, end));
                }
            }
        }

        // en passant
        let r = if piece.color == Team::White { 3 } else { 4 };
        let fr = if piece.color == Team::White { 2 } else { 5 };
        for direction in [1, -1].iter() {
            if inrange(col + direction) && (row == r) {
                if self.tiles[row as usize][(col as isize + direction) as usize].has_rival(piece.color) && (!pinned || (pin_direction == (piece.dir, *direction))) {
                    let p = self.tiles[row as usize][(col as isize + direction) as usize].piece();
                    if p.ptype == PieceType::Pawn {
                        if p.en_passant {
                            let mut initial = Tile::new();
                            let mut end = Tile::new();
                            initial.init(row, col, None);
                            end.init(fr, col + direction, Some(*p));
                            self.add_valid_move(piece, Move::new(initial, end));
                            break;
                        }
                    }
                }
            }
        }
    }
    pub fn calc_knight_moves(&mut self, piece: Piece, row: isize, col: isize, pinned: bool) {
        let valid_moves = [
            (row - 2, col + 1), 
            (row - 1, col + 2),
            (row + 1, col + 2),
            (row + 2, col + 1),
            (row + 2, col - 1),
            (row + 1, col - 2),
            (row - 1, col - 2),
            (row - 2, col - 1)
        ];

        for m in valid_moves.iter() {
            let (move_row, move_col) = m;
            if inrange(*move_row) && inrange(*move_col) {
                if self.tiles[*move_row as usize][*move_col as usize].is_empty_or_rival(piece.color) && !pinned {
                    let mut initial = Tile::new();
                    let mut end = Tile::new();
                    initial.init(row, col, None);
                    end.init(*move_row, *move_col, self.tiles[*move_row as usize][*move_col as usize].present_piece);
                    self.add_valid_move(piece, Move::new(initial, end));
                }
            }
        }
    }
    pub fn calc_straightline_moves(&mut self, piece: Piece, row: isize, col: isize, pinned: bool, pin_direction: (isize, isize), incrs: Vec<(isize, isize)>) {
        for inc in incrs.iter() {
            let (row_inc, col_inc) = inc;
            let mut valid_move_row = row + row_inc;
            let mut valid_move_col = col + col_inc;

            if !pinned || (pin_direction == *inc) || (pin_direction == (-inc.0, -inc.1)) {
                loop {
                    if !inrange(valid_move_row) || !inrange(valid_move_col) {
                        break;
                    }
                    let mut initial = Tile::new();
                    let mut end = Tile::new();
                    initial.init(row, col, None);
                    end.init(valid_move_row, valid_move_col, self.tiles[valid_move_row as usize][valid_move_col as usize].present_piece);
                    let action = Move::new(initial, end);

                    if self.tiles[valid_move_row as usize][valid_move_col as usize].has_team(piece.color) {
                        break;
                    }
                    else if self.tiles[valid_move_row as usize][valid_move_col as usize].has_rival(piece.color) {
                        self.add_valid_move(piece, action);
                        break;
                    }
                    else if self.tiles[valid_move_row as usize][valid_move_col as usize].is_empty() {
                        self.add_valid_move(piece, action);
                    }
                    valid_move_row += row_inc;
                    valid_move_col += col_inc;
                }
            }
        }
    }
    pub fn calc_king_moves(&mut self, piece: Piece, row: isize, col: isize) {
        let adjs: [(isize, isize); 8] = [
            (row - 1, col),
            (row - 1, col + 1),
            (row, col + 1),
            (row + 1, col + 1),
            (row + 1, col),
            (row + 1, col - 1),
            (row, col - 1),
            (row - 1, col - 1)
        ];

        for m in adjs.iter() {
            let (move_row, move_col) = m;
            if inrange(*move_row) && inrange(*move_col) {
                if self.tiles[*move_row as usize][*move_col as usize].is_empty_or_rival(piece.color) {
                    let mut initial = Tile::new();
                    let mut end = Tile::new();
                    initial.init(row, col, None);
                    end.init(*move_row, *move_col, None);
                    let action = Move::new(initial, end);

                    let temp_piece = piece.copy();
                    let mut temp_board = self.copy();
                    temp_board.simulate_move(temp_piece, action.copy());
                    let (in_check, _, _) = temp_board.get_pins_and_checks(piece.color);
                    if !in_check {
                        self.add_valid_move(piece, action);
                    }
                }
            }
        }

        let mut king_left_rook = None;
        let mut king_right_rook = None;
        if !piece.has_moved {
            let left_rook = self.tiles[row as usize][0].present_piece;
            if left_rook.is_some() && (left_rook.unwrap().ptype == PieceType::Rook) {
                if !left_rook.unwrap().has_moved {
                    for c in 1..4 {
                        if self.tiles[row as usize][c].has_piece() {
                            break;
                        }
                        if c == 3 {
                            king_left_rook = left_rook;
                            let mut initial = Tile::new();
                            let mut end = Tile::new();
                            initial.init(row, 0, None);
                            end.init(row, 3, None);
                            let rook_move = Move::new(initial, end);

                            let mut initial = Tile::new();
                            let mut end = Tile::new();
                            initial.init(row, col, None);
                            end.init(row, 2, None);
                            let king_move = Move::new(initial, end);

                            let mut temp_board = self.copy();
                            temp_board.simulate_move(piece, king_move.copy());
                            temp_board.simulate_move(piece, rook_move.copy());
                            let (in_check, _, _) = temp_board.get_pins_and_checks(piece.color);
                            if !in_check {
                                self.add_valid_move(piece, king_move);
                                self.castle_moves.insert(left_rook.unwrap(), Some(rook_move));
                            }
                        }
                    }
                }
            }
            let right_rook = self.tiles[row as usize][7].present_piece;
            if right_rook.is_some() && (right_rook.unwrap().ptype == PieceType::Rook) {
                if !right_rook.unwrap().has_moved {
                    for c in 5..7 {
                        if self.tiles[row as usize][c].has_piece() {
                            break;
                        }
                        if c == 6 {
                            king_right_rook = right_rook;
                            let mut initial = Tile::new();
                            let mut end = Tile::new();
                            initial.init(row, 7, None);
                            end.init(row, 5, None);
                            let rook_move = Move::new(initial, end);

                            let mut initial = Tile::new();
                            let mut end = Tile::new();
                            initial.init(row, col, None);
                            end.init(row, 6, None);
                            let king_move = Move::new(initial, end);

                            let mut temp_board = self.copy();
                            temp_board.simulate_move(piece, king_move.copy());
                            temp_board.simulate_move(piece, rook_move.copy());
                            let (in_check, _, _) = temp_board.get_pins_and_checks(piece.color);
                            if !in_check {
                                self.add_valid_move(piece, king_move);
                                self.castle_moves.insert(right_rook.unwrap(), Some(rook_move));
                            }
                        }
                    }
                }
            }
        }
        self.castle_rooks.insert(piece, [king_left_rook, king_right_rook]);
    }
    pub fn calc_valid_moves(&mut self, piece: Piece, row: isize, col: isize, mut pins: &mut Vec<(Tile, (isize, isize))>) -> Vec<(Tile, (isize, isize))> {
        let mut piece_pinned = false;
        let mut pin_direction = (0, 0);
        for pin in pins.iter() {
            if (pin.0.row == row) && (pin.0.col == col) {
                piece_pinned = true;
                pin_direction = pin.1;
            }
        }
        pins.retain(|pin| {
            (pin.0.row != row) || (pin.0.col != col)
        });

        self.valid_moves.insert(piece, Vec::new());
        if piece.ptype == PieceType::Pawn {
            self.calc_pawn_moves(piece, row, col, piece_pinned, pin_direction);
        } else if piece.ptype == PieceType::Knight {
            self.calc_knight_moves(piece, row, col, piece_pinned);
        } else if piece.ptype == PieceType::Bishop {
            self.calc_straightline_moves(piece, row, col, piece_pinned, pin_direction, Vec::from([
                (-1, 1),
                (-1, -1),
                (1, 1),
                (1, -1)
            ]))
        } else if piece.ptype == PieceType::Rook {
            self.calc_straightline_moves(piece, row, col, piece_pinned, pin_direction, Vec::from([
                (-1, 0),
                (1, 0),
                (0, 1),
                (0, -1)
            ]))
        } else if piece.ptype == PieceType::Queen {
            self.calc_straightline_moves(piece, row, col, piece_pinned, pin_direction, Vec::from([
                (-1, 1),
                (-1, -1),
                (1, 1),
                (1, -1),
                (-1, 0),
                (1, 0),
                (0, 1),
                (0, -1)
            ]))
        } else if piece.ptype == PieceType::King {
            self.calc_king_moves(piece, row, col);
        }
        return pins.clone();
    }
    pub fn calc_team_valid_moves(&mut self, color: Team) {
        let (in_check, mut pins, checks) = self.get_pins_and_checks(color);
        let mut king_tile = &Tile::new();
        for row in self.tiles.iter() {
            for tile in row.iter() {
                if tile.has_team(color) && (tile.piece().ptype == PieceType::King) {
                    king_tile = tile;
                }
            }
        }

        if in_check {
            if checks.len() == 1 {
                let check_tile = checks[0].0.copy();
                let check_dir = checks[0].1;
                let mut valid_moves = Vec::new();
                if check_tile.piece().ptype == PieceType::Knight {
                    valid_moves.push(check_tile);
                }
                else {
                    for i in 1..8 {
                        let r = king_tile.row as isize + check_dir.0 * i;
                        let c = king_tile.col as isize + check_dir.1 * i;
                        if inrange(r) && inrange(c) {
                            let valid_tile = self.tiles[r as usize][c as usize].copy();
                            valid_moves.push(valid_tile.copy());
                            if valid_tile == check_tile {
                                break;
                            }
                        }
                    }
                }
                for i in 0..self.tiles.len() {
                    for j in 0..self.tiles[0].len() {
                        if self.tiles[i][j].has_team(color) {
                            pins = self.calc_valid_moves(*self.tiles[i][j].piece(), i as isize, j as isize, &mut pins);
                            if self.tiles[i][j].piece().ptype != PieceType::King {
                                self.valid_moves.get_mut(&self.tiles[i][j].piece()).unwrap().retain(|action| {
                                    valid_moves.contains(&action.end)
                                });
                            }
                        }
                    }
                }
            }
            else {
                for i in 0..self.tiles.len() {
                    for j in 0..self.tiles[0].len() {
                        if self.tiles[i][j].has_team(color) && (self.tiles[i][j].piece().ptype == PieceType::King) {
                            self.calc_king_moves(*self.tiles[i][j].piece(), i as isize, j as isize);
                        }
                    }
                }
            }
        }
        else {
            for i in 0..self.tiles.len() {
                for j in 0..self.tiles[0].len() {
                    if self.tiles[i][j].has_team(color) {
                        pins = self.calc_valid_moves(*self.tiles[i][j].piece(), i as isize, j as isize, &mut pins);
                    }
                }
            }
        }
    }
    pub fn copy(&self) -> Self {
        let mut copy = Self::new();
        copy.move_log = self.move_log.clone();
        copy.game_stage = self.game_stage;
        copy.tiles = self.tiles.clone();
        copy.zobrist_keys = self.zobrist_keys.clone();
        return copy;
    }
}


fn add_pieces(tiles: &mut [[Tile; COLS]; ROWS], color: Team, start_uid: isize) -> isize {
    let (pawns, others): (usize, usize) = match color {
        Team::White => (6, 7),
        _ => (1, 0)
    };

    let mut uid = start_uid;
    for c in 0..COLS {
        tiles[pawns][c].present_piece = Some(Piece::new(PieceType::Pawn, color, uid, pawns, c));
        uid += 1;
    }

    tiles[others][1].present_piece = Some(Piece::new(PieceType::Knight, color, uid, others, 1));
    tiles[others][6].present_piece = Some(Piece::new(PieceType::Knight, color, uid + 1, others, 6));

    tiles[others][2].present_piece = Some(Piece::new(PieceType::Bishop, color, uid + 2, others, 2));
    tiles[others][5].present_piece = Some(Piece::new(PieceType::Bishop, color, uid + 3, others, 5));

    tiles[others][0].present_piece = Some(Piece::new(PieceType::Rook, color, uid + 4, others, 0));
    tiles[others][7].present_piece = Some(Piece::new(PieceType::Rook, color, uid + 5, others, 7));

    tiles[others][3].present_piece = Some(Piece::new(PieceType::Queen, color, uid + 6, others, 3));
    tiles[others][4].present_piece = Some(Piece::new(PieceType::King, color, uid + 7, others, 4));
    return uid + 8;
}


fn psqt_bonuses(tiles: &mut [[Tile; COLS]; ROWS]) {
    let pawns = [
        [0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0],
        [50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0],
        [10.0, 10.0, 20.0, 30.0, 30.0, 20.0, 10.0, 10.0],
        [5.0,  5.0, 10.0, 25.0, 25.0, 10.0,  5.0,  5.0],
        [0.0,  0.0,  0.0, 20.0, 20.0,  0.0,  0.0,  0.0],
        [5.0, -5.0,-10.0,  0.0,  0.0,-10.0, -5.0,  5.0],
        [5.0, 10.0, 10.0,-20.0,-20.0, 10.0, 10.0,  5.0],
        [0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0]
    ];
    let knights = [
        [-50.0,-40.0,-30.0,-30.0,-30.0,-30.0,-40.0,-50.0],
        [-40.0,-20.0,  0.0,  0.0,  0.0,  0.0,-20.0,-40.0],
        [-30.0,  0.0, 10.0, 15.0, 15.0, 10.0,  0.0,-30.0],
        [-30.0,  5.0, 15.0, 20.0, 20.0, 15.0,  5.0,-30.0],
        [-30.0,  0.0, 15.0, 20.0, 20.0, 15.0,  0.0,-30.0],
        [-30.0,  5.0, 10.0, 15.0, 15.0, 10.0,  5.0,-30.0],
        [-40.0,-20.0,  0.0,  5.0,  5.0,  0.0,-20.0,-40.0],
        [-50.0,-40.0,-30.0,-30.0,-30.0,-30.0,-40.0,-50.0]
    ];
    let bishops = [
        [-20.0,-10.0,-10.0,-10.0,-10.0,-10.0,-10.0,-20.0],
        [-10.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,-10.0],
        [-10.0,  0.0,  5.0, 10.0, 10.0,  5.0,  0.0,-10.0],
        [-10.0,  5.0,  5.0, 10.0, 10.0,  5.0,  5.0,-10.0],
        [-10.0,  0.0, 10.0, 10.0, 10.0, 10.0,  0.0,-10.0],
        [-10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0,-10.0],
        [-10.0,  5.0,  0.0,  0.0,  0.0,  0.0,  5.0,-10.0],
        [-20.0,-10.0,-10.0,-10.0,-10.0,-10.0,-10.0,-20.0]
    ];
    let rooks = [
        [0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0],
        [5.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0,  5.0],
        [-5.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -5.0],
        [-5.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -5.0],
        [-5.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -5.0],
        [-5.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -5.0],
        [-5.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -5.0],
        [0.0,  0.0,  0.0,  5.0,  5.0,  0.0,  0.0,  0.0]
    ];
    let queens = [
        [-20.0,-10.0,-10.0, -5.0, -5.0,-10.0,-10.0,-20.0],
        [-10.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,-10.0],
        [-10.0,  0.0,  5.0,  5.0,  5.0,  5.0,  0.0,-10.0],
        [-5.0,  0.0,  5.0,  5.0,  5.0,  5.0,  0.0, -5.0],
        [0.0,  0.0,  5.0,  5.0,  5.0,  5.0,  0.0, -5.0],
        [-10.0,  5.0,  5.0,  5.0,  5.0,  5.0,  0.0,-10.0],
        [-10.0,  0.0,  5.0,  0.0,  0.0,  0.0,  0.0,-10.0],
        [-20.0,-10.0,-10.0, -5.0, -5.0,-10.0,-10.0,-20.0]
    ];
    let kings = [
        [-30.0,-40.0,-40.0,-50.0,-50.0,-40.0,-40.0,-30.0],
        [-30.0,-40.0,-40.0,-50.0,-50.0,-40.0,-40.0,-30.0],
        [-30.0,-40.0,-40.0,-50.0,-50.0,-40.0,-40.0,-30.0],
        [-30.0,-40.0,-40.0,-50.0,-50.0,-40.0,-40.0,-30.0],
        [-20.0,-30.0,-30.0,-40.0,-40.0,-30.0,-30.0,-20.0],
        [-10.0,-20.0,-20.0,-20.0,-20.0,-20.0,-20.0,-10.0],
        [20.0, 20.0,  0.0,  0.0,  0.0,  0.0, 20.0, 20.0],
        [20.0, 30.0, 10.0,  0.0,  0.0, 10.0, 30.0, 20.0]
    ];
    for (i, row) in izip!(pawns, knights, bishops, rooks, queens, kings).enumerate() {
        for (j, (p, n, b, r, q, k)) in izip!(row.0, row.1, row.2, row.3, row.4, row.5).enumerate() {
            tiles[i][j].bonuses.insert((PieceType::Pawn, Team::White), p);
            tiles[i][j].bonuses.insert((PieceType::Knight, Team::White), n);
            tiles[i][j].bonuses.insert((PieceType::Bishop, Team::White), b);
            tiles[i][j].bonuses.insert((PieceType::Rook, Team::White), r);
            tiles[i][j].bonuses.insert((PieceType::Queen, Team::White), q);
            tiles[i][j].bonuses.insert((PieceType::King, Team::White), k);

            tiles[(i as i32 * -1 + (ROWS as i32 - 1)) as usize][j].bonuses.insert((PieceType::Pawn, Team::Black), p);
            tiles[(i as i32 * -1 + (ROWS as i32 - 1)) as usize][j].bonuses.insert((PieceType::Knight, Team::Black), n);
            tiles[(i as i32 * -1 + (ROWS as i32 - 1)) as usize][j].bonuses.insert((PieceType::Bishop, Team::Black), b);
            tiles[(i as i32 * -1 + (ROWS as i32 - 1)) as usize][j].bonuses.insert((PieceType::Rook, Team::Black), r);
            tiles[(i as i32 * -1 + (ROWS as i32 - 1)) as usize][j].bonuses.insert((PieceType::Queen, Team::Black), q);
            tiles[(i as i32 * -1 + (ROWS as i32 - 1)) as usize][j].bonuses.insert((PieceType::King, Team::Black), k);
        }
    }
}


fn create_zobrist_keys() -> [[u64; 12]; (ROWS * COLS)] {
    let mut zobrist = [[0u64; 12]; (ROWS * COLS)];
    if File::open(Constants::new().zobrist_file).is_err() {
        let mut rng = thread_rng();
        for i in 0..(ROWS * COLS) {
            for j in 0..12 {
                zobrist[i][j] = rng.gen_range(0..u64::MAX);
            }
        }

        let file = File::create(Constants::new().zobrist_file).unwrap();
        serialize_into(file, &zobrist.to_vec()).unwrap();
    }
    else {
        let file = File::open(Constants::new().zobrist_file).unwrap();
        let as_vec: Vec<[u64; 12]> = deserialize_from(file).unwrap();
        for i in 0..(ROWS * COLS) {
            for j in 0..12 {
                zobrist[i][j] = as_vec[i][j];
            }
        }
    }
    return zobrist;
}