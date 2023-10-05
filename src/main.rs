use std::{env, thread};
use std::io::stdin;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::time::Duration;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::{ContextBuilder, event};
use crate::view::board_view::BoardView;
use crate::view::MainState;
use local_ip_address::local_ip;
use crate::client::ClientGame;
use crate::server::{ProtocolState, ServerGame};

mod view;
mod bridge;
mod erikfran_chess_impl;
mod json_tcp_stream;
mod server;
mod client;

const PORT: u16 = 8384;

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

    let (mut ctx, event_loop) = builder.build().expect("Failed to start ggez.");


    println!("Do you want to be a server or client? (server, client)");
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();

    match &buf.trim()[..] {
        "server" => {
            let my_local_ip = local_ip().unwrap();
            println!("I am the server. Please tell people to join the ip: {}", my_local_ip);

            let game = erikfran_chess::Game::new();
            let mut server_game = ServerGame::new(game, 8384);

            loop {
                let state = server_game.get_protocol_state();

                match state {
                    ProtocolState::NotConnected => {
                        println!("Waiting for client to connect...");
                        server_game.try_accept_client();
                    },
                    ProtocolState::Handshake => {
                        println!("Waiting for handshake.");
                        server_game.try_handshake();
                    },
                    ProtocolState::Play => {
                        break;
                    }
                }

                thread::sleep(Duration::from_millis(1000));
            }

            // Ready to play.

            println!("Ready to play!");

            let main_state = MainState::new();

            let board_view = BoardView::new(&mut ctx, server_game).unwrap();

            main_state.set_view(board_view);

            event::run(ctx, event_loop, main_state);
        }
        "client" => {
            println!("Enter the ip to join");
            let mut buf2 = String::new();
            stdin().read_line(&mut buf2).unwrap();
            let ip: Ipv4Addr = buf2.trim().parse().unwrap();
            println!("Attempting to connect to {}", ip);

            let client_game = ClientGame::connect(ip, PORT);

            let main_state = MainState::new();

            let board_view = BoardView::new(&mut ctx, client_game).unwrap();

            main_state.set_view(board_view);

            event::run(ctx, event_loop, main_state);
        }
        _ => {
            panic!("Invalid option!");
        }
    }
}