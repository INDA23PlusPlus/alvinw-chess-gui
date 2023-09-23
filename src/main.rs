use chess::{Game, util::{Square, File, Rank}};

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
}
