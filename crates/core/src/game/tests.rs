#![cfg(test)]

use super::*;

#[test]
fn fn_move_duration_returns_move_duration_for_moves_0_to_3() {
    let mut game = Game::new(TimeControl::rapid());

    assert_eq!(game.move_duration(0), None);

    let action = Action::Move(game.board().possible_moves(Color::White).next().unwrap());
    game.apply_action(action, Color::White).unwrap();

    assert_eq!(game.move_duration(0), Some(Duration::ZERO));

    let action = Action::Move(game.board().possible_moves(Color::Black).next().unwrap());
    game.apply_action(action, Color::Black).unwrap();

    let move_duration = Duration::from_millis(10);
    let action = Action::Move(game.board().possible_moves(Color::White).next().unwrap());
    std::thread::sleep(move_duration);
    game.apply_action(action, Color::White).unwrap();

    assert_eq!(game.move_duration(1), Some(Duration::ZERO));
    assert!(game.move_duration(2).unwrap() - move_duration <= Duration::from_millis(5));
}
