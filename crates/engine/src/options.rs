use chessagon_core::game::TimeControl;

#[derive(Debug, Clone, Copy)]
pub struct Options {
    // pub allowed_to_think_in_opponent_time: bool,
    pub time_control: TimeControl,
}
