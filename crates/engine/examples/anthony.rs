use chessagon_core::game::TimeControl;
use chessagon_engine::{matcher::match_engines, models::Anthony};

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting anthony vs anthony match");
    match_engines::<Anthony, Anthony>(TimeControl::max());
}
