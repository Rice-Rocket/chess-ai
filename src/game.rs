use macroquad::prelude::*;
use macroquad::audio::*;
#[path = "config.rs"] mod config;
use config::Config;
#[path = "algo.rs"] mod algo;
pub use algo::*;


pub struct Game {
    pub next_player: Team,
    pub hovered_tile: (isize, isize),
    pub board: Board,
    pub algorithms: Algorithms,
    pub dragger: Dragger,
    pub config: Config,
    pub winner: Team,
    pub game_over: bool,
    pub win_screen_timer: Timer,
    pub use_ai: bool,
}

impl Game {
    pub async fn new() -> Self {
        Self {
            next_player: Team::White,
            hovered_tile: (-1, -1),
            board: Board::new(),
            algorithms: Algorithms::new(Team::Black),
            dragger: Dragger::new(),
            config: Config::new().await,
            winner: Team::None,
            game_over: false,
            win_screen_timer: Timer::new(6000),
            use_ai: false,
        }
    }
    pub fn init(&mut self) {
        self.board.calc_team_valid_moves(self.next_player);
    }
    pub async fn compute_move(&mut self, tilesize: f32) {
        if self.winner == Team::None {
            self.algorithms.evaluated_states = 0;
            self.algorithms.pruned_states = 0;
            self.algorithms.transpositions = 0;
            self.draw(tilesize).await;
            next_frame().await;

            let (evaluation, (piece, action)) = self.algorithms.search_multi(self.board.copy(), 4);
            // if self.algorithms.nth_move > 10 {
            //     (evaluation, (piece, action)) = self.algorithms.alphabetatrans(self.board, 4);
            // }
            // else {
            //     (evaluation, (piece, action)) = self.algorithms.choose_from_book(self.board);
            // }
            if piece.is_some() {
                self.execute_move(&mut piece.unwrap(), action.unwrap());
                self.algorithms.nth_move += 1;
            }
        }
    }
    pub fn execute_move(&mut self, mut piece: &mut Piece, action: Move) {
        if piece.ptype != PieceType::None {
            let captured = self.board.tiles[action.end.row as usize][action.end.col as usize].has_piece();
            self.board.execute_move(piece, action.copy(), false, false);

            // if self.board.in_checkmate(self.algorithms.opponent) {
            //     self.winner = self.algorithms.perspective;
            // }
            self.board.set_en_passant(&mut piece, action.copy());
            self.playsound(captured);
            self.next_turn();
        }
    }
    pub fn undo_move(&mut self) {
        let success = self.board.undo_last_move();
        if success {
            self.next_turn();
        }
    }
    pub fn next_turn(&mut self) {
        self.next_player = self.next_player.other();
        if self.next_player == self.algorithms.opponent {
            self.board.calc_team_valid_moves(self.algorithms.opponent);
            if self.board.in_checkmate(self.algorithms.opponent) {
                self.winner = self.algorithms.perspective;
            }
        }
        if !self.use_ai && (self.next_player == self.algorithms.perspective) {
            self.board.calc_team_valid_moves(self.next_player);
            if self.board.in_checkmate(self.next_player) {
                self.winner = self.next_player.other();
            }
        }
    }
    pub fn set_hover(&mut self, row: isize, col: isize) {
        if inrange(row) && inrange(col) {
            self.hovered_tile = (row, col);
        }
    }
    pub fn playsound(&mut self, capture: bool) {
        if capture {
            play_sound_once(self.config.capture_sound);
        }
        else {
            play_sound_once(self.config.move_sound);
        }
    }
    pub async fn reset(&mut self) {
        *self = Self::new().await;
        self.init();
    }
    pub fn render_bg(&mut self, tilesize: f32) {
        for row in 0..ROWS {
            for col in 0..COLS {
                let color;
                if ((row + col) % 2) == 0 {
                    color = self.config.theme.bg.light;
                }
                else {
                    color = self.config.theme.bg.dark;
                }
                draw_rectangle(col as f32 * tilesize, row as f32 * tilesize, tilesize, tilesize, color);
                if col == 0 {
                    let text_color;
                    if (row % 2) == 0 {
                        text_color = self.config.theme.bg.dark;
                    }
                    else {
                        text_color = self.config.theme.bg.light;
                    }
                    draw_text_ex(
                        &format!("{}", ROWS - row),
                        5.0,
                        20.0 + row as f32 * tilesize,
                        TextParams{font: self.config.font, font_size: 18u16, color: text_color, ..Default::default()}
                    )
                }
                if row == 7 {
                    let text_color;
                    if ((row + col) % 2) == 0 {
                        text_color = self.config.theme.bg.dark;
                    }
                    else {
                        text_color = self.config.theme.bg.light;
                    }
                    let txt = match col {
                        0 => "a",
                        1 => "b",
                        2 => "c",
                        3 => "d",
                        4 => "e",
                        5 => "f",
                        6 => "g",
                        _ => "h"
                    };
                    draw_text_ex(
                        &format!("{}", txt),
                        col as f32 * tilesize + tilesize - 20.0,
                        screen_height() - 10.0,
                        TextParams{font: self.config.font, font_size: 18u16, color: text_color, ..Default::default()}
                    )
                }
            }
        }
    }
    pub async fn render_pieces(&mut self, tilesize: f32) {
        for row in 0..ROWS {
            for col in 0..COLS {
                if self.board.tiles[row][col].has_piece() {
                    let piece = self.board.tiles[row][col].present_piece;
                    if self.dragger.piece.is_none() || (piece.unwrap() != *self.dragger.piece.as_ref().unwrap()) {
                        let im = load_texture(&piece.unwrap().texture_path(80)).await.unwrap();
                        let pos = (col as f32 * tilesize, row as f32 * tilesize);
                        draw_texture(im, pos.0 + tilesize / 2.0 - im.width() / 2.0, pos.1 + tilesize / 2.0 - im.height() / 2.0, Color::new(1.0, 1.0, 1.0, 1.0));
                    }
                }
            }
        }
    }
    pub fn render_valid_moves(&mut self, tilesize: f32) {
        if self.dragger.dragging {
            if !self.board.valid_moves.contains_key(self.dragger.piece.as_ref().unwrap()) {
                return;
            }
            for action in self.board.valid_moves.get(self.dragger.piece.as_ref().unwrap()).unwrap() {
                let color;
                if ((action.end.row) + (action.end.col) % 2) == 0 {
                    color = self.config.theme.moves.light;
                }
                else {
                    color = self.config.theme.moves.dark;
                }
                draw_rectangle(action.end.col as f32 * tilesize, action.end.row as f32 * tilesize, tilesize, tilesize, color);
            }
        }
    }
    pub fn render_last_move(&mut self, tilesize: f32) {
        if self.board.move_log.len() > 0 {
            let initial = self.board.move_log[self.board.move_log.len() - 1].0.copy().initial;
            let end = self.board.move_log[self.board.move_log.len() - 1].0.copy().end;
            for pos in [initial, end].iter() {
                let color;
                if ((pos.row + pos.col) % 2) == 0 {
                    color = self.config.theme.trace.light;
                }
                else {
                    color = self.config.theme.trace.dark;
                }
                draw_rectangle(pos.col as f32 * tilesize, pos.row as f32 * tilesize, tilesize, tilesize, color);
            }
        }
    }
    pub fn render_hover(&mut self, tilesize: f32) {
        if self.hovered_tile != (-1, -1) {
            draw_rectangle_lines(self.hovered_tile.1 as f32 * tilesize, self.hovered_tile.0 as f32 * tilesize, tilesize, tilesize, 4.0, Color::from_rgba(180, 180, 180, 255))
        }
    }
    pub fn render_win_screen(&mut self) {
        let alpha = if self.win_screen_timer.runtime > 800 {
            ((self.win_screen_timer.runtime as f32 / 1200.0) * 255.0) as u8
        }
        else {
            170
        };
        let smaller_dim = screen_width().min(screen_height());
        let x_shift = screen_width() - smaller_dim;
        let y_shift = screen_height() - smaller_dim;
        draw_rectangle(0.0, 0.0, screen_width() - x_shift, screen_height() - y_shift, Color::from_rgba(0, 0, 0, alpha));

        let checkmate_text = "Checkmate";
        let winner_text = format!("{} Wins", self.winner.as_string());
        let checkmate_dims = measure_text(checkmate_text, Some(self.config.font), 75u16, 1.0);
        let dims = measure_text(&winner_text, Some(self.config.font), 75u16, 1.0);
        let text_pos = if self.win_screen_timer.runtime < 800 {
            ((screen_width() - x_shift) / 2.0 - dims.width / 2.0, (screen_height() - y_shift) / 2.0 - dims.height / 2.0 - (54.0 - (self.win_screen_timer.runtime / 15) as f32))
        }
        else {
            ((screen_width() - x_shift) / 2.0 - dims.width / 2.0, (screen_height() - y_shift) / 2.0 - dims.height / 2.0)
        };
        draw_text_ex(
            &winner_text,
            text_pos.0,
            text_pos.1 + dims.height - 20.0,
            TextParams{font: self.config.font, font_size: 75u16, color: self.config.theme.title_color, ..Default::default()}
        );
        draw_text_ex(
            checkmate_text,
            (screen_width() - x_shift) / 2.0 - checkmate_dims.width / 2.0,
            text_pos.1 - checkmate_dims.height - 20.0,
            TextParams{font: self.config.font, font_size: 75u16, color: self.config.theme.title_color, ..Default::default()}
        );
    }
    pub async fn draw(&mut self, tilesize: f32) {
        clear_background(Color::from_rgba(33, 32, 33, 255));
        self.render_bg(tilesize);
        self.render_last_move(tilesize);
        self.render_valid_moves(tilesize);
        self.render_hover(tilesize);
        self.render_pieces(tilesize).await;
        if self.winner != Team::None {
            if !self.win_screen_timer.active && !self.win_screen_timer.finished {
                self.win_screen_timer.activate();
            }
            self.win_screen_timer.update();
            self.render_win_screen();
        }
    }
}