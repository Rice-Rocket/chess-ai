use macroquad::prelude::*;
#[path = "game.rs"] mod game;
use game::*;
#[path = "menu.rs"] mod menu;
use menu::MainMenu;


async fn check_events(game: &mut Game, tilesize: f32) {
    if is_mouse_button_pressed(MouseButton::Left) && (!game.use_ai || (game.next_player != game.algorithms.perspective)) {
        let mousepos = mouse_position();
        game.dragger.update_mouse(mousepos);
        let clicked_col = (game.dragger.mousex / tilesize) as isize;
        let clicked_row = (game.dragger.mousey / tilesize) as isize;
        if inrange(clicked_row) && inrange(clicked_col) && game.board.tiles[clicked_row as usize][clicked_col as usize].has_piece() {
            if game.board.tiles[clicked_row as usize][clicked_col as usize].piece().color == game.next_player {
                game.dragger.save_initial(mousepos, tilesize);
                game.dragger.begin_drag(game.board.tiles[clicked_row as usize][clicked_col as usize].piece());
            }
        }
    }
    if is_mouse_button_released(MouseButton::Left) && (!game.use_ai || (game.next_player != game.algorithms.perspective)) {
        let mousepos = mouse_position();
        if game.dragger.dragging {
            game.dragger.update_mouse(mousepos);
            let released_col = (game.dragger.mousex / tilesize) as isize;
            let released_row = (game.dragger.mousey / tilesize) as isize;

            let mut initial = Tile::new();
            let mut final_ = Tile::new();
            initial.init(game.dragger.init_row, game.dragger.init_col, None);
            final_.init(released_row, released_col, None);
            let action = Move::new(initial, final_);

            if game.board.is_valid(game.dragger.piece.unwrap(), action.clone()) {
                let captured = game.board.tiles[released_row as usize][released_col as usize].has_piece();
                game.board.execute_move(&mut game.dragger.piece.unwrap(), action.clone(), false, false);
                // for i in 0..ROWS {
                //     for j in 0..COLS {
                //         if game.board.tiles[i][j].has_piece() && (*game.board.tiles[i][j].piece() == game.dragger.piece.unwrap()) {
                //             game.board.tiles[i][j].piece().has_moved = true;
                //         }
                //     }
                // }
                game.board.set_en_passant(&mut game.dragger.piece.unwrap(), action);
                game.playsound(captured);
                game.draw(tilesize).await;
                game.next_turn();
            }
        }
        game.dragger.end_drag();
    }
    if is_key_pressed(KeyCode::T) {
        game.config.change_theme();
    }
    if is_key_pressed(KeyCode::R) {
        game.reset().await;
    }
    if is_key_pressed(KeyCode::Z) && !game.use_ai {
        game.undo_move();
    }
    let mousepos = mouse_position();
    game.set_hover((mousepos.1 / tilesize) as isize, (mousepos.0 / tilesize) as isize);
    if !game.use_ai || (game.next_player != game.algorithms.perspective) {
        if game.dragger.dragging {
            game.dragger.update_mouse(mousepos);
            game.dragger.draw().await;
        }
    }
}

async fn update_moves(game: &mut Game, tilesize: f32) {
    if (game.next_player == game.algorithms.perspective) && game.use_ai {
        game.compute_move(tilesize).await;
    }
}

async fn draw(menu: &mut MainMenu, game: &mut Game, tilesize: f32) {
    clear_background(Color::from_rgba(33, 32, 33, 255));
    menu.draw();
    game.draw(tilesize).await;
    if game.dragger.dragging {
        game.dragger.draw().await;
    }
}





#[macroquad::main("Chess")]
async fn main() {
    request_new_screen_size(640., 400.);
    let mut menu = MainMenu::new().await;
    menu.show_load();
    let mut game = Game::new().await;
    menu.show().await;
    game.use_ai = menu.should_use_ai;
    game.init();
    let mut tilesize: f32;

    loop {
        tilesize = screen_width().min(screen_height()) / 8.0;
        check_events(&mut game, tilesize).await;
        update_moves(&mut game, tilesize).await;
        draw(&mut menu, &mut game, tilesize).await;
        next_frame().await
    }
}
