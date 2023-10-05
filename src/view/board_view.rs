use erikfran_chess::{util::{Square, BoardMove, Rank}, PieceTypes, MoveError, Move, CastlingSide};
use ggez::{ContextBuilder, event::{self, MouseButton}, Context, GameResult, graphics::{self, Image, MeshBuilder, FillOptions, Rect, Color, Mesh, Text, DrawParam}, conf::{WindowSetup, WindowMode}, glam::Vec2};
use crate::bridge;
use crate::server::{ProtocolState, ServerGame};
use crate::view::View;

const SQUARE_SIZE: f32 = 64.0;
const BOARD_SIZE: f32 = SQUARE_SIZE * 8.0;

struct CastlingPossibility {
    _color: erikfran_chess::Color,
    kingside: bool,
    queenside: bool,
}

pub struct BoardView<T: bridge::ChessGame> {
    frames: usize,
    board: Mesh,
    game: T,
    board_start: Vec2,
    scale: f32,
    possible_moves: Option<BoardMove>,
    possible_castling: Option<CastlingPossibility>,
    white_icons: PieceIcons,
    black_icons: PieceIcons,
    latest_error: Option<MoveError>,
    promotion_square: Option<Square>,
    promotion_coordinates: Option<PromotionCoordinates>,
}

struct PieceIcons {
    king: Image,
    queen: Image,
    bishop: Image,
    rook: Image,
    knight: Image,
    pawn: Image,
}

