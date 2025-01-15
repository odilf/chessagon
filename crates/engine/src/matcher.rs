use crate::{Engine, options::Options};
use chessagon_core::{
    Color,
    board::Board,
    game::{Game, GameResult, TimeControl},
};

pub fn match_engines_from_position<White: Engine, Black: Engine>(
    board: Board,
    time_control: TimeControl,
) -> GameResult {
    let mut game = Game::from_position(board, time_control);

    let mut white = White::new(Color::White, time_control);
    let mut black = Black::new(Color::Black, time_control);

    let mut players: [&mut dyn Engine; 2] = [&mut white, &mut black];

    let result = loop {
        tracing::debug!("Board state: \n{}", game.board());
        if let Some(result) = game.result() {
            break result;
        };

        let action = players[game.turn()].get_action(&game);
        tracing::debug!("{}: {action}", game.turn());

        if let Err(apply_action_err) = game.apply_action(action, game.turn()) {
            tracing::debug!("Action was invalid: {apply_action_err}");
        }
    };

    result
}
pub fn match_engines<White: Engine, Black: Engine>(time_control: TimeControl) -> GameResult {
    match_engines_from_position::<White, Black>(Board::default(), time_control)
}
