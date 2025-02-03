//! Hexagonal coordinates.
//!
//! Coordinates in hexagons are a bit tricky. Using cartesian coordinates is really not practical,
//! so generally we would like to use a different basis. The problem is that there is no useful
//! perpendicual basis, and hexagons always have this 3-way symmetry, given that you have neighbors
//! in 3 distinct directions (each direction has two neighbors, so total of 6 neighbors). This
//! makes it fundamentally impossible to have a completely analogous system to regular cartesian
//! coordinates.
//!
//! So, how can we describe the neighbors? Instead of being at directions `(0,1)` and `(1,0)`, they
//! should be at `(0,1)`, `(1,0)` and `(1,1)`. From the center hexagon, each should be one of the
//! neighbours. Technically you can choose any combination, but the most reasonable is probably
//! to say that `(1,0)` is the left one, `(1,1)` is the center, and `(0,1)` the right.
//!
//! There is an other alternative to this system. You see that `(1,1)` is straight vertical. So...
//! why not make that `(0,1)`? and then find the straight horizontal, which turns out to be `(1,-1)` and make that `(1,0)`.
//! This sounds nice because it's easier to think about, but the problem then is that the two
//! neighbors are at `1/√2` distances, which are very annoying. TODO: Is this necessarly true?

use core::fmt;
use std::fmt::{Debug, Display};
use std::ops::{self, Deref, DerefMut};

use std::hash::Hash;

use nalgebra::{Scalar, coordinates::XY};

/// A vector in hexagonal coordinates, inside of the hexagonal chessboard.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Vec2<T = u8>(nalgebra::Vector2<T>);

impl<T: Debug + Copy + Display> Debug for Vec2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "Vec2(\n    {},\n    {}\n)", self.x(), self.y())
        } else {
            write!(f, "Vec2({},{})", self.x(), self.y())
        }
    }
}

impl<T: Hash + Scalar> Hash for Vec2<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: Scalar> Deref for Vec2<T> {
    type Target = XY<T>;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: Scalar> DerefMut for Vec2<T> {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        self.0.deref_mut()
    }
}

// Generic, non-hexagonal, implementations
impl<T> Vec2<T> {
    /// const-compatible way of doing `self.x`
    pub const fn x(&self) -> T
    where
        T: Copy,
    {
        self.0.data.0[0][0]
    }

    /// const-compatible way of doing `self.y`
    pub const fn y(&self) -> T
    where
        T: Copy,
    {
        self.0.data.0[0][1]
    }

    /// Creates a new vector in hexagonal basis.
    ///
    /// Doesn't check if it's a valid hexagonal chess position.
    #[inline]
    pub const fn new_unchecked(x: T, y: T) -> Self {
        let vec = nalgebra::Vector2::new(x, y);
        Self(vec)
    }

    // pub fn map<U>(self, f: impl Fn(T) -> U) -> Vec2<U>
    // where
    //     T: Copy,
    // {
    //     Vec2::new_unchecked(f(self.x()), f(self.y()))
    // }
}

impl Vec2 {
    /// The maximum value that a coordinate can have.
    pub const MAX: u8 = 10;

    /// The maximum allowed absolute difference of two coordinates.
    pub const WIDTH: u8 = 5;

    /// The maximum value for the rank of a tile.
    pub const MAX_RANK: u8 = 20;

    /// The maximum value for the file of a tile.
    pub const MAX_FILE: u8 = 10;

    /// Create a new vector.
    ///
    /// Returns `Err` if it's outside of the standard board.
    pub const fn new(x: u8, y: u8) -> Result<Self, ()> {
        if x.abs_diff(y) > Self::WIDTH || x > Self::MAX || y > Self::MAX {
            return Err(());
        }

        Ok(Self::new_unchecked(x, y))
    }

