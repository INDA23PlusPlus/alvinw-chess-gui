use chess::{Game, util::{Square, File, Rank}};
use ggez::{ContextBuilder, event, Context, GameResult, graphics, conf::WindowSetup};

struct MainState {
    frames: usize,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState { frames: 0 };
        Ok(s)
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

    let builder = ContextBuilder::new("alvinw-chess-gui", "alvinw")
        .window_setup(WindowSetup::default().title("alvinw-chess-gui"));
    
    let (mut ctx, event_loop) = builder.build().expect("Failed to start ggez.");

    let state = MainState::new(&mut ctx).expect("Failed to create MainState.");
    event::run(ctx, event_loop, state);
}
