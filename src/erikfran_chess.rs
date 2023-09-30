use crate::bridge;

impl bridge::ChessGame for erikfran_chess::Game {
    fn get_pieces(&self) -> [[Option<bridge::Piece>; 8]; 8] {
        let mut ret = [[None; 8]; 8];
        for (row_index, row) in self.board.rows.squares.iter().enumerate() {
            for (piece_index, piece) in row.squares.iter().enumerate() {
                ret[row_index][piece_index] = piece.map(|piece| piece.into());
            }
        }
        ret
    }

    fn get_piece(&self, at: bridge::Square) -> Option<bridge::Piece> {
        let square = (file, rank).try_into()
            .expect(&format!("Unable to convert {} {} to a square.", file, rank));
        self.board[square]
    }

    fn get_state(&self) -> bridge::GameState {
        if self.check {
            bridge::GameState::Check
        } else {
            bridge::GameState::Normal
        }
    }

    fn is_check(&self) -> bool {
        self.check
    }

    fn current_turn(&self) -> bridge::Color {
        self.turn.into()
    }

    fn promote(&self, piece_type: PieceType) {
        // This backend does not implement promotion. So we implement it manually here
        // instead.

        // Find the piece to promote
        for (rank, row) in self.get_pieces().iter().enumerate() {
            if rank != 0 || rank != 7 {
                continue;
            }
            for (file, piece) in row.iter().enumerate() {
                if let Some(piece) = piece {
                    if piece.piece == PieceType::Pawn {
                        let square: erikfran_chess::util::Square = (file as u8, rank as u8).into();
                        self.board[square] = Some(erikfran_chess::Piece {
                            color: match piece.color {
                                bridge::Color::White => erikfran_chess::Color::White,
                                bridge::Color::Black => erikfran_chess::Color::Black,
                            },
                            piece: match piece_type {
                                bridge::PieceType::Pawn => erikfran_chess::PieceTypes::Pawn,
                                bridge::PieceType::Bishop => erikfran_chess::PieceTypes::Bishop,
                                bridge::PieceType::Knight => erikfran_chess::PieceTypes::Knight,
                                bridge::PieceType::Rook => erikfran_chess::PieceTypes::Rook,
                                bridge::PieceType::Queen => erikfran_chess::PieceTypes::Queen,
                                bridge::PieceType::King => erikfran_chess::PieceTypes::King,
                            }
                        });
                    }
                }
            }
        }
    }

    fn possible_moves(&mut self, at: bridge::Square) -> Vec<bridge::Move> {
        let result = erikfran_chess::Game::possible_moves(&mut self, at.into(), true);
        todo!()
    }
}

impl Into<bridge::Piece> for erikfran_chess::Piece {
    fn into(self) -> bridge::Piece {
        bridge::Piece { piece: self.piece.into(), color: self.color.into() }
    }
}

impl Into<bridge::Color> for erikfran_chess::Color {
    fn into(self) -> bridge::Color {
        match self {
            erikfran_chess::Color::White => bridge::Color::White,
            erikfran_chess::Color::Black => bridge::Color::Black,
        }
    }
}

impl Into<bridge::PieceType> for erikfran_chess::PieceTypes {
    fn into(self) -> bridge::PieceType {
        match self {
            erikfran_chess::PieceTypes::Pawn(_) => bridge::PieceType::Pawn,
            erikfran_chess::PieceTypes::Bishop => bridge::PieceType::Bishop,
            erikfran_chess::PieceTypes::Knight => bridge::PieceType::Knight,
            erikfran_chess::PieceTypes::Rook => bridge::PieceType::Rook,
            erikfran_chess::PieceTypes::Queen => bridge::PieceType::Queen,
            erikfran_chess::PieceTypes::King => bridge::PieceType::King,
        }
    }
}

impl Into<erikfran_chess::util::Square> for bridge::Square {
    fn into(self) -> erikfran_chess::util::Square {
        (self.file() as i32, self.rank() as i32).try_into().unwrap()
    }
}