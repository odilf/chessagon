use core::fmt;
use std::{
    cmp::Ordering,
    ops::{Index, IndexMut},
};

use crate::coordinate::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline]
    pub const fn choose<T: Copy>(self, white: T, black: T) -> T {
        match self {
            Self::White => white,
            Self::Black => black,
        }
    }

    /// The direction towards the center from the given color. `1` for white, `-1` for black.
    ///
    /// Analogous to "signum", but the color.
    #[inline]
    pub const fn direction(self) -> i8 {
        self.choose(1, -1)
    }

    #[inline]
    pub fn other(self) -> Color {
        self.choose(Color::Black, Color::White)
    }

    /// Compares whether `a` is closer to the color than `b`.
    ///
    /// If color is white it returns `a.cmp(b)` else `b.cmp(a)`.
    #[inline]
    pub fn compare_towards(&self, a: u8, b: u8) -> Ordering {
        match self {
            Color::White => a.cmp(&b),
            Color::Black => b.cmp(&a),
        }
    }
}

impl<T> Index<Color> for [T; 2] {
    type Output = T;
    fn index(&self, index: Color) -> &Self::Output {
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<Color> for [T; 2] {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.choose("white", "black"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    King,
    Queen,
}

impl Side {
    #[inline]
    pub fn choose<T>(self, king: T, queen: T) -> T {
        match self {
            Self::King => king,
            Self::Queen => queen,
        }
    }

    pub fn direction(self) -> i8 {
        self.choose(1, -1)
    }

    pub fn other(self) -> Self {
        self.choose(Self::Queen, Self::King)
    }

    /// Makes a step of size `step_size` towards the given side.
    // TODO: Unit test this function
    pub const fn step_towards(&self, step_size: i8) -> Vec2<i8> {
        let x_axis = matches!(*self, Side::Queen) ^ (step_size < 0);
        match x_axis {
            true => Vec2::new_unchecked(step_size, 0),
            false => Vec2::new_unchecked(0, step_size),
        }
    }
}

impl<T> Index<Side> for [T; 2] {
    type Output = T;
    fn index(&self, index: Side) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<Side> for [T; 2] {
    fn index_mut(&mut self, index: Side) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.choose("king's", "queen's"))
    }
}