/// The screen coordinates where the promotion GUI is.
struct PromotionCoordinates {
    queen_pos: Vec2,
    bishop_pos: Vec2,
    rook_pos: Vec2,
    knight_pos: Vec2,
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

impl<T: bridge::ChessGame> BoardView<T> {
    pub fn new(ctx: &mut Context, game: T) -> GameResult<BoardView<T>> {
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

        Ok(BoardView {
            frames: 0,
            game,
            board,
            board_start: Vec2::new(0.0, 0.0),
            scale: 1.0,
            possible_moves: None,
            possible_castling: None,
            white_icons,
            black_icons,
            latest_error: None,
            promotion_square: None,
            promotion_coordinates: None,
        })
    }
}

impl<T: bridge::ChessGame> View for BoardView<T> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.game.update();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));
        
        let screen_coordinates = canvas.screen_coordinates();

        let mut scale = 1_f32;
        if let Some(screen) = screen_coordinates {
            loop {
                let next_size = BOARD_SIZE * (scale + 1.0);
                if next_size > screen.w || next_size > screen.h {
                    break;
                }
                scale += 1.0;
            }
        }
        self.scale = scale;
        let scale_vec = Vec2::new(scale, scale);

        self.board_start = match screen_coordinates {
            None => Vec2::new(0.0, 0.0),
            Some(rect) => {
                Vec2::new((rect.w - BOARD_SIZE * scale) / 2.0, (rect.h - BOARD_SIZE * scale) / 2.0)
            }
        };

        let params = DrawParam::new().dest(self.board_start).scale(scale_vec);
        canvas.draw(&self.board, params);

        for (rank_index, row) in self.game.get_pieces().iter().enumerate() {
            for (file_index, piece) in row.iter().enumerate() {
                let square: Square = (file_index as i32, rank_index as i32).try_into().unwrap();
                let pos = self.board_start + Vec2::new(file_index as f32 * SQUARE_SIZE * scale, (7 - rank_index) as f32 * SQUARE_SIZE * scale);
                let draw_param = DrawParam::new().dest(pos).scale(scale_vec);
                if let Some(piece) = piece {
                    let icons = match piece.color {
                        erikfran_chess::Color::White => &self.white_icons,
                        erikfran_chess::Color::Black => &self.black_icons,
                    };
                    let image = icons.get_image(piece.piece);
                    canvas.draw(image, draw_param);
                }
                if let Some(possible_moves) = self.possible_moves {
                    if possible_moves[square].is_some() {
                        let sin = ((self.frames as f64) / 25.0).sin() + 1.0;
                        let green = (sin * 32.0) as u8 + 192;
                        let radius = (sin * 2.0 + 16.0) as f32;
                        let color = Color::from_rgb(0, green, 0);
                        let mesh = Mesh::from_data(ctx, MeshBuilder::new().circle(
                            graphics::DrawMode::Fill(FillOptions::default()),
                            Vec2::new(32.0, 32.0),
                            radius,
                            0.5,
                            color
                        ).unwrap().build());
                        canvas.draw(&mesh, draw_param);
                    }
                }
            }
        }

        if let Some(possible_castling) = &self.possible_castling {
            // TODO implement when backend stops panicking when castling is possible.
            if possible_castling.queenside {
                let mesh = Mesh::from_data(ctx, MeshBuilder::new().rectangle(
                    graphics::DrawMode::Fill(FillOptions::default()),
                    Rect::new(32.0, 32.0, 100.0, 10.0),
                    Color::BLACK,
                ).unwrap().build());
                let draw_param = DrawParam::new().dest(self.board_start - Vec2::new(-100.0, 0.0));
                canvas.draw(&mesh, draw_param);
            }
        }

        if let Some(promotion_square) = self.promotion_square {
            let width = ((SQUARE_SIZE + 4.0) * 4.0) * scale;
            let height = (SQUARE_SIZE + 8.0) * scale;
            let pos = self.board_start + Vec2::new(
                (BOARD_SIZE * scale - width) / 2.0,
                (BOARD_SIZE * scale - height) / 2.0,
            );

            let color = self.game.get_piece(promotion_square).map_or(erikfran_chess::Color::White, |piece| piece.color);
            let piece_icons = match color {
                erikfran_chess::Color::White => &self.white_icons,
                erikfran_chess::Color::Black => &self.black_icons,
            };

            let mut mesh = MeshBuilder::new();
            mesh.rectangle(
                graphics::DrawMode::Fill(FillOptions::default()),
                Rect::new(0.0, 0.0, width, height),
                if color == erikfran_chess::Color::White { Color::BLACK } else { Color::WHITE }
            ).expect("Failed to draw rectangle.");
            let mesh = Mesh::from_data(ctx, mesh.build());
            canvas.draw(&mesh, pos);

            let queen_pos = pos + Vec2::new(4.0 * scale, 4.0 * scale);
            let params = DrawParam::new().dest(queen_pos).scale(scale_vec);
            canvas.draw(piece_icons.get_image(PieceTypes::Queen), params);

            let bishop_pos = pos + Vec2::new(SQUARE_SIZE * scale + 8.0 * scale, 4.0 * scale);
            let params = DrawParam::new().dest(bishop_pos).scale(scale_vec);
            canvas.draw(piece_icons.get_image(PieceTypes::Bishop), params);

            let rook_pos = pos + Vec2::new(2.0 * SQUARE_SIZE * scale + 16.0 * scale, 4.0 * scale);
            let params = DrawParam::new().dest(rook_pos).scale(scale_vec);
            canvas.draw(piece_icons.get_image(PieceTypes::Rook), params);
            
            let knight_pos = pos + Vec2::new(3.0 * SQUARE_SIZE * scale + 24.0 * scale, 4.0 * scale);
            let params = DrawParam::new().dest(knight_pos).scale(scale_vec);
            canvas.draw(piece_icons.get_image(PieceTypes::Knight), params);

            self.promotion_coordinates = Some(PromotionCoordinates { queen_pos, bishop_pos, rook_pos, knight_pos });
        }

        if self.game.is_check() {
            let mut text = Text::new("check");
            text.set_scale(16.0 * scale);
            let sin = ((self.frames as f64) / 25.0).sin() + 1.0;
            let color_value = (sin * 128.0) as u8;
            let color = Color::from_rgb(255, color_value, color_value);
            let text_size = text.measure(ctx).expect("measure");
            let pos = Vec2::new(self.board_start.x + (BOARD_SIZE * scale - text_size.x) / 2.0, self.board_start.y - (20.0 * scale));
            let param = DrawParam::new().dest(pos).color(color);
            canvas.draw(&text, param);

        }

        if let Some(error) = &self.latest_error {
            let mut text = Text::new(error.to_string());
            text.set_scale(16.0 * self.scale);
            let text_size = text.measure(ctx).unwrap();
            let text_pos = match screen_coordinates {
                None => Vec2::new(0.0, 0.0),
                Some(screen) => {
                    let board_end = self.board_start.y + BOARD_SIZE * scale;
                    Vec2::new(
                        (screen.w - text_size.x) / 2.0,
                        board_end + (screen.h - board_end - text_size.y) / 2.0
                    )
                }
            };
            canvas.draw(&text, text_pos);
        }

        let color_text = if self.game.current_turn() == erikfran_chess::Color::White { "white" } else { "black" };
        let mut color_text = Text::new(format!("{color_text}'s turn"));
        color_text.set_scale(16.0 * self.scale);
        canvas.draw(&color_text, Vec2::new(20.0, 40.0));

        canvas.finish(ctx)?;

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {:.2}", ctx.time.fps());
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
        if let (Some(promotion_square), Some(coords)) = (self.promotion_square, &self.promotion_coordinates) {
            let size = SQUARE_SIZE * self.scale;
            let clicked_inside = |piece_pos: Vec2| -> bool {
                x > piece_pos.x && x < piece_pos.x + size
                && y > piece_pos.y && y < piece_pos.y + size
            };
            let mut promoted = false;
            let mut promote = |piece_type: PieceTypes| {
                println!("promoting to {piece_type:?}");
                self.game.promote(promotion_square, piece_type);
                promoted = true;
            };

            if clicked_inside(coords.queen_pos) {
                promote(PieceTypes::Queen);
            }
            if clicked_inside(coords.bishop_pos) {
                promote(PieceTypes::Bishop);
            }
            if clicked_inside(coords.rook_pos) {
                promote(PieceTypes::Rook);
            }
            if clicked_inside(coords.knight_pos) {
                promote(PieceTypes::Knight);
            }

            if promoted {
                self.promotion_square = None;
                self.promotion_coordinates = None;
            }

            // While the promotion gui is open no other moves can be made.
            return Ok(());
        }

        let rel_x = x - self.board_start.x;
        let rel_y = y - self.board_start.y;
        let file = (rel_x / SQUARE_SIZE / self.scale) as i32;
        let rank = 7 - (rel_y / SQUARE_SIZE / self.scale) as i32;
        let square: Option<Square> = (file, rank).try_into().ok();
        println!("Mouse button pressed: {button:?}, x: {file}, y: {rank}");

        if let Some(square) = square {
            println!("Clicked {square:?}");
            if let Some(mv) = self.possible_moves.and_then(|moves| moves[square]) {
                let result = self.game.perform_move(mv);
                match result {
                    Ok(_) => {
                        self.possible_moves = None;
                        self.possible_castling = None;
                        self.latest_error = None;
                        match mv {
                            Move::Normal { from: _, to } => {
                                let is_pawn: bool = self.game.get_piece(to).map_or(false, |piece| piece.piece == PieceTypes::Pawn(true));
                                if (to.rank == Rank::R1 || to.rank == Rank::R8) && is_pawn {
                                    // Promotion
                                    self.promotion_square = Some(to);
                                }
                            },
                            _ => {},
                        }
                    },
                    Err(err) => {
                        self.latest_error = Some(err);
                    },
                }
            } else {
                let possible_moves = self.game.possible_moves(square);
                match possible_moves {
                    Ok(possible_moves) => {
                        let (board_move, castling_moves) = possible_moves;
                        self.possible_moves = Some(board_move);
                        
                        let color = self.game.get_piece(square).map_or(erikfran_chess::Color::White, |piece| piece.color);
                        let mut possible_castling = CastlingPossibility {
                            _color: color,
                            kingside: false,
                            queenside: false,
                        };
                        for mv in castling_moves {
                            if let Move::Castle { side } = mv {
                                if side == CastlingSide::KingSide {
                                    possible_castling.kingside = true;
                                }
                                if side == CastlingSide::QueenSide {
                                    possible_castling.queenside = true;
                                }
                            }
                        }
                        self.possible_castling = Some(possible_castling);
                    },
                    Err(err) => {
                        self.latest_error = Some(err);
                    },
                }
            }
        }


        Ok(())
    }

}
