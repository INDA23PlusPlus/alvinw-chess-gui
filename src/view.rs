use std::cell::RefCell;
use ggez::event::{EventHandler, MouseButton};
use ggez::{Context, GameError, GameResult};

pub mod main_menu;
pub mod board_view;

pub struct MainState{
    current_view: RefCell<Box<dyn View>>,
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
        Self { current_view: RefCell::new(Box::new(NoopView {})) }
    }

    pub fn set_view(&self, view: impl View + 'static) {
        let mut v = self.current_view.borrow_mut();
        *v = Box::new(view);
    }
}

impl EventHandler<GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.current_view.borrow_mut().update(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.current_view.borrow_mut().draw(ctx)
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) -> GameResult {
        self.current_view.borrow_mut().mouse_button_down_event(ctx, button, x, y)
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) -> Result<(), GameError> {
        self.current_view.borrow_mut().mouse_motion_event(ctx, x, y, dx, dy)
    }
}