    /// Iterator over all valid hexagonal coordinates
    pub fn iter() -> impl Iterator<Item = Self> {
        (0_u8..=Self::MAX).flat_map(|x| {
            (0..=Self::MAX).filter_map(move |y| {
                if x.abs_diff(y) > Self::WIDTH {
                    return None;
                }

                Some(Self::new_unchecked(x, y))
            })
        })
    }

    /// The "color" of the tile. Always one of 0, 1 or 2.
    ///
    /// Two touching tiles will never have the same index.
    ///
    /// Bishops can only travel to tiles with the same index as the one they're on.
    ///
    /// Computed as the sum of the coordinates, mod 3.
    pub const fn index(&self) -> u8 {
        (self.x() + self.y()) % 3
    }

    /// The hexagonal analogous to a row.
    ///
    /// Visually:
    ///
    /// ```text
    #[doc = include_str!("./diagrams/ranks.txt")]
    /// ```
    ///
    /// Computed as the sum of the coordinates.
    #[inline]
    pub const fn rank(&self) -> u8 {
        self.x() + self.y()
    }

    /// The hexagonal analogous to a column.
    ///
    /// Visually:
    ///
    /// ```text
    #[doc = include_str!("./diagrams/files.txt")]
    /// ```
    ///
    /// Computed as the difference of the coordinates, plus 5.
    #[inline]
    pub const fn file(&self) -> u8 {
        // The 5 has to be at the start to avoid an underflow error.
        5 + self.y() - self.x()
    }

    /// The corresponding vector from the other side of the board.
    #[inline]
    pub const fn flipped(self) -> Self {
        Vec2::new_unchecked(Self::MAX - self.x(), Self::MAX - self.y())
    }

    /// Number of valid coordinates with a given rank
    ///
    /// Visually:
    ///
    /// ```text
    #[doc = include_str!("./diagrams/rank_widths.txt")]
    /// ```
    pub fn rank_width(rank: u8) -> u8 {
        // The conditions for a valid coordinate are:
        // 1. `y <= Vec2::MAX && x <= Vec2::MAX`
        // 2. `x.abs_diff(y) <= Vec2::WIDTH`
        //
        // We have `rank = x + y`, so
        //
        // TODO: Explain the reasoning of this function
        (rank.min(Self::MAX_RANK - rank) / 2).min(2) * 2 + 1 + rank % 2
        // let max_valid_rank_coordinate = rank - 1;
        // max_valid_rank_coordinate - Vec2::min_valid_rank_coordinate(rank)
    }

    /// Retuns the lowest value that a coordinate can have on a given rank.
    pub fn min_valid_rank_coordinate(rank: u8) -> u8 {
        // The conditions for a valid coordinate are:
        // 1. `y <= Vec2::MAX && x <= Vec2::MAX`
        // 2. `x.abs_diff(y) <= Vec2::WIDTH`
        //
        // We know that `rank = x + y`, so we can do some algebraic manipulation with to get
        // - x + y == rank
        // - Vec2::MAX + y >= rank
        // - y >= rank - Vec2::MAX
        //
        // And then with the second condition:
        // - x + y == rank
        // - x - y == rank - 2y
        // - Vec2::WIDTH => rank - 2y
        // - 2y => rank - Vec2::WIDTH
        // - y => (rank - Vec2::WIDTH) / 2.
        //
        // So the first valid y is 0 unless it is overriden by one of those conditions.

        let h = rank as i8 - Vec2::MAX as i8;
        // The +1 is necessary because the condition would be `y >= something.5`
        let w = (rank as i8 - Vec2::WIDTH as i8 + 1) / 2;

        h.max(w).max(0) as u8
    }
}

impl Vec2<i8> {
    #[allow(missing_docs)]
    pub const ZERO: Self = Vec2::new_unchecked(0, 0);
}

impl ops::Sub<Vec2> for Vec2 {
    type Output = Vec2<i8>;
    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2(self.0.cast() - rhs.0.cast())
    }
}

