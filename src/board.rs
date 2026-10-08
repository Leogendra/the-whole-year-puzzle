pub mod path;
pub mod square;
pub mod piece;
pub mod placement;
pub mod compact;

pub use self::path::*;
pub use self::square::*;
pub use self::piece::*;
pub use self::placement::*;

use Status::*;

use std::fmt;

/// The drawn frame follows the original puzzle: its top-right corner is cut out,
/// so the first rows are one column narrower than the others.
const NOTCH_ROWS: usize = 2;
const NOTCH_COLUMNS: usize = 1;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
pub struct Board([Status; GRID_SIZE]);

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
enum Status {
    Empty,
    Nonexistent,
    Occupied(Piece),
}

impl Default for Board {
    fn default() -> Self {
        let mut status = [Empty; GRID_SIZE];
        for (index, cell) in status.iter_mut().enumerate() {
            if Square::try_from(index as u8).is_err() {
                *cell = Nonexistent;
            }
        }
        Board(status)
    }
}

impl Board {
    pub fn place(&self, piece: Piece, start: Square, path: &Path) -> Option<Self> {
        let mut status = self.0;
        let mut square = start;

        match self.0[square as usize] {
            Empty => status[square as usize] = Occupied(piece),
            _ => return None,
        }

        for dir in path.iter() {
            square = square.step(*dir)?;
            match self.0[square as usize] {
                Empty => status[square as usize] = Occupied(piece),
                _ => return None,
            }
        }

        Some(Board(status))
    }

    pub fn solved_for(&self) -> Option<Date> {
        let mut status = self.0.iter();

        let month = status.position(|&sq| sq == Empty)? as u8;
        let day = 1 + month + status.position(|&sq| sq == Empty)? as u8;

        let date = Date {
            month: Square::try_from(month).ok()?,
            day: Square::try_from(day).ok()?
        };

        match status.find(|&&sq| sq == Empty) {
            None if date.is_valid() => Some(date),
            _ => None,
        }
    }

    fn check_placement(&self, piece: Piece, start: Square) -> Option<(Rotation, bool)> {
        if self.0[start as usize] != Occupied(piece) {
            return None;
        }

        for &mirror in if piece.mirror_symmetric() { [false].iter() } else { [false, true].iter() } {
            for rotation in Rotation::all_by_symmetry(piece.rotational_symmetry()) {
                let path = Path::from_orientation(piece, rotation, mirror);
                let mut square = start;
                'walk: {
                    for &dir in path.iter() {
                        match square.step(dir) {
                            Some(new_square) if self.0[new_square as usize] == Occupied(piece) => { square = new_square; }
                            _ => { break 'walk; }
                        };
                    }
                    return Some((rotation, mirror));
                }
            }
        }

        None
    }

    /// Whether the nonexistent cells are exactly the holes of the frame.
    fn has_frame_holes(&self) -> bool {
        self.0.iter().enumerate().all(|(index, &status)| {
            (status == Nonexistent) == Square::try_from(index as u8).is_err()
        })
    }

    pub fn placements(&self) -> Result<Vec<Placement>, PlacementError> {
        if !self.has_frame_holes() {
            return Err(PlacementError);
        }

        let mut placed = [false; Piece::COUNT];
        for piece in Piece::pieces() {
            let num_squares = self.0.iter().filter(|&&status| status == Occupied(piece)).count();
            match num_squares {
                0 => {}
                n if n == piece.square_count() => { placed[piece as usize] = true; }
                _ => { return Err(PlacementError); }
            }
        }

        let mut placements = [None; Piece::COUNT];
        for square in Square::squares() {
            if let Occupied(piece) = self.0[square as usize] {
                if let (None, Some((rotation, mirror))) = (placements[piece as usize], self.check_placement(piece, square)) {
                    placements[piece as usize] = Some((square, rotation, mirror));
                }
            }
        }

        let mut result = Vec::new();
        for piece in Piece::pieces() {
            match (placed[piece as usize], placements[piece as usize]) {
                (true, Some((square, rotation, mirror))) => {
                    result.push(Placement { square, piece, rotation, mirror });
                }
                _ => { return Err(PlacementError); }
            }
        }

        Ok(result)
    }

    /// Status of the cell at (row, col), cells outside the grid being nonexistent.
    fn status_at(&self, row: Option<usize>, col: Option<usize>) -> Status {
        match (row, col) {
            (Some(row), Some(col)) if row < GRID_HEIGHT && col < GRID_WIDTH => self.0[col + GRID_WIDTH * row],
            _ => Nonexistent,
        }
    }

    /// Statuses of the four cells around the corner at the top left of cell (row, col),
    /// in the order: above left, above right, below left, below right.
    fn get_statuses_by_corner(&self, row: usize, col: usize) -> (Status, Status, Status, Status) {
        let (above, left) = (row.checked_sub(1), col.checked_sub(1));
        (
            self.status_at(above, left),
            self.status_at(above, Some(col)),
            self.status_at(Some(row), left),
            self.status_at(Some(row), Some(col)),
        )
    }

