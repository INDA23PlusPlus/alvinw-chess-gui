use std::env;
use std::path::PathBuf;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::{ContextBuilder, event};
use crate::view::board_view::BoardView;
use crate::view::main_menu::MainMenu;
use crate::view::{MainState, View};

mod view;
mod bridge;
mod erikfran_chess_impl;
mod server;

fn main() {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        PathBuf::from("./resources")
    };

    let builder = ContextBuilder::new("alvinw-chess-gui", "alvinw")
        .window_setup(WindowSetup::default().title("alvinw-chess-gui"))
        .window_mode(WindowMode::default()
            .resizable(true)
        )
        .add_resource_path(resource_dir);

    let (ctx, event_loop) = builder.build().expect("Failed to start ggez.");

    let mut main_state = MainState::new();

    let main_menu = MainMenu::new(&mut main_state);

    main_state.set_view(main_menu);

    event::run(ctx, event_loop, main_state);
}