impl ops::Add<Vec2> for Vec2 {
    type Output = Vec2<u8>;
    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2(self.0 + rhs.0)
    }
}

impl ops::Add<Vec2<i8>> for Vec2 {
    type Output = Vec2<u8>;
    fn add(self, rhs: Vec2<i8>) -> Self::Output {
        Vec2::new_unchecked(
            self.x.wrapping_add(rhs.x as u8),
            self.y.wrapping_add(rhs.y as u8),
        )
    }
}

impl<T> ops::Mul<T> for Vec2<T>
where
    T: Copy + ops::Mul<T, Output = T>,
{
    type Output = Vec2<T>;
    fn mul(self, rhs: T) -> Self::Output {
        Vec2::new_unchecked(self.x() * rhs, self.y() * rhs)
    }
}

impl<T> ops::Div<T> for Vec2<T>
where
    T: Copy + ops::Div<T, Output = T>,
{
    type Output = Vec2<T>;
    fn div(self, rhs: T) -> Self::Output {
        Vec2::new_unchecked(self.x() / rhs, self.y() / rhs)
    }
}

// We could relax the `T: Copy` requirement and add `self.x_ref()` methods, but that's unnecessary
// because we only ever use simple numeric types for `T`.
impl<T: fmt::Display + Copy> fmt::Display for Vec2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    use crate::diagrams;

    use super::Vec2;

    #[test]
    fn rank_is_between_0_and_max() {
        let mut ranks = (0..=Vec2::MAX_RANK).collect::<HashSet<_>>();
        for position in Vec2::iter() {
            ranks.remove(&position.rank());
        }

        assert!(ranks.is_empty())
    }

    #[test]
    fn rank_width_matches_manual_impl() {
        let manual_rank = |rank| match rank {
            0 => 1,
            1 => 2,
            2 => 3,
            3 => 4,
            4 => 5,
            5 => 6,
            6 => 5,
            7 => 6,
            8 => 5,
            9 => 6,
            10 => 5,
            11 => 6,
            12 => 5,
            13 => 6,
            14 => 5,
            15 => 6,
            16 => 5,
            17 => 4,
            18 => 3,
            19 => 2,
            20 => 1,
            _ => panic!("Invalid rank {rank}"),
        };

        for rank in 0..=Vec2::MAX_RANK {
            assert_eq!(manual_rank(rank), Vec2::rank_width(rank));
        }
    }

    #[test]
    fn file_is_between_0_and_10() {
        let mut ranks = (0..=10).collect::<HashSet<_>>();
        for position in Vec2::iter() {
            ranks.remove(&position.file());
        }

        assert!(ranks.is_empty())
    }

    #[test]
    fn files_match_diagram() {
        let rendered = diagrams::visualize_tile_property(
            |position| position.file(),
            |file| char::from_digit(*file as u32, 16).unwrap(),
        );

        assert_eq!(rendered.trim(), diagrams::FILES.trim());
    }

    #[test]
    fn ranks_match_diagram() {
        let rendered = diagrams::visualize_tile_property(
            |position| position.rank(),
            |rank| char::from_digit(*rank as u32, 36).unwrap(),
        );

        assert_eq!(rendered.trim(), diagrams::RANKS.trim());
    }

    #[test]
    fn rank_widths_match_diagram() {
        let rendered = diagrams::visualize_tile_property(
            |position| Vec2::rank_width(position.rank()),
            |width| char::from_digit(*width as u32, 16).unwrap(),
        );

        assert_eq!(rendered.trim(), diagrams::RANK_WIDTHS.trim());
    }

    #[test]
    fn min_valid_rank_coordinates_match_diagram() {
        let rendered = diagrams::visualize_tile_property(
            |position| Vec2::min_valid_rank_coordinate(position.rank()),
            |width| char::from_digit(*width as u32, 16).unwrap(),
        );

        assert_eq!(rendered.trim(), diagrams::MIN_VALID_RANK_COORDINATES.trim());
    }
}
