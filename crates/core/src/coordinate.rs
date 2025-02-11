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

use std::{
    cmp::{max, min},
    fmt, ops,
};

use crate::piece::movement;

mod tests;

/// A vector in hexagonal coordinates, inside of the hexagonal chessboard.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vec2 {
    x: u8,
    y: u8,
}

/// Construct a [`Vec2`] that is checked to be valid at compile-time
#[macro_export]
macro_rules! vec2 {
    ($x:literal, $y:literal) => {{
        static_assertions::const_assert!($crate::Vec2::is_valid($x, $y));
        $crate::Vec2::new_unchecked($x, $y)
    }};
}

/// A valid differenece between two [`Vec2`]s.
///
/// This type is obtained by doing subtraction via [`ops::Sub`] on two [`Vec2`]s.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IVec2 {
    x: i8,
    y: i8,
}

/// Construct an [`IVec2`] that is checked to be valid at compile-time
#[macro_export]
macro_rules! ivec2 {
    ($x:literal, $y:literal) => {{
        static_assertions::const_assert!($crate::coordinate::IVec2::is_valid($x, $y));
        $crate::IVec2::new_unchecked($x, $y)
    }};
}

///////////////////////////////////////////////////////////////////////////////
/// Chessagon specific implementations
///////////////////////////////////////////////////////////////////////////////

impl Vec2 {
    /// The maximum value that a coordinate can have.
    pub const MAX: u8 = 10;

    /// The maximum allowed absolute difference of two coordinates.
    pub const WIDTH: u8 = 5;

    /// The maximum value for the rank of a tile.
    pub const MAX_RANK: u8 = 20;

    /// The maximum value for the file of a tile.
    pub const MAX_FILE: u8 = 10;

    pub const CENTER: Self = vec2!(5, 5);

    /// Whether these coordinates are possible in a chesssagon board.
    ///
    /// For the coordinates to be valid, the following conditions must be met:
    /// - `|x - y| <= WIDTH`
    /// - `x <= MAX` and `y <= MAX`
    pub const fn is_valid(x: u8, y: u8) -> bool {
        x.abs_diff(y) <= Self::WIDTH && x <= Self::MAX && y <= Self::MAX
    }

    /// Create a new vector.
    ///
    /// Returns `Err` if it's outside of the standard board.
    pub const fn new(x: u8, y: u8) -> Option<Self> {
        if !Self::is_valid(x, y) {
            return None;
        }

        Some(Self::new_unchecked(x, y))
    }

