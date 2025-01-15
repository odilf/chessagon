use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct TimeControl {
    base_time: Duration,
    increment: Duration,
}

impl TimeControl {
    pub fn max() -> Self {
        TimeControl {
            base_time: Duration::from_secs(u64::MAX),
            increment: Duration::from_secs(u64::MAX),
        }
    }
}
