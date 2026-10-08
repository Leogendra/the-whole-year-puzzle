use crate::board::path::Direction::{self, *};

use Square::*;

use std::fmt;

/// The frame is laid out on a 7x7 grid, indexed from left to right and top to bottom.
pub const GRID_WIDTH: usize = 7;
pub const GRID_HEIGHT: usize = 7;
pub const GRID_SIZE: usize = GRID_WIDTH * GRID_HEIGHT;

/// Each square's discriminant is its index in the grid.
/// Indices without a square (6, 13, 42, 43, 47 and 48) are holes in the frame.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Hash)]
#[repr(u8)]
pub enum Square {
    Jan =  0, Feb, Mar, Apr, May, Jun,
    Jul =  7, Aug, Sep, Oct, Nov, Dec,
    D01 = 14, D02, D03, D04, D05, D06, D07,
    D08 = 21, D09, D10, D11, D12, D13, D14,
    D15 = 28, D16, D17, D18, D19, D20, D21,
    D22 = 35, D23, D24, D25, D26, D27, D28,
    D29 = 44, D30, D31,
}

/// Lookup table from grid index to square, built from the enum so that holes are defined in one place.
const SQUARES_BY_INDEX: [Option<Square>; GRID_SIZE] = {
    let mut table = [None; GRID_SIZE];
    let mut i = 0;
    while i < Square::MONTHS.len() {
        table[Square::MONTHS[i] as usize] = Some(Square::MONTHS[i]);
        i += 1;
    }
    let mut i = 0;
    while i < Square::DAYS.len() {
        table[Square::DAYS[i] as usize] = Some(Square::DAYS[i]);
        i += 1;
    }
    table
};

impl Square {
    pub const MONTHS: [Square; 12] = [
        Jan, Feb, Mar, Apr, May, Jun,
        Jul, Aug, Sep, Oct, Nov, Dec,
    ];

    pub const DAYS: [Square; 31] = [
        D01, D02, D03, D04, D05, D06, D07,
        D08, D09, D10, D11, D12, D13, D14,
        D15, D16, D17, D18, D19, D20, D21,
        D22, D23, D24, D25, D26, D27, D28,
        D29, D30, D31,
    ];

    pub fn step(&self, dir: Direction) -> Option<Self> {
        let square = *self as usize;
        let (col, row) = (square % GRID_WIDTH, square / GRID_WIDTH);
        let target = match dir {
            Up    if row > 0               => square - GRID_WIDTH,
            Down  if row + 1 < GRID_HEIGHT => square + GRID_WIDTH,
            Left  if col > 0               => square - 1,
            Right if col + 1 < GRID_WIDTH  => square + 1,
            _ => return None,
        };
        SQUARES_BY_INDEX[target]
    }

    pub fn squares() -> Vec<Square> {
        [Self::MONTHS.as_slice(), Self::DAYS.as_slice()].concat()
    }

    /// The square of a month, numbered from 1 (January) to 12 (December).
    pub fn month(number: usize) -> Option<Square> {
        Self::MONTHS.get(number.checked_sub(1)?).copied()
    }

    /// The square of a day of the month, numbered from 1 to 31.
    pub fn day(number: usize) -> Option<Square> {
        Self::DAYS.get(number.checked_sub(1)?).copied()
    }

    /// The month number (1-12) or day number (1-31) written on the square.
    pub fn number(&self) -> usize {
        let position = match Self::MONTHS.iter().position(|square| square == self) {
            Some(position) => position,
            None => Self::DAYS.iter().position(|square| square == self).expect("every square should be a month or a day"),
        };
        position + 1
    }
}

#[derive(Debug)]
pub struct IndexError;

impl TryFrom<u8> for Square {
    type Error = IndexError;

    fn try_from(index: u8) -> Result<Self, Self::Error> {
        SQUARES_BY_INDEX.get(index as usize).copied().flatten().ok_or(IndexError)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Jan => "Jan", Feb => "Feb", Mar => "Mar", Apr => "Apr", May => "May", Jun => "Jun",
            Jul => "Jul", Aug => "Aug", Sep => "Sep", Oct => "Oct", Nov => "Nov", Dec => "Dec",
            D01 =>   "1", D02 =>   "2", D03 =>   "3", D04 =>   "4", D05 =>   "5", D06 =>   "6", D07 =>   "7",
            D08 =>   "8", D09 =>   "9", D10 =>  "10", D11 =>  "11", D12 =>  "12", D13 =>  "13", D14 =>  "14",
            D15 =>  "15", D16 =>  "16", D17 =>  "17", D18 =>  "18", D19 =>  "19", D20 =>  "20", D21 =>  "21",
            D22 =>  "22", D23 =>  "23", D24 =>  "24", D25 =>  "25", D26 =>  "26", D27 =>  "27", D28 =>  "28",
            D29 =>  "29", D30 =>  "30", D31 =>  "31",
        })
    }
}

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Hash)]
pub struct Date {
    pub month: Square,
    pub day: Square,
}

