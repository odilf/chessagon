#![allow(missing_docs)]

use std::time::Duration;

use crate::Color;

/// Time constraints on chessagon games.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeControl {
    pub base_time: [Duration; 2],
    pub increment: [Duration; 2],
}

impl TimeControl {
    pub const fn new_asymetric(base_time: [Duration; 2], increment: [Duration; 2]) -> Self {
        Self {
            base_time,
            increment,
        }
    }

    pub const fn new(base_time: Duration, increment: Duration) -> Self {
        Self::new_asymetric([base_time; 2], [increment; 2])
    }

    pub const fn no_increment(base_time: Duration) -> Self {
        Self::new_asymetric([base_time; 2], [Duration::ZERO; 2])
    }

    /// 1+0
    pub const fn bullet() -> Self {
        Self::no_increment(Duration::from_secs(60))
    }

    /// 3+2
    pub const fn blitz() -> Self {
        Self::new(Duration::from_secs(60 * 3), Duration::from_secs(2))
    }

    /// 10+5
    pub const fn rapid() -> Self {
        Self::new(Duration::from_secs(60 * 10), Duration::from_secs(5))
    }

    pub const fn max() -> Self {
        Self::new_asymetric([Duration::MAX; 2], [Duration::MAX; 2])
    }

    pub fn canonical_duration(&self) -> Duration {
        let white = self.base_time[Color::White] + self.increment[Color::White] * 40;
        let black = self.base_time[Color::Black] + self.increment[Color::Black] * 40;

        (white + black) / 2
    }
}
