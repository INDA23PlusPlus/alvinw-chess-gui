use ggez::{Context, GameResult};
use ggez::event::MouseButton;
use ggez::glam::Vec2;
use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, FillOptions, Mesh, MeshBuilder, Rect, Text};
use crate::view::board_view::BoardView;
use crate::view::{MainState, View};

struct Button {
    x: u32,
    y: u32,
    width: f32,
    height: f32,
    text: String,
    hover: bool,
}

impl Button {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) {

        let primary_color = if self.hover { Color::BLACK } else { Color::WHITE };
        let secondary_color = if self.hover { Color::WHITE } else { Color::BLACK };

        let mut text = Text::new(&self.text);
        text.set_scale(32.0);
        let text_size = text.measure(ctx).unwrap();

        self.width = text_size.x + 10.0;
        self.height = text_size.y + 10.0;

        let mut rect = MeshBuilder::new();
        rect.rectangle(
            DrawMode::Fill(FillOptions::default()),
            Rect::new(0.0, 0.0, self.width, self.height),
            secondary_color
        ).unwrap();
        let rect = Mesh::from_data(ctx, rect.build());
        canvas.draw(&rect, Vec2::new(self.x as f32, self.y as f32));

        let param = DrawParam::new()
            .dest(Vec2::new(self.x as f32 + 5.0, self.y as f32 + 5.0))
            .color(primary_color);
        canvas.draw(&text, param);
    }

    fn is_inside(&self, x: f32, y: f32) -> bool {
        x > self.x as f32 && y > self.y as f32
            && x < self.x as f32 + self.width && y < self.y as f32 + self.height
    }

    fn handle_mouse_move(&mut self, x: f32, y: f32) {
        self.hover = self.is_inside(x, y)
    }
}

pub struct MainMenu<'a> {
    main_state: &'a mut MainState,
    frames: usize,
    single_player_button: Button,
    host_button: Button,
}

impl<'a> MainMenu<'a> {
    pub fn new(main_state: &'a mut MainState) -> Self {
        Self {
            main_state,
            frames: 0,
            single_player_button: Button {
                x: 300,
                y: 250,
                text: String::from("Single player"),
                hover: false,
                width: 0.0, height: 0.0,
            },
            host_button: Button {
                x: 250,
                y: 350,
                text: String::from("Host multiplayer game"),
                hover: false,
                width: 0.0, height: 0.0,
            }
        }
    }
}

impl<'a> View for MainMenu<'a> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));

        let mut text = Text::new("Hello, welcome to Chess game, yes");
        text.set_scale(40.0);

        canvas.draw(&text, Vec2::new(30.0, 30.0));

        self.single_player_button.draw(ctx, &mut canvas);
        self.host_button.draw(ctx, &mut canvas);

        canvas.finish(ctx)?;

        self.frames += 1;

        Ok(())
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, _button: MouseButton, x: f32, y: f32) -> GameResult {

        if self.single_player_button.is_inside(x, y) {
            // Start single player game
            let game = erikfran_chess::Game::new();
            let view = BoardView::new(ctx, game).unwrap();

            self.main_state.set_view(view);
        }

        Ok(())
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) -> GameResult {

        self.single_player_button.handle_mouse_move(x, y);
        self.host_button.handle_mouse_move(y, y);

        Ok(())
    }
}