impl Date {
    pub fn is_valid(&self) -> bool {
        match (self.month, self.day) {
            _ if !Square::MONTHS.contains(&self.month) || !Square::DAYS.contains(&self.day) => false,
            (Apr | Jun | Sep | Nov, D31) => false,
            (Feb, D30 | D31) => false,
            _ => true
        }
    }

    pub fn next(&self) -> Self {
        if !self.is_valid() {
            return Date { month: Jan, day: D01 };
        }

        if let Some(next_day) = Square::day(self.day.number() + 1) {
            let next_date = Date { month: self.month, day: next_day };
            if next_date.is_valid() {
                return next_date;
            }
        }

        let next_month = Square::month(self.month.number() % 12 + 1).expect("month number should be in range 1-12");

        Date { month: next_month, day: D01 }
    }

    pub fn prev(&self) -> Self {
        if !self.is_valid() {
            return Date { month: Jan, day: D01 };
        }

        if self.day == D01 {
            let prev_month = Square::month((self.month.number() + 10) % 12 + 1).expect("month number should be in range 1-12");
            let prev_day = match self.month {
                Mar => D29,
                May | Jul | Oct | Dec => D30,
                _ => D31,
            };
            return Date { month: prev_month, day: prev_day };
        }

        Date { month: self.month, day: Square::day(self.day.number() - 1).expect("day number should be in range 2-31") }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.month, self.day)
    }
}

pub type DateMap<T> = std::collections::HashMap<Date, T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holes_are_not_squares() {
        for hole in [6, 13, 42, 43, 47, 48, 49] {
            assert!(Square::try_from(hole).is_err(), "index {hole} should be a hole");
        }
    }

    #[test]
    fn every_square_round_trips_through_its_index() {
        for square in Square::squares() {
            assert_eq!(Square::try_from(square as u8).ok(), Some(square));
        }
        assert_eq!(Square::squares().len(), 43);
    }

    #[test]
    fn steps_into_holes_or_off_the_grid_are_rejected() {
        assert_eq!(D22.step(Down), None);
        assert_eq!(D28.step(Down), None);
        assert_eq!(D31.step(Right), None);
        assert_eq!(D29.step(Left), None);
        assert_eq!(Jun.step(Right), None);
        assert_eq!(D07.step(Right), None);
        assert_eq!(D24.step(Down), Some(D29));
        assert_eq!(D29.step(Up), Some(D24));
    }

    #[test]
    fn numbers_match_month_and_day() {
        assert_eq!(Square::month(7), Some(Jul));
        assert_eq!(Square::day(29), Some(D29));
        assert_eq!(Square::day(0), None);
        assert_eq!(Square::day(32), None);
        assert_eq!(D29.number(), 29);
        assert_eq!(Dec.number(), 12);
    }

    #[test]
    fn next_crosses_the_holes_between_days() {
        assert_eq!(Date { month: Jan, day: D28 }.next(), Date { month: Jan, day: D29 });
        assert_eq!(Date { month: Jan, day: D31 }.next(), Date { month: Feb, day: D01 });
        assert_eq!(Date { month: Feb, day: D29 }.next(), Date { month: Mar, day: D01 });
        assert_eq!(Date { month: Jun, day: D30 }.next(), Date { month: Jul, day: D01 });
        assert_eq!(Date { month: Dec, day: D31 }.next(), Date { month: Jan, day: D01 });
    }

    #[test]
    fn prev_crosses_the_holes_between_days() {
        assert_eq!(Date { month: Jan, day: D29 }.prev(), Date { month: Jan, day: D28 });
        assert_eq!(Date { month: Mar, day: D01 }.prev(), Date { month: Feb, day: D29 });
        assert_eq!(Date { month: Jul, day: D01 }.prev(), Date { month: Jun, day: D30 });
        assert_eq!(Date { month: Jan, day: D01 }.prev(), Date { month: Dec, day: D31 });
    }

    #[test]
    fn next_visits_all_366_dates() {
        let start = Date { month: Jan, day: D01 };
        let mut date = start.next();
        let mut count = 1;
        while date != start {
            assert_eq!(date.next().prev(), date);
            date = date.next();
            count += 1;
        }
        assert_eq!(count, 366);
    }
}
