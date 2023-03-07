use macroquad::prelude::*;
#[path = "constants.rs"] mod constants;
use constants::BG_COLOR;


struct Button {
    font: Font,
    rect: (f32, f32, f32, f32),
    text: String,
}

impl Button {
    fn new(pos: (f32, f32), size: (f32, f32), text: String, font: Font) -> Self {
        Self {
            font: font,
            rect: (pos.0, pos.1, size.0, size.1),
            text: text,
        }
    }
    fn clicked(&mut self, mousepos: (f32, f32)) -> bool {
        let (x, y) = mousepos;
        if (x < self.rect.0 + self.rect.2) && (y < self.rect.1 + self.rect.3) && (x > self.rect.0) && (y > self.rect.1) {
            return true;
        }
        return false;
    }
    fn draw(&self) {
        draw_rectangle(self.rect.0, self.rect.1, self.rect.2, self.rect.3, Color::from_rgba(200, 200, 200, 255));
        let dims = measure_text(&self.text, Some(self.font), 96u16, 1.0);
        draw_text_ex(
            &self.text,
            self.rect.2 / 2.0 - dims.width / 2.0 + self.rect.0,
            self.rect.3 / 2.0 + dims.height / 4.0 + self.rect.1,
            TextParams{font: self.font, font_size: 96u16, color: Color::from_rgba(150, 150, 150, 255), ..Default::default()}
        )
    }
}



pub struct MainMenu {
    pub active: bool,
    pub should_use_ai: bool,
    font: Font,
    buttons: Vec<Button>
}

impl MainMenu {
    pub async fn new() -> Self {
        Self {
            active: false,
            should_use_ai: false,
            font: load_ttf_font("assets/Monaco.ttf").await.unwrap(),
            buttons: Vec::new()
        }
    }
    pub fn show_load(&self) {
        clear_background(BG_COLOR);
        let txt = "Loading Assets...";
        let dims = measure_text(txt, Some(self.font), 80u16, 1.0);
        draw_text_ex(
            txt,
            screen_width() / 2.0 - dims.width / 2.0,
            screen_height() / 2.0 - dims.height / 2.0,
            TextParams{font: self.font, font_size: 80u16, color: WHITE, ..Default::default()}
        )
    }
    pub async fn show(&mut self) {
        let button_width = 700.0;
        self.buttons = Vec::from([
            Button::new((screen_width() / 2.0 - button_width / 2.0, screen_height() / 2.0 - 100.0), (button_width, 150.0), "Fight AI".to_string(), self.font),
            Button::new((screen_width() / 2.0 - button_width / 2.0, screen_height() / 2.0 + 100.0), (button_width, 150.0), "Begin PVP".to_string(), self.font),
        ]);
        self.active = true;
        while self.active {
            if is_mouse_button_pressed(MouseButton::Left) {
                self.on_click();
            }
            clear_background(BG_COLOR);
            self.draw();
            next_frame().await;
        }
    }
    pub fn start_pvp(&mut self) {
        self.should_use_ai = false;
        self.active = false;
    }
    pub fn start_ai(&mut self) {
        self.should_use_ai = true;
        self.active = false;
    }
    pub fn on_click(&mut self) {
        if self.active {
            for (i, button) in self.buttons.iter_mut().enumerate() {
                let was_clicked = button.clicked(mouse_position());
                if was_clicked {
                    match i {
                        0 => {
                            self.should_use_ai = true;
                            self.active = false;
                        },
                        _ => {
                            self.should_use_ai = false;
                            self.active = false;
                        }
                    }
                }
            }
        }
    }
    pub fn draw(&self) {
        if self.active {
            let text = "Chess";
            let dims = measure_text(text, Some(self.font), 100u16, 1.0);
            draw_text_ex(
                text,
                screen_width() / 2.0 - dims.width / 2.0,
                screen_height() / 2.0 - 200.0,
                TextParams{font: self.font, font_size: 100u16, color: WHITE, ..Default::default()}
            );
            for button in self.buttons.iter() {
                button.draw();
            }
        }
    }
}