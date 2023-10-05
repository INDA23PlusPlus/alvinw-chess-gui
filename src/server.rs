use std::net::TcpListener;

use chess_network_protocol::{ServerToClient, Joever, ClientToServerHandshake, ServerToClientHandshake, Piece as ProtocolPiece, Move as ProtocolMove, Features, ClientToServer};
use erikfran_chess::{Move, MoveError, Piece, PieceTypes};
use erikfran_chess::util::{BoardMove, Square};

use crate::bridge::{self, ChessGame};
use crate::json_tcp_stream::JsonTcpStream;

pub struct ServerGame {
    game: erikfran_chess::Game,
    listener: TcpListener,
    client: Option<JsonTcpStream>,
    protocol_state: ProtocolState,
    last_move_made: Option<ProtocolMove>,
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

    pub fn try_accept_client(&mut self) {
        let res = self.listener.accept();
        if let Ok((stream, addr)) = res {
            self.client = Some(JsonTcpStream::new(stream));
            self.protocol_state = ProtocolState::Handshake;
            println!("{} connected", addr);
        } else {
            // println!("{:?}", res);
        }
    }

    /// Attempt to handle the client handshake if it has been received.
    /// 
    /// # Panics
    /// This method will panic if the client is not connected.
    pub fn try_handshake(&mut self) {
        let stream = self.client.as_mut().expect("Must have client to handle handshake.");

        let handshake: ClientToServerHandshake = match stream.read() {
            Some(handshake) => handshake,
            None => return,
        };

        println!("Got handshake {:?}", handshake);
        let server_handshake = ServerToClientHandshake {
            board: convert_board(self.get_pieces()),
            moves: self.get_moves(),
            joever: Joever::Ongoing,
            features: vec![
                Features::PossibleMoveGeneration,
            ],
        };
        let stream = self.client.as_mut().unwrap().stream();
        serde_json::to_writer(stream, &server_handshake).unwrap();
        println!("sent {:?}", server_handshake);
        self.protocol_state = ProtocolState::Play;

    }

    pub fn get_moves(&mut self) -> Vec<ProtocolMove> {
        let mut moves = vec![];
        for file in 0..8 {
            for rank in 0..8 {
                let square = (file, rank).try_into().unwrap();
                if let Ok((board_move, _castle_moves)) = self.game.possible_moves(square, true) {
                    for row in board_move.rows.squares {
                        for piece in row.squares {
                            if let Some(mv) = piece {
                                moves.push(convert_move(mv));
                            }
                        }
                    }
                }
            }
        }
        moves
    }

    pub fn send_state(&mut self) {
        let board = convert_board(self.get_pieces());
        let state = ServerToClient::State {
            board,
            moves: self.get_moves(),
            move_made: self.last_move_made.expect("Cannot call send_state when no last move."),
            joever: Joever::Ongoing,
        };
        if let Some(ref mut stream) = &mut self.client {
            serde_json::to_writer(stream.stream(), &state).unwrap();
        }
    }
}

impl bridge::ChessGame for ServerGame {
    fn update(&mut self) {
        let client = match &mut self.client {
            Some(client) => client,
            None => return,
        };

        let packet: ClientToServer = match client.read() {
            Some(packet) => packet,
            None => return,
        };

        match packet {
            ClientToServer::Move(mv) => {
                let from: Square = (mv.start_x as i32, mv.start_y as i32).try_into().unwrap();
                let to: Square = (mv.end_x as i32, mv.end_y as i32).try_into().unwrap();
                let erikfran_move = Move::Normal { from, to };

                match self.game.perform_move(erikfran_move) {
                    Ok(_) => {
                        // Client move accepted.
                        self.last_move_made = Some(mv);
                        self.send_state();
                    }
                    Err(err) => {
                        let error_packet = ServerToClient::Error {
                            board: convert_board(self.get_pieces()),
                            moves: self.get_moves(),
                            joever: Joever::Ongoing,
                            message: format!("{}", err),
                        };
                        serde_json::to_writer(self.client.as_mut().unwrap().stream(), &error_packet).unwrap();
                    }
                }
            }
            ClientToServer::Resign => {}
            ClientToServer::Draw => {}
        }
    }

    fn perform_move(&mut self, mv: erikfran_chess::Move) -> Result<(), MoveError> {
        // Perform the move
        let res = self.game.perform_move(mv);
        self.last_move_made = Some(convert_move(mv));
        if res.is_ok() {
            // If the move actually happened, notify the client.
            self.send_state();
        }
        res
    }

    fn promote(
        &mut self,
        promotion_square: Square,
        piece: PieceTypes,
    ) {
        self.game.promote(promotion_square, piece)

        // TODO networking
    }

    // Delegate informational methods to the erikfran chess backend.

    fn get_pieces(&self) -> [[Option<Piece>; 8]; 8] {
        self.game.get_pieces()
    }

    fn get_piece(&self, at: Square) -> Option<Piece> {
        self.game.get_piece(at)
    }

    fn possible_moves( &mut self, at: Square) -> Result<(BoardMove, Vec<Move>), MoveError> {
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

fn convert_piece(erikfran_piece: Option<Piece>) -> ProtocolPiece {
    match erikfran_piece {
        None => ProtocolPiece::None,
        Some(erikfran_piece) => match erikfran_piece.color {
            erikfran_chess::Color::White => match erikfran_piece.piece {
                PieceTypes::Pawn(_) => ProtocolPiece::WhitePawn,
                PieceTypes::Bishop => ProtocolPiece::WhiteBishop,
                PieceTypes::Knight => ProtocolPiece::WhiteKnight,
                PieceTypes::Rook => ProtocolPiece::WhiteRook,
                PieceTypes::Queen => ProtocolPiece::WhiteQueen,
                PieceTypes::King => ProtocolPiece::WhiteKing,
            }
            erikfran_chess::Color::Black => match erikfran_piece.piece {
                PieceTypes::Pawn(_) => ProtocolPiece::BlackPawn,
                PieceTypes::Bishop => ProtocolPiece::BlackBishop,
                PieceTypes::Knight => ProtocolPiece::BlackKnight,
                PieceTypes::Rook => ProtocolPiece::BlackRook,
                PieceTypes::Queen => ProtocolPiece::BlackQueen,
                PieceTypes::King => ProtocolPiece::BlackKing,
            }
        }
    }   
}

fn convert_board(erikfran_board: [[Option<Piece>; 8]; 8]) -> [[ProtocolPiece; 8]; 8] {
    erikfran_board.map(
        |row| row.map(
            |piece| convert_piece(piece)
        )
    )
}

pub fn convert_move(mv: Move) -> ProtocolMove {
    let (from, to) = match mv {
        Move::Normal { from, to } => (from, to),
        Move::Castle { .. } => panic!("Castling is unsupported."),
    };
    ProtocolMove {
        start_x: i32::from(from.file) as usize,
        start_y: i32::from(from.rank) as usize,
        end_x: i32::from(to.file) as usize,
        end_y: i32::from(to.rank) as usize,
        promotion: ProtocolPiece::None,
    }
}