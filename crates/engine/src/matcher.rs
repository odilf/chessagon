use crate::Engine;
use chessagon_core::{
    Color,
    board::Board,
    game::{Game, TimeControl},
};

pub fn match_engines_from_position<White: Engine, Black: Engine>(
    board: Board,
    time_control: TimeControl,
) -> Game {
    let mut game = Game::from_position(board, time_control);

    let mut white = White::new(Color::White, time_control);
    let mut black = Black::new(Color::Black, time_control);

    let mut players: [&mut dyn Engine; 2] = [&mut white, &mut black];

    loop {
        tracing::debug!("Board state: \n{}", game.board());
        if game.result().is_some() {
            break;
        };

        let action = players[game.turn()].get_action(&game);
        tracing::debug!("{}: {action}", game.turn());

        if let Err(apply_action_err) = game.apply_action(action, game.turn()) {
            tracing::debug!("Action was invalid: {apply_action_err}");
        }
    }

    game
}
pub fn match_engines<White: Engine, Black: Engine>(time_control: TimeControl) -> Game {
    match_engines_from_position::<White, Black>(Board::default(), time_control)
}
