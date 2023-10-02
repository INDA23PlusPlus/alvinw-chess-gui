use std::net::{TcpListener, TcpStream};

use chess_network_protocol::{ServerToClient, Joever, ClientToServerHandshake};

use crate::bridge::{self, ChessGame};

pub struct ServerGame {
    game: erikfran_chess::Game,
    listener: TcpListener,
    client: Option<TcpStream>,
    protocol_state: ProtocolState,
    last_move_made: Option<chess_network_protocol::Move>,
}

#[derive(Debug, Copy, Clone)]
pub enum ProtocolState {
    NotConnected,
    Handshake,
    Play,
}

impl ServerGame {
    pub fn new(game: erikfran_chess::Game, port: u16) -> Self {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
        listener.set_nonblocking(true).unwrap();
        Self {
            game,
            listener,
            client: None,
            protocol_state: ProtocolState::NotConnected,
            last_move_made: None,
        }
    }

    pub fn get_protocol_state(&self) -> ProtocolState { self.protocol_state }

    pub fn is_client_connected(&self) -> bool {
        self.client.is_some()
    }

    pub fn try_accept_client(&mut self) {
        let res = self.listener.accept();
        if let Ok((stream, addr)) = res {
            self.client = Some(stream);
            self.protocol_state = ProtocolState::Handshake;
            println!("{} connected", addr);
        }
    }

    /// Attempt to handle the client handshake if it has been received.
    /// 
    /// # Panics
    /// This method will panic if the client is not connected.
    pub fn try_handshake(&mut self) {
        let stream = self.client.as_ref().expect("Must have client to handle handshake.");
        let res: ClientToServerHandshake = serde_json::from_reader(stream).expect("Failed to read handshake.");
        println!("Got handshake {:?}", res);

    }

    pub fn send_state(&self) {
        if let Some(stream) = &self.client {
            let board = convert_board(self.get_pieces());
            let state = ServerToClient::State {
                board,
                moves: vec![],
                move_made: self.last_move_made.expect("Cannot call send_state when no last move."),
                joever: Joever::Ongoing,
            };
            serde_json::to_writer(stream, &state).unwrap();
        }
    }
}

impl bridge::ChessGame for ServerGame {
    fn get_if_server(&mut self) -> Option<&mut ServerGame> {
        Some(self)
    }
    fn perform_move(&mut self, mv: erikfran_chess::Move) -> Result<(), erikfran_chess::MoveError> {
        // Perform the move
        let res = self.game.perform_move(mv);
        if res.is_ok() {
            // If the move actually happened, notify the client.
            self.send_state();
        }
        res
    }

    fn promote(
        &mut self,
        promotion_square: erikfran_chess::util::Square,
        piece: erikfran_chess::PieceTypes,
    ) {
        self.game.promote(promotion_square, piece)

        // TODO networking
    }

    // Delegate informational methods to the erikfran chess backend.

    fn get_pieces(&self) -> [[Option<erikfran_chess::Piece>; 8]; 8] {
        self.game.get_pieces()
    }

    fn get_piece(&self, at: erikfran_chess::util::Square) -> Option<erikfran_chess::Piece> {
        self.game.get_piece(at)
    }

    fn possible_moves(
        &mut self,
        at: erikfran_chess::util::Square,
    ) -> Result<
        (erikfran_chess::util::BoardMove, Vec<erikfran_chess::Move>),
        erikfran_chess::MoveError,
    > {
        bridge::ChessGame::possible_moves(&mut self.game, at)
    }

    fn get_state(&self) -> bridge::GameState {
        self.game.get_state()
    }

    fn is_check(&self) -> bool {
        self.game.is_check()
    }

    fn current_turn(&self) -> erikfran_chess::Color {
        self.game.current_turn()
    }
}

fn convert_piece(erikfran_piece: Option<erikfran_chess::Piece>) -> chess_network_protocol::Piece {
    match erikfran_piece {
        None => chess_network_protocol::Piece::None,
        Some(erikfran_piece) => match erikfran_piece.color {
            erikfran_chess::Color::White => match erikfran_piece.piece {
                erikfran_chess::PieceTypes::Pawn(_) => chess_network_protocol::Piece::WhitePawn,
                erikfran_chess::PieceTypes::Bishop => chess_network_protocol::Piece::WhiteBishop,
                erikfran_chess::PieceTypes::Knight => chess_network_protocol::Piece::WhiteKnight,
                erikfran_chess::PieceTypes::Rook => chess_network_protocol::Piece::WhiteRook,
                erikfran_chess::PieceTypes::Queen => chess_network_protocol::Piece::WhiteQueen,
                erikfran_chess::PieceTypes::King => chess_network_protocol::Piece::WhiteKing,
            }
            erikfran_chess::Color::Black => match erikfran_piece.piece {
                erikfran_chess::PieceTypes::Pawn(_) => chess_network_protocol::Piece::BlackPawn,
                erikfran_chess::PieceTypes::Bishop => chess_network_protocol::Piece::BlackBishop,
                erikfran_chess::PieceTypes::Knight => chess_network_protocol::Piece::BlackKnight,
                erikfran_chess::PieceTypes::Rook => chess_network_protocol::Piece::BlackRook,
                erikfran_chess::PieceTypes::Queen => chess_network_protocol::Piece::BlackQueen,
                erikfran_chess::PieceTypes::King => chess_network_protocol::Piece::BlackKing,
            }
        }
    }   
}

fn convert_board(erikfran_board: [[Option<erikfran_chess::Piece>; 8]; 8]) -> [[chess_network_protocol::Piece; 8]; 8] {
    erikfran_board.map(
        |row| row.map(
            |piece| convert_piece(piece)
        )
    )
}