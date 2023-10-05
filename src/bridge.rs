//! This module describes an abstract api for interacting with a chess game
//! regardless of backend.

use erikfran_chess::{Piece, util::{Square, BoardMove}, Color, Move, PieceTypes, MoveError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Normal,
    Check,
}

pub trait ChessGame {
    fn update(&mut self);

    fn get_pieces(&self) -> [[Option<Piece>; 8]; 8];

    fn get_piece(&self, at: Square) -> Option<Piece>;

    fn get_state(&self) -> GameState;

    fn is_check(&self) -> bool;

    fn current_turn(&self) -> Color;

    fn promote(&mut self, promotion_square: Square, piece: PieceTypes);

    fn possible_moves(&mut self, at: Square) -> Result<(BoardMove, Vec<Move>), MoveError>;

    fn perform_move(&mut self, mv: Move) -> Result<(), MoveError>;
}