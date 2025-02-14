use std::collections::HashSet;

use chessagon_core::{
    Board, Color, Move,
    game::{Action, Game, TimeControl},
};

use crate::Engine;

/// Very simple chessagon engine, used as an example.
pub struct Anthony {
    color: Color,
    played_moves: HashSet<Move>,
}

impl Anthony {
    pub const SEARCH_DEPTH: usize = 2;

    pub fn search_move(
        &mut self,
        board: &Board,
        color: Color,
        depth: usize,
    ) -> (Option<Move>, f64) {
        if depth == 0 {
            return (None, self.eval_for(board, color));
        }

        let mut best_move = None;
        let mut best_move_score = f64::NEG_INFINITY;
        for mov in board.possible_moves(color) {
            let mut board = board.clone();
            board.apply_move_unchecked(mov, color);

            let (_best_response, opponent_score) =
                self.search_move(&board, color.other(), depth - 1);

            let mut score = -opponent_score;
            if self.played_moves.contains(&mov) {
                score -= 50.0;
            }

            if score > best_move_score {
                best_move_score = score;
                best_move = Some(mov);
            }
        }

        if best_move.is_none() {
            for mov in board.possible_moves(color) {
                tracing::debug!("There is {mov}");
                let mut test_board = board.clone();
                test_board.apply_move_unchecked(mov, color);
                let eval = self.eval_for(&test_board, color);

                tracing::debug!("Evaluated at {eval}");
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
        let (Some(mov), _score) = self.search_move(game.board(), self.color, Self::SEARCH_DEPTH)
        else {
            unreachable!("If no moves are left, game should have been considered finished before.");
        };

        self.played_moves.insert(mov);

        Action::Move(mov)
    }

    fn accept_draw_offer(&mut self, _: &Game) -> bool {
        false
    }

    fn eval(&mut self, board: &Board) -> f64 {
        (board.total_piece_value(Color::White) as i16
            - board.total_piece_value(Color::Black) as i16
            - board.in_check(Color::White).is_some() as i16 * 100
            + board.in_check(Color::Black).is_some() as i16 * 200) as f64
    }
}
