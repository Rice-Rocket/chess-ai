use std::collections::HashMap;

#[path = "dragger.rs"] mod dragger;
pub use dragger::*;
use num_cpus;
use rayon::prelude::*;
use std::sync::Mutex;


pub struct Algorithms {
    pub perspective: Team,
    pub opponent: Team,

    // visited_states: Vec<>,
    // hash_table: HashMap<u64>,

    pub evaluated_states: isize,
    pub pruned_states: isize,
    pub transpositions: isize,
    pub nth_move: isize
}

impl Algorithms {
    pub fn new(color: Team) -> Self {
        Self {
            perspective: color,
            opponent: color.other(),

            // visited_states: Vec::new(),
            // hash_table: HashMap::new(),

            evaluated_states: 0,
            pruned_states: 0,
            transpositions: 0,
            nth_move: 0
        }
    }
    pub fn get_valid_moves(&self, state: &mut Board, color: Team) -> Vec<(Piece, Move)> {
        let mut valid_moves = Vec::new();
        state.calc_team_valid_moves(color);
        for (piece, moves) in state.valid_moves.iter() {
            if piece.color == color {
                for m in moves.iter() {
                    valid_moves.push((*piece, m.copy()));
                }
            }
        }
        return valid_moves;
    }
    pub fn get_ordered_valid_moves(&self, mut state: &mut Board, color: Team) -> Vec<(Piece, Move)> {
        let mut valid_moves = self.get_valid_moves(&mut state, color);
        let mut scores = Vec::new();
        for m in valid_moves.iter() {
            let mut move_score_guess = 0;
            let capture_piece = state.tiles[m.1.end.row as usize][m.1.end.col as usize].present_piece;
            if capture_piece.is_some() {
                move_score_guess += 10 * capture_piece.unwrap().value_mg - m.0.value_mg;
            }
            if (m.0.ptype == PieceType::Pawn) && ((m.1.end.row == 0) || (m.1.end.row == 7)) {
                move_score_guess += PieceType::Queen.value_mg();
            }
            'outer: for p in state.valid_moves.keys() {
                if p.color != color {
                    for action in state.valid_moves.get(p).unwrap().iter() {
                        if m.1.end == action.end {
                            move_score_guess -= (m.0.value_mg - p.value_mg).max(100);
                            break 'outer;
                        }
                    }
                }
            }
            scores.push(move_score_guess);
        }
        let valid_moves_cpy = valid_moves.clone();
        valid_moves.sort_by_key(|m| scores[valid_moves_cpy.iter().position(|i| i == m).unwrap()]);
        return valid_moves;
    }


    pub fn alphabeta(&self, mut state: Board, depth: isize, perspective: f32, mut alpha: f32, beta: f32) -> (f32, (Option<Piece>, Option<Move>)) {
        state.calc_team_valid_moves(Team::White);
        state.calc_team_valid_moves(Team::Black);
        if (depth == 0) || state.is_terminal() {
            // self.evaluated_states += 1;
            return (perspective * state.evaluate(self.perspective), (None, None));
        }

        let color = if perspective == 1.0 { self.perspective } else { self.opponent };
        let mut best_eval = f32::MIN;
        let mut best_move = (None, None);
        for m in self.get_ordered_valid_moves(&mut state, color) {
            let mut temp_board = state.copy();
            temp_board.execute_move(&mut m.0.copy(), m.1.copy(), false, false);
            let evaluation = -self.alphabeta(temp_board, depth - 1, -perspective, -beta, -alpha).0;
            best_eval = best_eval.max(evaluation);
            if best_eval == evaluation {
                best_move = (Some(m.0), Some(m.1.clone()));
            }
            alpha = alpha.max(best_eval);
            if alpha >= beta {
                // self.pruned_states += 1;
                break;
            }
        }
        // self.evaluated_states += 1;
        return (best_eval, best_move);
    }
    pub fn search_multi_worker(&self, children: std::iter::StepBy<std::slice::Iter<'_, (Piece, Move)>>, state: Board, depth: isize, perspective: f32, mut alpha: f32, beta: f32) -> (f32, (Option<Piece>, Option<Move>)) {
        let mut best_eval = f32::MIN;
        let mut best_move = (None, None);
        for m in children {
            let mut temp_board = state.copy();
            temp_board.execute_move(&mut m.0.copy(), m.1.copy(), false, false);
            let evaluation = -self.alphabeta(temp_board, depth - 1, -perspective, -beta, -alpha).0;
            best_eval = best_eval.max(evaluation);
            if best_eval == evaluation {
                best_move = (Some(m.0), Some(m.1.clone()));
            }
            alpha = alpha.max(best_eval);
            if alpha >= beta {
                // self.pruned_states += 1;
                break;
            }
        }
        return (best_eval, best_move);
    }
    pub fn search_multi(&mut self, mut state: Board, depth: isize) -> (f32, (Option<Piece>, Option<Move>)) {
        let child_nodes = self.get_ordered_valid_moves(&mut state, self.perspective);
        let n_threads = num_cpus::get();
        let results = Mutex::new(Vec::new());
        (0..n_threads).into_par_iter().for_each(|i| {
            let result = self.search_multi_worker(child_nodes[i..child_nodes.len()].iter().step_by(n_threads), state.copy(), depth - 1, 1.0, f32::MIN, f32::MAX);
            results.lock().unwrap().push(result);
        });
        let res = results.lock().unwrap().clone();
        let best = res.iter().min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
        return best.clone();
    }
}