    /// Box-drawing character joining the borders that meet at a corner.
    /// A border separates two neighbouring cells with different statuses.
    fn corner_symbol(&self, row: usize, col: usize) -> char {
        let (above_left, above_right, below_left, below_right) = self.get_statuses_by_corner(row, col);
        let up = above_left != above_right;
        let down = below_left != below_right;
        let left = above_left != below_left;
        let right = above_right != below_right;

        match (up, down, left, right) {
            (false, false, false, false) => ' ',
            (true,  true,  false, false) => '│',
            (false, false, true,  true ) => '─',
            (false, true,  false, true ) => '┌',
            (false, true,  true,  false) => '┐',
            (true,  false, false, true ) => '└',
            (true,  false, true,  false) => '┘',
            (true,  true,  false, true ) => '├',
            (true,  true,  true,  false) => '┤',
            (false, true,  true,  true ) => '┬',
            (true,  false, true,  true ) => '┴',
            (true,  true,  true,  true ) => '┼',
            _ => unreachable!("a single border cannot end at a corner"),
        }
    }

    /// Whether a border runs along the top of cell (row, col).
    fn has_top_border(&self, row: usize, col: usize) -> bool {
        let (_, above_right, _, below_right) = self.get_statuses_by_corner(row, col);
        above_right != below_right
    }

    /// Whether a border runs along the left of cell (row, col).
    fn has_left_border(&self, row: usize, col: usize) -> bool {
        let (_, _, below_left, below_right) = self.get_statuses_by_corner(row, col);
        below_left != below_right
    }
}

impl Board {
    /// Number of grid columns inside the frame on a given row.
    fn frame_columns(row: usize) -> usize {
        if row < NOTCH_ROWS {
            GRID_WIDTH - NOTCH_COLUMNS
        } else {
            GRID_WIDTH
        }
    }

    /// Width between the two vertical lines of the frame when it encloses this many columns.
    fn frame_inner_width(columns: usize) -> usize {
        4 * columns + 3
    }

    fn write_border_line(&self, f: &mut fmt::Formatter<'_>, row: usize, columns: usize) -> fmt::Result {
        for col in 0..=columns {
            write!(f, "{}", self.corner_symbol(row, col))?;
            if col == columns {
                write!(f, " ")?;
            } else if self.has_top_border(row, col) {
                write!(f, "───")?;
            } else {
                write!(f, "   ")?;
            }
        }
        Ok(())
    }

    fn write_cell_line(&self, f: &mut fmt::Formatter<'_>, row: usize, columns: usize) -> fmt::Result {
        for col in 0..=columns {
            write!(f, "{}", if self.has_left_border(row, col) { '│' } else { ' ' })?;
            if col == columns {
                write!(f, " ")?;
            } else {
                let index = col + GRID_WIDTH * row;
                match (self.0[index], Square::try_from(index as u8)) {
                    (Empty, Ok(square)) => write!(f, "{:^3}", square)?,
                    _ => write!(f, "   ")?,
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let notch_width = Self::frame_inner_width(GRID_WIDTH - NOTCH_COLUMNS);
        let full_width = Self::frame_inner_width(GRID_WIDTH);

        writeln!(f, "╭{}╮", "─".repeat(notch_width))?;

        for row in 0..=GRID_HEIGHT {
            let columns = Self::frame_columns(row);

            write!(f, "│ ")?;
            self.write_border_line(f, row, columns)?;
            writeln!(f, "│")?;

            if row < GRID_HEIGHT {
                write!(f, "│ ")?;
                self.write_cell_line(f, row, columns)?;
                if row + 1 == NOTCH_ROWS {
                    writeln!(f, "╰{}╮", "─".repeat(full_width - notch_width - 1))?;
                } else {
                    writeln!(f, "│")?;
                }
            }
        }

        write!(f, "╰{}╯", "─".repeat(full_width))
    }
}

impl Board {
    pub fn to_mini_string(&self) -> String {
        let mut lines = Vec::new();

        for row in 0..=GRID_HEIGHT {
            let mut line = String::new();
            for col in 0..=GRID_WIDTH {
                line.push(self.corner_symbol(row, col));
                if col < GRID_WIDTH {
                    line.push(if self.has_top_border(row, col) { '─' } else { ' ' });
                }
            }
            lines.push(line);
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_board_has_the_frame_holes() {
        let board = Board::default();
        assert!(board.has_frame_holes());
        assert_eq!(board.0.iter().filter(|&&status| status == Empty).count(), 43);
    }

    #[test]
    fn pieces_cannot_be_placed_over_holes() {
        let board = Board::default();
        assert!(board.place(Piece::O, Square::D22, &Path::from(Piece::O)).is_none());
        assert!(board.place(Piece::O, Square::D23, &Path::from(Piece::O)).is_none());
        assert!(board.place(Piece::O, Square::D24, &Path::from(Piece::O)).is_some());
    }
}
