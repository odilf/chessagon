#![cfg(test)]

use pretty_assertions::assert_eq;
use std::collections::HashSet;

use crate::diagrams;

use super::*;

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

#[test]
fn fn_is_valid_in_ivec2_returns_valid_for_every_vec2_diff_and_invalid_otherwise() {
    let mut visited = HashSet::new();
    for a in Vec2::iter() {
        for b in Vec2::iter() {
            let diff = a - b;
            assert!(IVec2::is_valid(diff.x, diff.y));
            visited.insert(diff);
        }
    }

    for x in i8::MIN + 1..=i8::MAX {
        for y in i8::MIN + 1..=i8::MAX {
            assert_eq!(
                IVec2::is_valid(x, y),
                visited.contains(&IVec2::new_unchecked(x, y))
            )
        }
    }
}
