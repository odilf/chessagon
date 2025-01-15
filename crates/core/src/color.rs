use core::fmt;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline]
    pub fn choose<T>(self, white: T, black: T) -> T {
        match self {
            Self::White => white,
            Self::Black => black
        }
    }

    pub fn direction(self) -> i8 {
        self.choose(1, -1)
    }

    pub fn other(self) -> Color {
        self.choose(Color::Black, Color::White)
    }
}

impl<T> Index<Color> for [T; 2] {
    type Output = T;
    fn index(&self, index: Color) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<Color> for [T; 2] {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.choose("white", "black"))
    }
}
