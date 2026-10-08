use crate::board::path::{Direction::*, Path};

use std::fmt;

use Piece::*;

/// Pentominoes keep their usual letter. Tetrominoes use their Tetris letter,
/// except the L-tetromino which is named J to avoid clashing with the L-pentomino
/// (both mirror images are allowed, so J and L are the same piece).
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[repr(u8)]
pub enum Piece {
    J, L, O, P, S, T, U, X, Z,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-piece", match self {
            J => 'J', L => 'L', O => 'O', P => 'P', S => 'S',
            T => 'T', U => 'U', X => 'X', Z => 'Z',
        })
    }
}

impl Piece {
    pub const COUNT: usize = 9;

    pub fn pieces() -> [Piece; Piece::COUNT] {
        [J, L, O, P, S, T, U, X, Z]
    }

    /// Number of quarter turns after which the piece looks the same (4 means it never changes).
    pub fn rotational_symmetry(&self) -> u8 {
        match self {
            O | X => 4,
            S | Z => 2,
            _ => 1,
        }
    }

    /// Whether the mirror image of the piece is one of its rotations.
    pub fn mirror_symmetric(&self) -> bool {
        matches!(self, O | T | U | X)
    }

    pub fn square_count(&self) -> usize {
        match self {
            J | O | S | T => 4,
            L | P | U | X | Z => 5,
        }
    }
}

/// Each path starts on the piece's anchor and may walk back over a square already visited.
impl From<Piece> for Path {
    fn from(piece: Piece) -> Self {
        let directions = match piece {
            J => vec![Down, Down, Right],
            L => vec![Down, Down, Down, Right],
            O => vec![Right, Down, Left],
            P => vec![Up, Up, Right, Down],
            S => vec![Down, Right, Down],
            T => vec![Down, Right, Left, Down],
            U => vec![Down, Right, Right, Up],
            X => vec![Down, Left, Right, Right, Left, Down],
            Z => vec![Right, Down, Down, Right],
        };
        Path::from(directions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::path::Direction;
    use crate::board::placement::Rotation;

    use std::collections::BTreeSet;

    type Shape = BTreeSet<(i32, i32)>;

    fn shape(path: &Path) -> Shape {
        let mut position = (0, 0);
        let mut cells = Shape::from([position]);
        for direction in path.iter() {
            position = match direction {
                Direction::Up    => (position.0 - 1, position.1),
                Direction::Down  => (position.0 + 1, position.1),
                Direction::Left  => (position.0, position.1 - 1),
                Direction::Right => (position.0, position.1 + 1),
            };
            cells.insert(position);
        }
        normalize(cells)
    }

    fn normalize(cells: Shape) -> Shape {
        let min_row = cells.iter().map(|cell| cell.0).min().unwrap();
        let min_col = cells.iter().map(|cell| cell.1).min().unwrap();
        cells.into_iter().map(|(row, col)| (row - min_row, col - min_col)).collect()
    }

    fn all_orientations(piece: Piece) -> BTreeSet<Shape> {
        let mut shapes = BTreeSet::new();
        for mirror in [false, true] {
            for rotation in 0..4 {
                shapes.insert(shape(&Path::from_orientation(piece, Rotation::from(rotation), mirror)));
            }
        }
        shapes
    }

    fn generated_orientations(piece: Piece) -> Vec<Shape> {
        let mirrors: &[bool] = if piece.mirror_symmetric() { &[false] } else { &[false, true] };
        let mut shapes = Vec::new();
        for &mirror in mirrors {
            for rotation in Rotation::all_by_symmetry(piece.rotational_symmetry()) {
                shapes.push(shape(&Path::from_orientation(piece, rotation, mirror)));
            }
        }
        shapes
    }

    #[test]
    fn paths_cover_the_declared_number_of_squares() {
        for piece in Piece::pieces() {
            assert_eq!(shape(&Path::from(piece)).len(), piece.square_count(), "{piece}");
        }
    }

    #[test]
    fn declared_symmetries_generate_each_orientation_exactly_once() {
        for piece in Piece::pieces() {
            let generated = generated_orientations(piece);
            let distinct: BTreeSet<Shape> = generated.iter().cloned().collect();
            assert_eq!(distinct.len(), generated.len(), "{piece} generates duplicate orientations");
            assert_eq!(distinct, all_orientations(piece), "{piece} misses some orientations");
        }
    }

    #[test]
    fn pieces_are_all_different() {
        let pieces = Piece::pieces();
        for (i, &first) in pieces.iter().enumerate() {
            for &second in &pieces[i + 1..] {
                assert!(all_orientations(first).is_disjoint(&all_orientations(second)), "{first} and {second} are the same piece");
            }
        }
    }

    #[test]
    fn pieces_cover_the_frame_but_two_squares() {
        let covered: usize = Piece::pieces().iter().map(Piece::square_count).sum();
        assert_eq!(covered, 43 - 2);
    }
}
