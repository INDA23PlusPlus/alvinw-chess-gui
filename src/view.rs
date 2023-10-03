use ggez::event::{EventHandler, MouseButton};
use ggez::{Context, GameError, GameResult};

pub mod main_menu;
pub mod board_view;

pub struct MainState{
    current_view: Box<dyn View>,
}

pub trait View {
    fn update(&mut self, ctx: &mut Context) -> GameResult;
    fn draw(&mut self, ctx: &mut Context) -> GameResult;
    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) -> GameResult {
        Ok(())
    }
    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32, _dx: f32, _dy: f32) -> GameResult {
        Ok(())
    }
}

struct NoopView {}
impl View for NoopView {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
}

impl MainState {
    pub fn new() -> Self {
        Self { current_view: Box::new(NoopView {}) }
    }

    pub fn set_view(&mut self, view: impl View + 'static) {
        self.current_view = Box::new(view);
    }
}

impl EventHandler<GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.current_view.update(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.current_view.draw(ctx)
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) -> GameResult {
        self.current_view.mouse_button_down_event(ctx, button, x, y)
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) -> Result<(), GameError> {
        self.current_view.mouse_motion_event(ctx, x, y, dx, dy)
    }
}