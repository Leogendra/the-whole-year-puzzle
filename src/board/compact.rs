use crate::board::Board;
use crate::board::piece::Piece;
use crate::board::square::{Square, GRID_SIZE};
use crate::board::path::Path;

use crate::board::placement::*;
use Rotation::*;

/// One mirror bit per piece, packed from the most significant bit of the first byte.
pub const MIRROR_BYTE_COUNT: usize = Piece::COUNT.div_ceil(8);
/// Mirror bytes followed by one byte per piece holding its rotation and square.
pub const BOARD_BYTE_COUNT: usize = MIRROR_BYTE_COUNT + Piece::COUNT;

/// Six bits per square index, all six set meaning the piece is not placed.
const NO_SQUARE: u8 = 0b111111;
const _: () = assert!(GRID_SIZE <= NO_SQUARE as usize, "square indices must fit in six bits");

#[derive(Debug)]
pub struct CompactBoard {
    squares: [Option<Square>; Piece::COUNT],
    rotations: [Rotation; Piece::COUNT],
    mirrors: [bool; Piece::COUNT],
}

impl TryFrom<CompactBoard> for Board {
    type Error = PlacementError;

    fn try_from(cb: CompactBoard) -> Result<Self, Self::Error> {
        let mut board = Board::default();
        let pieces = Piece::pieces();
        for (i, &piece) in pieces.iter().enumerate() {
            if let (Some(square), rotation, mirror) = (cb.squares[i], cb.rotations[i], cb.mirrors[i]) {
                let path = Path::from_orientation(piece, rotation, mirror);
                board = board.place(piece, square, &path).ok_or(PlacementError)?;
            }
        }
        Ok(board)
    }
}

impl TryFrom<Board> for CompactBoard {
    type Error = PlacementError;

    fn try_from(board: Board) -> Result<Self, Self::Error> {
        let mut squares = [None; Piece::COUNT];
        let mut rotations = [Zero; Piece::COUNT];
        let mut mirrors = [false; Piece::COUNT];

        for Placement { piece, square, rotation, mirror } in board.placements()? {
            squares[piece as usize] = Some(square);
            rotations[piece as usize] = rotation;
            mirrors[piece as usize] = mirror;
        }

        Ok(CompactBoard { squares, rotations, mirrors })
    }
}

impl CompactBoard {
    pub fn to_bytes(&self) -> [u8; BOARD_BYTE_COUNT] {
        let mut bytes = [0; BOARD_BYTE_COUNT];

        for (i, &mirror) in self.mirrors.iter().enumerate() {
            bytes[i / 8] |= (mirror as u8) << (7 - i % 8);
        }

        for (i, (&rotation, &square)) in self.rotations.iter().zip(self.squares.iter()).enumerate() {
            bytes[MIRROR_BYTE_COUNT + i] = (rotation as u8) << 6 | match square {
                None => NO_SQUARE,
                Some(square) => square as u8,
            };
        }

        bytes
    }
}

impl From<[u8; BOARD_BYTE_COUNT]> for CompactBoard {
    fn from(bytes: [u8; BOARD_BYTE_COUNT]) -> Self {
        let mut squares = [None; Piece::COUNT];
        let mut rotations = [Zero; Piece::COUNT];
        let mut mirrors = [false; Piece::COUNT];

        for i in 0..Piece::COUNT {
            let piece_byte = bytes[MIRROR_BYTE_COUNT + i];
            squares[i] = Square::try_from(piece_byte & NO_SQUARE).ok();
            rotations[i] = Rotation::from((piece_byte >> 6) & 0b11);
            mirrors[i] = (bytes[i / 8] >> (7 - i % 8)) & 0b1 == 1;
        }

        CompactBoard { squares, rotations, mirrors }
    }
}
