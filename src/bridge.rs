//! This module describes an abstract api for interacting with a chess game
//! regardless of backend.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub piece: PieceType,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Normal,
    Check,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square {
    file: u8,
    rank: u8,
}

impl Square {
    pub fn new(file: u8, rank: u8) -> Self {
        if file > 7 || rank > 7 {
            panic!("Invalid Square: file: {} rank: {}", file, rank);
        }
        Square { file, rank }
    }
    pub fn file(&self) -> u8 { self.file }
    pub fn rank(&self) -> u8 { self.rank }
}

pub struct Move {
    from: Square,
    to: Square,
}

pub trait ChessGame {
    fn get_pieces(&self) -> [[Option<Piece>; 8]; 8];

    fn get_piece(&self, at: Square) -> Option<Piece>;

    fn get_state(&self) -> GameState;

    fn is_check(&self) -> bool;

    fn current_turn(&self) -> Color;

    fn promote(&self, piece: PieceType);

    fn possible_moves(&mut self, at: Square) -> Vec<Move>;

    fn perform_move(&mut self, mv: Move) -> String;
}