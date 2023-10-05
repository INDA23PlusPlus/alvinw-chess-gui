use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use chess_network_protocol::{ClientToServerHandshake, Color as ProtocolColor, Joever, Piece as ProtocolPiece, Move as ProtocolMove, ServerToClient, ServerToClientHandshake, Features, ClientToServer};
use erikfran_chess::{Color, Move, MoveError, Piece, PieceTypes};
use erikfran_chess::util::{BoardMove, Rows, Square};
use crate::bridge::{ChessGame, GameState};
use crate::json_tcp_stream::JsonTcpStream;
use crate::server::convert_move;

pub struct ClientGame {
    socket: JsonTcpStream,
    board: [[Option<Piece>; 8]; 8],
    joever: Joever,
    moves: Vec<ProtocolMove>,
    current_turn: Color,
    handshaking: bool,
    server_features: Vec<Features>,
}

impl ClientGame {
    pub fn connect(addr: Ipv4Addr, port: u16) -> Self {
        let socket_addr = SocketAddrV4::new(addr, port);

        println!("Connecting to server...");
        let mut socket = TcpStream::connect(socket_addr).unwrap();
        socket.set_nonblocking(true).unwrap();

        println!("Sending handshake...");
        let handshake = ClientToServerHandshake {
            server_color: ProtocolColor::Black,
        };
        serde_json::to_writer(&socket, &handshake).unwrap();
        socket.flush().unwrap();

        Self {
            socket: JsonTcpStream::new(socket),
            board: [[None; 8]; 8],
            joever: Joever::Ongoing,
            moves: vec![],
            current_turn: Color::White,
            handshaking: true,
            server_features: vec![],
        }
    }

    fn set_board(&mut self, board: [[chess_network_protocol::Piece; 8]; 8]) {
        self.board = board.map(|row| row.map(|piece| convert_piece(piece)));
    }
}

impl ChessGame for ClientGame {
    fn update(&mut self) {
        if self.handshaking {
            let handshake: ServerToClientHandshake = match self.socket.read() {
                Some(handshake) => handshake,
                None => return,
            };

            self.set_board(handshake.board);
            self.joever = handshake.joever;
            self.moves = handshake.moves;
            self.server_features = handshake.features;
            self.handshaking = false;

            return;
        }
        let packet: ServerToClient = match self.socket.read() {
            Some(packet) => packet,
            None => return,
        };

        match packet {
            ServerToClient::State { board, moves, joever, .. } => {
                self.set_board(board);
                self.moves = moves;
                self.joever = joever;
                // If it was the server's turn and the server sent State it means the server
                // has made its move. If it was our turn and we just made a move, State means
                // that the move was accepted and its now the server's turn.
                self.current_turn = self.current_turn.opposite();
            }
            ServerToClient::Error { board, moves, joever, message } => {
                // The server rejected out move. This means we need to make a move again.
                self.current_turn = Color::White;
                self.set_board(board);
                self.moves = moves;
                self.joever = joever;
                println!("error message = {}", message);
            }
            ServerToClient::Resigned { .. } => {}
            ServerToClient::Draw { .. } => {}
        }
    }

    fn get_pieces(&self) -> [[Option<Piece>; 8]; 8] {
        self.board
    }

    fn get_piece(&self, at: Square) -> Option<Piece> {
        let file: i32 = at.file.into();
        let rank: i32 = at.rank.into();
        self.board[rank as usize][file as usize]
    }

    fn get_state(&self) -> GameState {
        GameState::Normal
    }

    fn is_check(&self) -> bool {
        false
    }

    fn current_turn(&self) -> Color {
        self.current_turn
    }

    fn promote(&mut self, _promotion_square: Square, _piece: PieceTypes) {

    }

    fn possible_moves(&mut self, at: Square) -> Result<(BoardMove, Vec<Move>), MoveError> {
        if !self.server_features.contains(&Features::PossibleMoveGeneration) {
            let mut board_move = BoardMove { rows: Rows { squares: [Rows { squares: [None; 8] }; 8] } };
            for file in 0..8 {
                for rank in 0..8 {
                    let square = (file, rank).try_into().unwrap();
                    board_move[square] = Some(Move::Normal {
                        from: at,
                        to: square,
                    });
                }
            }
            return Ok((board_move, vec![]));
        }
        let mut board_move = BoardMove { rows: Rows { squares: [Rows { squares: [None; 8] }; 8] } };
        for mv in &self.moves {
            let from: Square = (mv.start_x as i32, mv.start_y as i32).try_into().unwrap();
            if from == at {
                let to: Square = (mv.end_x as i32, mv.end_y as i32).try_into().unwrap();
                board_move[to] = Some(Move::Normal { from, to })
            }
        }

        Ok((board_move, vec![]))
    }

    fn perform_move(&mut self, mv: Move) -> Result<(), MoveError> {
        let packet = ClientToServer::Move(convert_move(mv));

        serde_json::to_writer(self.socket.stream(), &packet).unwrap();
        Ok(())
    }

    fn can_play_right_now(&self) -> bool {
        self.current_turn == Color::White
    }

    fn has_possible_moves(&self) -> bool {
        self.server_features.contains(&Features::PossibleMoveGeneration)
    }
}

fn convert_piece(protocol_piece: ProtocolPiece) -> Option<Piece> {
    Some(match protocol_piece {
        ProtocolPiece::BlackPawn => Piece { piece: PieceTypes::Pawn(false), color: Color::Black },
        ProtocolPiece::BlackKnight => Piece { piece: PieceTypes::Knight, color: Color::Black },
        ProtocolPiece::BlackBishop => Piece { piece: PieceTypes::Bishop, color: Color::Black },
        ProtocolPiece::BlackRook => Piece { piece: PieceTypes::Rook, color: Color::Black },
        ProtocolPiece::BlackQueen => Piece { piece: PieceTypes::Queen, color: Color::Black },
        ProtocolPiece::BlackKing => Piece { piece: PieceTypes::King, color: Color::Black },
        ProtocolPiece::WhitePawn => Piece { piece: PieceTypes::Pawn(false), color: Color::White },
        ProtocolPiece::WhiteKnight => Piece { piece: PieceTypes::Knight, color: Color::White },
        ProtocolPiece::WhiteBishop => Piece { piece: PieceTypes::Bishop, color: Color::White },
        ProtocolPiece::WhiteRook => Piece { piece: PieceTypes::Rook, color: Color::White },
        ProtocolPiece::WhiteQueen => Piece { piece: PieceTypes::Queen, color: Color::White },
        ProtocolPiece::WhiteKing => Piece { piece: PieceTypes::King, color: Color::White },
        ProtocolPiece::None => return None,
    })
}