    /// Iterator over all valid hexagonal coordinates
    pub fn iter() -> impl Iterator<Item = Self> {
        (0_u8..=Self::MAX).flat_map(|x| {
            // There are two conditions for `y`:
            // 1. `0 <= y <= Self::MAX`
            // 2. `|x - y| <= Self::WIDTH`

            let lower = x.saturating_sub(Self::WIDTH);
            let upper = (x + Self::WIDTH).min(Self::MAX);
            (lower..=upper).map(move |y| Self::new_unchecked(x, y))
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
        (self.x + self.y) % 3
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
        self.x + self.y
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
        5 + self.y - self.x
    }

    /// The corresponding vector from the other side of the board.
    #[inline]
    pub const fn flipped(self) -> Self {
        Vec2::new_unchecked(Self::MAX - self.x, Self::MAX - self.y)
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
        min(min(rank, Self::MAX_RANK - rank) / 2, 2) * 2 + 1 + rank % 2
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

    /// The smallest number of adjacent tiles you have to traverse in order to go from `self`
    /// to `other`.
    ///
    /// See also [`IVec2::length`].
    #[inline]
    pub fn distance(self, other: Vec2) -> u8 {
        (other - self).length()
    }
}

impl IVec2 {
    /// Whether these coordinates are possible in a chesssagon board.
    ///
    /// The conditions are:
    /// - `|dx| <= MAX` and `|dy| <= MAX`
    /// - `|dx - dy| <= 2 * WIDTH`
    ///
    /// ## Derivation
    ///
    /// We are given two values, `dx` and `dy`, and we want to verify if these values can be produced as the difference between two [valid `Vec2`s](Vec2::is_valid).
    ///
    /// A valid [`Vec2`] has the properties:
    /// - `|x - y| <= WIDTH`
    /// - `x <= MAX` and `y <= MAX`
    ///
    /// We need to find whether there are any set of two vectors `(x1, y1)` and `(x2, y2)` such that they satisfy those conditions, and the extra
    /// `x1 - x2 == dx` and `y1 - y2 == dy`.
    ///
    /// Throughout the derivation, the operations are going to be done with `x` but they can all be aplied also to `y`
    ///
    /// First we start with some rearranging to get `x1` and `x2`:
    ///
    /// ```txt
    ///    x1 - x2 ==  dx
    /// -> x1      ==  dx + x2
    /// ->      x2 == -dx + x1
    /// ```
    ///
    /// Now we can apply the [validity conditions](Vec2::is_valid) on `x1` and `x2`.
    ///
    /// 1. Condition `x <= MAX`
    ///
    /// First, with `x1`
    /// ```txt
    ///         x1 <= MAX
    /// -> dx + x2 <= MAX
    /// ->      x2 <= MAX - dx
    /// ```
    ///
    /// Since `x2` has to be nonnegative then `0 <= x2 <= MAX - dx` which in turn implies that `dx <= MAX`.
    ///
    /// We also have the same condition with `x2`
    /// ```txt
    ///          x2 <= MAX
    /// -> -dx + x1 <= MAX
    /// ->       x1 <= MAX + dx
    /// ```
    ///
    /// Which implies similarly as before that `-dx <= MAX`.
    ///
    /// These two conditions can be merged into the simpler **`|dx| <= MAX`** (and also `|dy| <= MAX` by symmetry of `x` and `y`).
    ///
    /// 2. Condition `|x - y| <= WIDTH`
    ///
    /// First with `x1` and `y1`:
    ///
    /// ```txt
    ///    |    x1    -     y1   | <= WIDTH
    /// -> |(dx + x2) - (dy + y2)| <= WIDTH
    /// -> | dx - dy  +  x2 - y2 | <= WIDTH
    /// ```
    ///
    /// We know, since `v2` should be well-formed, that `|x2 - y2| <= WIDTH` which is equivalent to `-WIDTH <= x2 - y2 <= WIDTH`.
    ///
    /// The condition above can also be reformulated to
    /// ```txt
    ///
    ///       |dx - dy + x2 - y2| <= WIDTH
    /// ->    -WIDTH <= dx - dy + x2 - y2 <= WIDTH
    /// ->    -WIDTH - (dx - dy) <= x2 - y2 <= WIDTH - (dx - dy)
    ///
    /// given -WIDTH <= x2 - y2 <= WIDTH:
    ///
    /// -> -WIDTH - (dx - dy) <= x2 - y2 <= WIDTH
    /// -> -WIDTH - (dx - dy) <= WIDTH
    /// -> -2 * WIDTH <= dx - dy
    ///
    /// and
    ///
    /// -> -WIDTH <= x2 - y2 <= WIDTH - (dx - dy)
    /// -> -WIDTH <= WIDTH - (dx - dy)
    /// -> dx - dy <= 2 * WIDTH
    /// ```
    ///
    /// These two results can be combined into `-2 * WIDTH <= dx - dy <= 2 * WIDTH` or just **`|dx - dy| <= 2 * WIDTH`**
    ///
    /// Doing this with `x2` and `y2` leads to the same result, so we have obtained both conditions for a valid [`IVec2`]!
    pub const fn is_valid(dx: i8, dy: i8) -> bool {
        dx.unsigned_abs() <= Vec2::MAX
            && dy.unsigned_abs() <= Vec2::MAX
            && dx.abs_diff(dy) <= 2 * Vec2::WIDTH
    }

    /// The smallest number of adjacent tiles you have to traverse in order to go from `self`
    /// to `other`.
    ///
    /// See also [`Vec2::distance`].
    pub fn length(&self) -> u8 {
        // Adjecent moves are either (1, 0), (0, 1) or (1, 1).
        //
        // There are two possible cases:
        //
        // 1. The sign of `x` is different to the sign of `y`: In this scenario, there are no
        // (1, 1) components, since adding that would make the resulting position be closer to one
        // coordinate and further from the other, so you would still need the same amount of moves
        // after that to reach the target which means you would just be adding 1 length with no reason,
        // which means the path would no longer be optimal.
        //
        // Therefore in this case, the result is the sum of the absolute values.
        //
        // 2. The sign of `x` and `y` are the same. In this scenario, there is some amount of movement `u`
        // in the (1, 1) direction and some `v` in the (0, 1) or (1, 0) (where both `u` and `v` can be negative).
        // Then, the vector would be decomposable as `(u + v, u)` or `(u, u + v)`, so the total distance (i.e.,
        // `u + v`) would be the max of the two coordinates.

        if self.x.signum() == self.y.signum() {
            max(self.x.abs() as u8, self.y.abs() as u8)
        } else {
            // Since `x` and `y` have different signs in this case, we might aswell take the absolute difference?
            // I feel this may be faster, but I feel even more that this is not going to even be measurable.
            // Alternative: `self.x.abs() as u8 + self.y.abs() as u8`.
            self.x.abs_diff(self.y)
        }
    }

    /// Iterator over all valid hexagonal coordinate differences.
    pub fn iter() -> impl Iterator<Item = Self> {
        (-(Vec2::MAX as i8)..=Vec2::MAX as i8).flat_map(|dx| {
            // Conditions are:
            // - `-MAX <= dy <= MAX`
            // - `-2 * WIDTH <= dx - dy <= 2 * WIDTH`
            // So:
            // - `-2 * WIDTH - dx <= -dy <= 2 * WIDTH + dx`
            // - `-2 * WIDTH - dx <= dy <= 2 * WIDTH + dx`
            let lower = max(-2 * Vec2::WIDTH as i8 - dx, -(Vec2::MAX as i8));
            let upper = min(2 * Vec2::WIDTH as i8 + dx, Vec2::MAX as i8);

            (lower..=upper).map(move |dy| IVec2::new_unchecked(dx, dy))
        })
    }

    /// Iterator over all possible strides (i.e., values you could get out of
    /// [`crate::piece::movement::get_stride`]).
    pub fn strides() -> impl Iterator<Item = Self> {
        Self::iter().filter_map(|delta| {
            if delta == IVec2::ZERO {
                None
            } else {
                let (stride, 1) = movement::get_stride(delta) else {
                    return None;
                };
                Some(stride)
            }
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Math and operations
///////////////////////////////////////////////////////////////////////////////

impl ops::Sub<Vec2> for Vec2 {
    type Output = IVec2;
    fn sub(self, rhs: Vec2) -> Self::Output {
        IVec2 {
            x: self.x as i8 - rhs.x as i8,
            y: self.y as i8 - rhs.y as i8,
        }
    }
}

impl ops::Add<IVec2> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: IVec2) -> Self::Output {
        Vec2 {
            x: self.x.wrapping_add(rhs.x as u8),
            y: self.y.wrapping_add(rhs.y as u8),
        }
    }
}

macro_rules! impl_generic_vec {
    (
        impl Vec2 {
            $($inherent_implementation:tt)*
        }


        $($implementation:tt)*
    ) => {
        mod __vec2_generic_impls {
            use super::*;
            type T = u8;


            impl Vec2 {
                $($inherent_implementation)*
            }

            $($implementation)*
        }

        mod __ivec2_generic_impls {
            use super::*;
            type T = i8;
            type Vec2 = IVec2;

            macro_rules! vec2 { ($x:tt, $y:tt) => { ivec2!($x, $y) } }

            impl IVec2 {
                $($inherent_implementation)*
            }

            $($implementation)*
        }
   }
}

impl_generic_vec! {
    impl Vec2 {
        #[inline(always)]
        pub const fn new_unchecked(x: T, y: T) -> Self {
            Self { x, y }
        }

        pub const ZERO: Self = vec2!(0, 0);

        #[inline(always)]
        pub const fn x(&self) -> T {
            self.x
        }

        #[inline(always)]
        pub const fn y(&self) -> T {
            self.y
        }
    }


    impl ops::Mul<T> for Vec2
    where
        T: ops::Mul<T, Output = T>,
    {
        type Output = Vec2;
        fn mul(self, rhs: T) -> Self::Output {
            Vec2::new_unchecked(self.x * rhs, self.y * rhs)
        }
    }

    impl ops::Div<T> for Vec2
    where
        T: Copy + ops::Div<T, Output = T>,
    {
        type Output = Vec2;
        fn div(self, rhs: T) -> Self::Output {
            Vec2::new_unchecked(self.x / rhs, self.y / rhs)
        }
    }

    impl fmt::Display for Vec2 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "({}, {})", self.x, self.y)
        }
    }
}
