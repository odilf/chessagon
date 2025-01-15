use std::collections::HashSet;

use chessagon_core::{
    Color,
    board::Board,
    game::{Action, Game, TimeControl},
    mov::Move,
};

use crate::Engine;

/// Very simple chessagon engine, used as an example.
pub struct Anthony {
    color: Color,
    played_moves: HashSet<Move>,
}

impl Anthony {
    pub const SEARCH_DEPTH: usize = 3;

    pub fn evaluate(board: &Board, color: Color) -> i16 {
        board.total_piece_value(color) as i16 - board.total_piece_value(color.other()) as i16
    }

    pub fn search_move(&self, board: &Board, color: Color, depth: usize) -> (Option<Move>, i16) {
        if depth == 0 {
            return (None, Self::evaluate(board, color));
        }

        let mut best_move = None;
        let mut best_move_score = i16::MIN;
        for mov in board.enumerate_moves(color) {
            if self.played_moves.contains(&mov) {
                continue;
            }

            let mut board = board.clone();
            board.apply_move_unchecked(mov, color);

            let (_best_response, opponent_score) =
                self.search_move(&board, color.other(), depth - 1);

            let score = -opponent_score;
            if score > best_move_score {
                best_move_score = score;
                best_move = Some(mov);
            }
        }

        (best_move, best_move_score)
    }
}

impl Engine for Anthony {
    fn new(color: Color, _: TimeControl) -> Self {
        Self {
            color,
            played_moves: HashSet::new(),
        }
    }

    fn get_action(&mut self, game: &Game) -> Action {
        tracing::debug!("Finding action");

        let (Some(mov), _score) = self.search_move(game.board(), self.color, Self::SEARCH_DEPTH)
        else {
            panic!("Didn't find a single move?");
        };

        self.played_moves.insert(mov);

        Action::Move(mov)
    }

    fn accept_draw_offer(&mut self, _: &Game) -> bool {
        false
    }
}
