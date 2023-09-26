use std::{path::PathBuf, env};

use chess::{Game, util::{Square, File, Rank, Board, BoardMove}, Move, PieceTypes, MoveError};
use ggez::{ContextBuilder, event::{self, MouseButton}, Context, GameResult, graphics::{self, Image, MeshBuilder, FillOptions, Rect, Color, Mesh, Text}, conf::{WindowSetup, WindowMode}, glam::Vec2};

const SQUARE_SIZE: f32 = 64.0;
const BOARD_SIZE: f32 = SQUARE_SIZE * 8.0;

struct MainState {
    frames: usize,
    board: Mesh,
    highlight: Image,
    game: Game,
    board_start: Vec2,
    possible_moves: Option<BoardMove>,
    white_icons: PieceIcons,
    black_icons: PieceIcons,
    latest_error: Option<MoveError>,
}

struct PieceIcons {
    king: Image,
    queen: Image,
    bishop: Image,
    rook: Image,
    knight: Image,
    pawn: Image,
}

impl PieceIcons {
    fn new(ctx: &Context, prefix: &str) -> GameResult<Self> {
        Ok(Self {
            king: Image::from_path(ctx, format!("/{prefix}_king.png"))?,
            queen: Image::from_path(ctx, format!("/{prefix}_queen.png"))?,
            bishop: Image::from_path(ctx, format!("/{prefix}_bishop.png"))?,
            rook: Image::from_path(ctx, format!("/{prefix}_rook.png"))?,
            knight: Image::from_path(ctx, format!("/{prefix}_knight.png"))?,
            pawn: Image::from_path(ctx, format!("/{prefix}_pawn.png"))?,
        })
    }

    fn get_image(&self, piece_type: PieceTypes) -> &Image {
        match piece_type {
            PieceTypes::King => &self.king,
            PieceTypes::Queen => &self.queen,
            PieceTypes::Bishop => &self.bishop,
            PieceTypes::Rook => &self.rook,
            PieceTypes::Knight => &self.knight,
            PieceTypes::Pawn(_) => &self.pawn,
        }
    }
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let white_icons = PieceIcons::new(ctx, "white")?;
        let black_icons = PieceIcons::new(ctx, "black")?;

        let black_square_color: Color = Color::from_rgb(120, 100, 80);
        let white_square_color: Color = Color::from_rgb(220, 190, 30);

        let mut board_builder = MeshBuilder::new();
        for i in 0..8 {
            for j in 0..8 {
                let color = if (i + j) % 2 == 0 { white_square_color } else { black_square_color };
                board_builder.rectangle(
                    graphics::DrawMode::Fill(FillOptions::default()),
                    Rect::new(i as f32 * SQUARE_SIZE, j as f32 * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE),
                    color
                ).expect("Failed to draw rectangle.");
            }
        }
        let board = Mesh::from_data(ctx, board_builder.build());

        let highlight = Image::from_color(ctx, SQUARE_SIZE as u32, SQUARE_SIZE as u32, Some(Color::GREEN));

        Ok(MainState {
            frames: 0,
            game: Game::new(),
            board,
            highlight,
            board_start: Vec2::new(0.0, 0.0),
            possible_moves: None,
            white_icons,
            black_icons,
            latest_error: None,
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

        let screen_coordinates = canvas.screen_coordinates();

        self.board_start = match screen_coordinates {
            None => Vec2::new(0.0, 0.0),
            Some(rect) => {
                Vec2::new((rect.w - BOARD_SIZE) / 2.0, (rect.h - BOARD_SIZE) / 2.0)
            }
        };

        canvas.draw(&self.board, self.board_start);

        for (rank_index, row) in self.game.board.rows.squares.iter().enumerate() {
            for (file_index, piece) in row.squares.iter().enumerate() {
                let square: Square = (file_index as i32, rank_index as i32).try_into().unwrap();
                let pos = self.board_start + Vec2::new(file_index as f32 * SQUARE_SIZE, (7 - rank_index) as f32 * SQUARE_SIZE);
                if let Some(possible_moves) = self.possible_moves {
                    if possible_moves[square].is_some() {
                        canvas.draw(&self.highlight, pos);
                    }
                }
                if let Some(piece) = piece {
                    let icons = match piece.color {
                        chess::Color::White => &self.white_icons,
                        chess::Color::Black => &self.black_icons,
                    };
                    let image = icons.get_image(piece.piece);
                    canvas.draw(image, pos);
                }
            }
        }

        if let Some(error) = &self.latest_error {
            let text = Text::new(error.to_string());
            let text_size = text.measure(ctx).unwrap();
            let text_pos = match screen_coordinates {
                None => Vec2::new(0.0, 0.0),
                Some(rect) => {
                    Vec2::new((rect.w - text_size.x) / 2.0, self.board_start.y + (rect.h - self.board_start.y - text_size.y) / 2.0)
                }
            };
            canvas.draw(&text, text_pos);
        }

        canvas.finish(ctx)?;

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ctx.time.fps());
        }

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        let rel_x = x - self.board_start.x;
        let rel_y = y - self.board_start.y;
        let file = (rel_x / SQUARE_SIZE) as i32;
        let rank = 7 - (rel_y / SQUARE_SIZE) as i32;
        let square: Option<Square> = (file, rank).try_into().ok();
        println!("Mouse button pressed: {button:?}, x: {file}, y: {rank}");

        if let Some(square) = square {
            println!("Clicked {square:?}");
            if let Some(mv) = self.possible_moves.and_then(|moves| moves[square]) {
                let result = self.game.try_move(mv);
                match result {
                    Ok(_) => {
                        self.possible_moves = None;
                    },
                    Err(err) => {
                        println!("error: {err:?}");
                        self.latest_error = Some(err);
                    },
                }
            } else {
                let possible_moves = self.game.possible_moves(square, true);
                self.possible_moves = possible_moves.ok().map(|(board_move, _castling_moves)| board_move);
            }
        }


        Ok(())
    }

}

fn main() {
    let mut game = Game::new();

    game.try_move(Move::Normal { from: Square {
        file: File::F, rank: Rank::R2,
    }, to: Square {
        file: File::F, rank: Rank::R3,
    } }).unwrap();

    let square = Square { file: File::F, rank: Rank::R3 };

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
        .window_mode(WindowMode::default()
            .resizable(true)
        )
        .add_resource_path(resource_dir);
    
    let (mut ctx, event_loop) = builder.build().expect("Failed to start ggez.");

    let state = MainState::new(&mut ctx).expect("Failed to create MainState.");
    event::run(ctx, event_loop, state);
}
