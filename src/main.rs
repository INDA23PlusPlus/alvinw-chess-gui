use std::{path::PathBuf, env};

use chess::{Game, util::{Square, File, Rank}};
use ggez::{ContextBuilder, event, Context, GameResult, graphics::{self, Image, MeshBuilder, FillOptions, Rect, Color, MeshData, Mesh}, conf::WindowSetup};

const SQUARE_SIZE: f32 = 64.0;

struct MainState {
    frames: usize,
    board: Mesh,
    black_king_image: Image,
    black_queen_image: Image,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let black_king_image = Image::from_path(ctx, "/black_king.png")?;
        let black_queen_image = Image::from_path(ctx, "/black_king.png")?;

        let BLACK_SQUARE_COLOR: Color = Color::from_rgb(120, 100, 80);
        let WHITE_SQUARE_COLOR: Color = Color::from_rgb(230, 200, 30);

        let mut board_builder = MeshBuilder::new();
        for i in 0..8 {
            for j in 0..8 {
                let color = if (i + j) % 2 == 0 { WHITE_SQUARE_COLOR } else { BLACK_SQUARE_COLOR };
                board_builder.rectangle(
                    graphics::DrawMode::Fill(FillOptions::default()),
                    Rect::new(i as f32 * SQUARE_SIZE, j as f32 * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE),
                    color
                ).expect("Failed to draw rectangle.");
            }
        }
        let board = Mesh::from_data(ctx, board_builder.build());

        Ok(MainState {
            frames: 0,
            black_king_image,
            black_queen_image,
            board,
        })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        // Text is drawn from the top-left corner.
        let offset = self.frames as f32 / 10.0;
        let dest_point = ggez::glam::Vec2::new(offset, offset);
        canvas.draw(
            graphics::Text::new("Hello, world!")
                .set_scale(48.),
            dest_point,
        );

        canvas.draw(&self.board, dest_point);

        canvas.draw(&self.black_king_image, dest_point);

        canvas.finish(ctx)?;

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ctx.time.fps());
        }

        Ok(())
    }
}

fn main() {
    let mut game = Game::new();

    let square = Square { file: File::E, rank: Rank::R2 };

    let moves = game.possible_moves(square, true).unwrap();
    
    let (board_move, vec_moves) = moves;
    
    // println!("{:?}", boardMove);
    println!("vec_moves = {:?}", vec_moves);

    for a in board_move.rows.squares {
        for b in a.squares {
            if let Some(m) = b {
                println!("{:?}", m);
            }
        }
    }

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        PathBuf::from("./resources")
    };

    let builder = ContextBuilder::new("alvinw-chess-gui", "alvinw")
        .window_setup(WindowSetup::default().title("alvinw-chess-gui"))
        .add_resource_path(resource_dir);
    
    let (mut ctx, event_loop) = builder.build().expect("Failed to start ggez.");

    let state = MainState::new(&mut ctx).expect("Failed to create MainState.");
    event::run(ctx, event_loop, state);
}
