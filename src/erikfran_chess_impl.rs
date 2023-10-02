use erikfran_chess::{Piece, util::{Square, BoardMove}, Color, PieceTypes, Move, MoveError};

use crate::bridge::{self, GameState};
use crate::server::ServerGame;

impl bridge::ChessGame for erikfran_chess::Game {
    fn get_pieces(&self) -> [[Option<Piece>; 8]; 8] {
        let mut ret = [[None; 8]; 8];
        for (row_index, row) in self.board.rows.squares.iter().enumerate() {
            for (piece_index, piece) in row.squares.iter().enumerate() {
                ret[row_index][piece_index] = piece.map(|piece| piece.into());
            }
        }
        ret
    }

    fn get_piece(&self, at: Square) -> Option<Piece> {
        self.board[at]
    }

    fn get_state(&self) -> GameState {
        if self.check {
            GameState::Check
        } else {
            GameState::Normal
        }
    }

    fn is_check(&self) -> bool {
        self.check
    }

    fn current_turn(&self) -> Color {
        self.turn.into()
    }

    fn promote(&mut self, promotion_square: Square, piece_type: PieceTypes) {
        // This backend does not implement promotion. So we implement it manually here
        // instead.

        let piece = self.board[promotion_square].expect("Expected piece to promote");
        self.board[promotion_square] = Some(erikfran_chess::Piece {
            color: piece.color,
            piece: piece_type,
        });
    }

    fn possible_moves(&mut self, at: Square) -> Result<(BoardMove, Vec<Move>), MoveError> {
        erikfran_chess::Game::possible_moves(self, at.into(), true)
    }

    fn perform_move(&mut self, mv: Move) -> Result<(), MoveError> {
        self.try_move(mv)
    }

    fn get_if_server(&mut self) -> Option<&mut ServerGame> {
        None
    }
}