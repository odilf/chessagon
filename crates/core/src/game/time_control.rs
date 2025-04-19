#![allow(missing_docs)]

use std::{fmt, time::Duration};

use crate::Color;

/// Time constraints on chessagon games.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Returns the base time of the time control, if it's symmetric.
    ///
    /// If each player has a different base time, it returns `None`. You have to get
    /// it with field access in that case.
    pub fn base_time(&self) -> Option<Duration> {
        if self.base_time[0] != self.base_time[1] {
            None
        } else {
            Some(self.base_time[0])
        }
    }

    /// Similar to [`Self::base_time`], but for increment.
    pub fn increment(&self) -> Option<Duration> {
        if self.increment[0] != self.increment[1] {
            None
        } else {
            Some(self.increment[0])
        }
    }

    pub const fn new(base_time: Duration, increment: Duration) -> Self {
        Self::new_asymetric([base_time; 2], [increment; 2])
    }

    pub const fn no_increment(base_time: Duration) -> Self {
        Self::new_asymetric([base_time; 2], [Duration::ZERO; 2])
    }

    /// Construct a time control from format "m+s" (**m**inutes **p**lus **s**econds)
    pub const fn mps(base_time_minutes: u64, increment_seconds: u64) -> Self {
        Self::new(
            Duration::from_secs(base_time_minutes * 60),
            Duration::from_secs(increment_seconds),
        )
    }

    /// 1+0
    pub const fn bullet() -> Self {
        Self::mps(1, 0)
    }

    /// 3+2
    pub const fn blitz() -> Self {
        Self::mps(3, 2)
    }

    /// 10+5
    pub const fn rapid() -> Self {
        Self::mps(10, 5)
    }

    pub const fn max() -> Self {
        Self::new_asymetric([Duration::MAX; 2], [Duration::MAX; 2])
    }

    /// Some measure of an "average" duration of a game (but not actually statistically average).
    ///
    /// It is defined as the maximum time for a game that took 40 moves.
    pub fn canonical_duration(&self) -> Duration {
        let white = self.base_time[Color::White] + self.increment[Color::White] * 40;
        let black = self.base_time[Color::Black] + self.increment[Color::Black] * 40;

        (white + black) / 2
    }

    pub fn formatted(&self) -> String {
        match (self.base_time(), self.increment()) {
            (Some(base_time), Some(increment)) => {
                format!("{}+{}", base_time.as_secs() / 60, increment.as_secs())
            }
            _ => todo!(),
        }
    }

    pub fn category(&self) -> Category {
        Category::classify(self.canonical_duration())
    }
}

/// Broad categories you can put [`TimeControl`]s in.
#[derive(Debug)]
pub enum Category {
    UltraBullet,
    Bullet,
    Blitz,
    Rapid,
    Classical,
}

impl Category {
    /// Classify a [`TimeControl::canonical_duration`].
    ///
    /// Follows [lichess' rules](https://lichess.org/faq#time-controls).
    pub fn classify(duration: Duration) -> Self {
        match duration.as_secs() {
            0..30 => Self::UltraBullet,
            30..180 => Self::Bullet,
            180..480 => Self::Blitz,
            480..1500 => Self::Rapid,
            1500.. => Self::Classical,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UltraBullet => f.write_str("UltraBullet"),
            Self::Bullet => f.write_str("Bullet"),
            Self::Blitz => f.write_str("Blitz"),
            Self::Rapid => f.write_str("Rapid"),
            Self::Classical => f.write_str("Classical"),
        }
    }
}
