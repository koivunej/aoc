use crate::Word;

#[derive(Debug)]
pub enum ParsingError {
    Io(std::io::Error, usize),
    Int(std::num::ParseIntError, usize, String),
}

pub fn parse_program<R: std::io::BufRead>(r: R) -> Result<Vec<Word>, ParsingError> {
    parse_program_n_lines(r, None)
}

pub fn parse_program_n_lines<R: std::io::BufRead>(mut r: R, lines: Option<usize>) -> Result<Vec<Word>, ParsingError> {
    use std::str::FromStr;

    let mut data = vec![];
    let mut buffer = String::new();
    let mut line = 0;

    loop {
        match lines {
            Some(max) if line == max => {
                return Ok(data);
            }
            _ => {},
        }

        buffer.clear();
        let bytes = r
            .read_line(&mut buffer)
            .map_err(|e| ParsingError::Io(e, line))?;

        if bytes == 0 {
            return Ok(data);
        }

        let parts = buffer.trim().split(',').map(Word::from_str);

        for part in parts {
            let part = match part {
                Ok(part) => part,
                Err(e) => return Err(ParsingError::Int(e, line, buffer)),
            };

            data.push(part);
        }

        line += 1;
    }
}

pub fn parse_stdin_program() -> Vec<Word> {
    parse_stdin_program_n_lines(None)
}

pub fn parse_stdin_program_n_lines(n: Option<usize>) -> Vec<Word> {
    let stdin = std::io::stdin();
    let locked = stdin.lock();

    match parse_program_n_lines(locked, n) {
        Ok(data) => data,
        Err(ParsingError::Io(e, line)) => {
            eprintln!("Failed to read stdin near line {}: {}", line, e);
            std::process::exit(1);
        }
        Err(ParsingError::Int(e, line, raw)) => {
            eprintln!("Bad input at line {}: \"{}\" ({})", line, raw, e);
            std::process::exit(1);
        }
    }
}

/// Testing utility: parses "input" from current working directory as a program with
/// `parse_program`, unwrapping on error.
pub fn with_parsed_program<V, F>(f: F) -> V
    where F: FnOnce(&[Word]) -> V
{
    use std::io::BufReader;

    let file = std::fs::File::open("input").expect("Could not open day02 input?");

    let data = parse_program(BufReader::new(file)).unwrap();

    f(&data)
}

mod gamedisplay {
    use crate::Word;
    use std::convert::TryFrom;
    use std::fmt;

    /// Does not really belong to `intcode` but useful for maps and displays.
    #[derive(Default)]
    pub struct GameDisplay<T> {
        cells: Vec<T>,
        // coordinates of the left top corner or bottom?
        smallest_coordinates: (Word, Word),
        width: usize,
        height: usize,
    }

    impl<T: fmt::Display + Default> fmt::Display for GameDisplay<T> {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            let width = self.width();

            let mut any_newline = false;

            for offset in 0..self.cells.len() {
                if offset > 0 && offset % width == 0 {
                    writeln!(fmt, "")?;
                    any_newline = true;
                }
                write!(fmt, "{}", self.cells.get(offset).unwrap_or(&T::default()))?;
            }

            if any_newline && false {
                // not sure if this was a good idea after all
                writeln!(fmt, "")?;
            }

            Ok(())
        }
    }

    impl<T> GameDisplay<T> {
        pub fn to_index(&self, p: &(Word, Word)) -> Option<usize> {
            let (x, y) = *p;
            let w = self.width as Word;
            let h = self.height as Word;

            let contained =
                x >= self.smallest_coordinates.0
                && y >= self.smallest_coordinates.1
                && x < self.smallest_coordinates.0 + w
                && y < self.smallest_coordinates.1 + h;

            if contained {
                let (dx, dy) = (x - self.smallest_coordinates.0, y - self.smallest_coordinates.1);
                let offset = dy * w + dx;
                assert!(offset >= 0, "offset shouldn't be negative: {}", offset);
                Some(offset as usize)
            } else {
                None
            }
        }

        fn width(&self) -> usize {
            self.width
        }

        #[allow(dead_code)]
        fn height(&self) -> usize {
            self.height
        }

        #[allow(dead_code)]
        pub fn get(&self, p: &(Word, Word)) -> Option<&T> {
            self.to_index(p)
                .and_then(|index| self.cells.get(index))
        }

        pub fn cells(&self) -> &[T] {
            self.cells.as_slice()
        }

        pub fn to_coordinates(&self, index: usize) -> (Word, Word) {
            let x = index % self.width;
            let y = index / self.width;
            assert!(y < self.height);
            (x as Word, y as Word)
        }

        pub fn len(&self) -> usize {
            self.cells.len()
        }
    }

    impl<T: Default + Clone> GameDisplay<T> {

        pub fn insert(&mut self, p: &(Word, Word), t: T) {
            // protect against infinite loops
            let mut gas = 3usize;

            if self.cells.is_empty() {
                self.smallest_coordinates = *p;
                self.cells.push(t);
                self.width = 1;
                self.height = 1;
                return;
            }

            loop {
                if let Some(next_gas) = gas.checked_sub(1) {
                    gas = next_gas;
                } else {
                    panic!("ran out of gas while trying to insert {:?}, w={}, h={}, smallest={:?}", p, self.width, self.height, self.smallest_coordinates);
                }

                // this could be Result<index, OutsideCoordinates::Before(Word, Word)> where err would be "how much outside"
                if let Some(index) = self.to_index(p) {
                    self.cells[index] = t;
                    return;
                }

                let g = Growth::from_setup(
                    (self.width, self.height),
                    self.smallest_coordinates,
                    *p);

                let g = g.unwrap();

                let mut size = (self.width, self.height);

                g.grow(&mut self.cells, &mut size, &mut self.smallest_coordinates);

                self.width = size.0;
                self.height = size.1;
            }
        }
    }


    impl<T: TryFrom<char> + Default + Clone> GameDisplay<T> {
        pub fn parse_grid_lines(&mut self, line: &str, start_pos: (Word, Word)) -> Result<(Word, Word), <T as TryFrom<char>>::Error>  {
            let mut pos = start_pos;
            for ch in line.chars() {
                if ch == '\n' {
                    pos.0 = 0;
                    pos.1 += 1;
                    continue;
                }

                let item = T::try_from(ch)?;

                self.insert(&pos, item);
                pos.0 += 1;
            }
            Ok(pos)
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum Direction { Width, Height }

    #[derive(Debug, PartialEq, Eq)]
    enum Position { Before, After }

    #[derive(Debug, PartialEq, Eq)]
    struct Growth(Direction, Position, usize);

    impl Growth {
        fn from_setup((w, h): (usize, usize), (minx, miny): (Word, Word), (x, y): (Word, Word)) -> Option<Growth> {
            let w = w as Word;
            let h = h as Word;

            // inclusive max{x,y}
            let maxx = minx + w - 1;
            let maxy = miny + h - 1;

            if x < minx {
                Some(Growth(Direction::Width, Position::Before, usize::try_from((x - minx).abs()).unwrap()))
            } else if x > maxx {
                Some(Growth(Direction::Width, Position::After, usize::try_from((x - maxx).abs()).unwrap()))
            } else if y < miny {
                Some(Growth(Direction::Height, Position::Before, usize::try_from((y - miny).abs()).unwrap()))
            } else if y > maxy {
                Some(Growth(Direction::Height, Position::After, usize::try_from((y - maxy).abs()).unwrap()))
            } else {
                None
            }
        }

        fn grow<T: Clone + Default>(&self, mut cells: &mut Vec<T>, size: &mut (usize, usize), min: &mut (Word, Word)) {
            assert_eq!(cells.len(), size.0 * size.1);
            match *self {
                Growth(Direction::Width, ref p, columns) => {
                    let new_columns = vec![T::default(); columns];

                    let mut next = Vec::with_capacity((size.0 + columns) * size.1);

                    for _ in 0..size.1 {
                        match *p {
                            Position::Before => {
                                next.extend(new_columns.iter().cloned());
                                next.extend(cells.drain(..size.0));
                            },
                            Position::After => {
                                next.extend(cells.drain(..size.0));
                                next.extend(new_columns.iter().cloned());
                            },
                        }
                    }

                    assert!(cells.is_empty());
                    std::mem::swap(&mut next, &mut cells);

                    if p == &Position::Before {
                        min.0 -= columns as Word;
                    }

                    size.0 += columns;
                },
                Growth(Direction::Height, ref p, rows) => {
                    let new_row = vec![T::default(); size.0];

                    match *p {
                        Position::Before => {
                            let mut next = Vec::with_capacity(size.0 * (size.1 + rows));

                            if rows > 1 {
                                for _ in 0..(rows - 1) {
                                    next.extend(new_row.iter().cloned());
                                }
                            }
                            next.extend(new_row);

                            next.extend(cells.drain(..));
                            std::mem::swap(&mut next, &mut cells);

                            min.1 -= rows as Word;
                        },
                        Position::After => {
                            if rows > 1 {
                                for _ in 0..(rows - 1) {
                                    cells.extend(new_row.iter().cloned());
                                }
                            }
                            cells.extend(new_row);
                        }
                    }

                    size.1 += rows;
                }
            }
            assert_eq!(cells.len(), size.0 * size.1);
        }

    }

    #[test]
    fn grow_positive_one() {
        // assume singleton at (0,0)
        assert_eq!(Growth::from_setup(( 1, 1), ( 0, 0), ( 1, 1)), Some(Growth(Direction::Width, Position::After, 1)));
        assert_eq!(Growth::from_setup(( 2, 1), ( 0, 0), ( 1, 1)), Some(Growth(Direction::Height, Position::After, 1)));
    }

    #[test]
    fn growth_growing() {
        growth_scenario(
            Growth(Direction::Width, Position::Before, 1),
            vec!['x'], (1, 1), (0, 0),
            vec!['\0', 'x'], (2, 1), (-1, 0));

        growth_scenario(
            Growth(Direction::Width, Position::After, 1),
            vec!['x'], (1, 1), (0, 0),
            vec!['x', '\0'], (2, 1), (0, 0));

        growth_scenario(
            Growth(Direction::Height, Position::Before, 1),
            vec!['x'], (1, 1), (0, 0),
            vec!['\0', 'x'], (1, 2), (0, -1));

        growth_scenario(
            Growth(Direction::Height, Position::After, 1),
            vec!['x'], (1, 1), (0, 0),
            vec!['x', '\0'], (1, 2), (0, 0));
    }

    #[cfg(test)]
    fn growth_scenario<T: PartialEq + Clone + Default + fmt::Debug>(
        g: Growth,
        mut initial: Vec<T>,
        mut size: (usize, usize),
        mut min: (Word, Word),
        expected_cells: Vec<T>,
        expected_size: (usize, usize),
        expected_min: (Word, Word))
    {
        g.grow(&mut initial, &mut size, &mut min);
        assert_eq!(expected_cells, initial);
        assert_eq!(expected_size, size);
        assert_eq!(expected_min, min);
    }


    #[test]
    fn positive_growth_not_needed() {
        assert_eq!(Growth::from_setup(( 2, 2), ( 0, 0), ( 1, 1)), None);
        assert_eq!(Growth::from_setup(( 2, 2), ( 0, 0), ( 0, 1)), None);
        assert_eq!(Growth::from_setup(( 2, 2), ( 0, 0), ( 1, 0)), None);
        assert_eq!(Growth::from_setup(( 2, 2), ( 0, 0), ( 0, 0)), None);
    }

    #[test]
    fn grow_positive_jump() {
        assert_eq!(Growth::from_setup(( 1, 1), ( 0, 0), ( 0,10)), Some(Growth(Direction::Height, Position::After, 10)));
        assert_eq!(Growth::from_setup(( 1, 1), ( 0, 0), (10, 0)), Some(Growth(Direction::Width, Position::After, 10)));
    }

    #[test]
    fn grow_negative_one() {
        //    |
        //    |
        //----O----
        //   x|
        //    |
        assert_eq!(Growth::from_setup(( 1, 1), ( 0, 0), (-1,-1)), Some(Growth(Direction::Width, Position::Before, 1)));
        assert_eq!(Growth::from_setup(( 2, 1), (-1, 0), (-1,-1)), Some(Growth(Direction::Height, Position::Before, 1)));
    }

    #[test]
    fn negative_growth_not_needed() {
        assert_eq!(Growth::from_setup(( 2, 2), (-1,-1), (-1,-1)), None);
        assert_eq!(Growth::from_setup(( 2, 2), (-1,-1), ( 0,-1)), None);
        assert_eq!(Growth::from_setup(( 2, 2), (-1,-1), (-1, 0)), None);
        assert_eq!(Growth::from_setup(( 2, 2), (-1,-1), ( 0, 0)), None);
    }

    #[test]
    fn grow_negative_more() {
        assert_eq!(Growth::from_setup(( 2, 2), ( 0, 0), (-1,-1)), Some(Growth(Direction::Width, Position::Before, 1)));
        assert_eq!(Growth::from_setup(( 3, 2), (-1, 0), (-1,-1)), Some(Growth(Direction::Height, Position::Before, 1)));
        assert_eq!(Growth::from_setup(( 3, 3), (-1,-1), (-1,-1)), None);
    }

    #[test]
    fn gamedisplay_grows() {

        #[derive(Debug, PartialEq, Clone)]
        enum Example {
            Ball,
            Block,
            Empty,
        }

        impl Default for Example {
            fn default() -> Example {
                Example::Empty
            }
        }

        impl fmt::Display for Example {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                let ch = match *self {
                    Example::Ball => 'O',
                    Example::Block => 'X',
                    Example::Empty => ' ',
                };

                write!(fmt, "{}", ch)
            }
        }


        let mut gd = GameDisplay::default();

        gd.insert(&(1, 1), Example::Ball);
        assert_eq!(gd.smallest_coordinates, (1, 1));
        assert_eq!(format!("{}", gd).as_str(), "O");
        assert_eq!((gd.width(), gd.height()), (1, 1));

        gd.insert(&(0, 0), Example::Block);
        assert_eq!(gd.smallest_coordinates, (0, 0));
        assert_eq!((gd.width(), gd.height()), (2, 2));
        assert_eq!(format!("{}", gd).as_str(), "X \n O");

        gd.insert(&(1, 0), Example::Block);
        assert_eq!((gd.width(), gd.height()), (2, 2));
        assert_eq!(format!("{}", gd).as_str(), "XX\n O");

        gd.insert(&(2, 0), Example::Block);
        assert_eq!((gd.width(), gd.height()), (3, 2));
        assert_eq!(format!("{}", gd).as_str(), "XXX\n O ");

        gd.insert(&(2, 2), Example::Block);
        assert_eq!((gd.width(), gd.height()), (3, 3));
        assert_eq!(format!("{}", gd).as_str(), "XXX\n O \n  X");

        gd.insert(&(-1, -1), Example::Block);
        assert_eq!(gd.smallest_coordinates, (-1, -1));
        assert_eq!((gd.width(), gd.height()), (4, 4));
        assert_eq!(format!("{}", gd).as_str(), "X   \n XXX\n  O \n   X");

        let mut contents = vec![Example::Empty; 16];
        contents[0] = Example::Block;
        contents[5] = Example::Block;
        contents[6] = Example::Block;
        contents[7] = Example::Block;
        contents[10] = Example::Ball;
        contents[15] = Example::Block;

        let checks = (-1..3).into_iter()
            .flat_map(|y| ((-1..3).into_iter().map(move |x| (x, y))))
            .enumerate();

        for (i, (x, y)) in checks {
            assert_eq!(contents.get(i), gd.get(&(x, y)), "Failed at index {} or ({}, {})", i, x, y);
        }

        for y in -1..4 {
            assert_eq!(None, gd.get(&(-2, y)));
            assert_eq!(None, gd.get(&( 3, y)));
        }

        for x in -2..4 {
            assert_eq!(None, gd.get(&(x, -2)));
            assert_eq!(None, gd.get(&(x,  3)));
        }
    }

    #[test]
    fn first_bug_while_day15() {
        let mut gd: GameDisplay<char> = GameDisplay::default();

        let a = 'a';

        gd.insert(&( 0, 0), a);
        gd.insert(&( 0,-1), a);
        gd.insert(&( 1, 0), a);
        gd.insert(&( 0, 0), a);
        gd.insert(&( 1,-1), a);
        gd.insert(&( 1, 0), a);
        gd.insert(&( 1,-2), a);
        gd.insert(&( 2,-1), a);
        gd.insert(&( 0, 2), a);
    }

    #[test]
    fn second_bug_while_day15() {
        let mut gd: GameDisplay<char> = GameDisplay::default();

        let a = 'a';

        gd.insert(&( 0,-1), a);
        gd.insert(&( 1, 0), a);
        gd.insert(&( 0, 1), a);
        gd.insert(&(-1, 0), a);
        gd.insert(&( 1, 1), a);
        gd.insert(&( 0, 2), a);

    }
}

pub use gamedisplay::GameDisplay;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    // position grows
    Up,
    // position grows
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn turn_left(&self) -> Self {
        use Direction::*;

        match *self {
            Up => Left,
            Right => Up,
            Down => Right,
            Left => Down,
        }
    }

    pub fn turn_right(&self) -> Self {
        use Direction::*;

        match *self {
            Up => Right,
            Right => Down,
            Down => Left,
            Left => Up,
        }
    }
}

impl Into<Position> for &Direction {
    fn into(self) -> Position {
        use Direction::*;

        let tuple = match *self {
            Up => (0, -1),
            Right => (1, 0),
            Down => (0, 1),
            Left => (-1, 0),
        };

        tuple.into()
    }
}

impl Into<Position> for Direction {
    fn into(self) -> Position {
        (&self).into()
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position(Word, Word);

impl Position {
    pub fn x(&self) -> Word {
        self.0
    }

    pub fn y(&self) -> Word {
        self.1
    }
}

impl<T: Into<Position>> std::ops::Add<T> for Position {
    type Output = Position;

    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl std::ops::Mul<i64> for Position {
    type Output = Position;

    fn mul(self, rhs: i64) -> Self::Output {
        Position(self.0 * rhs, self.1 * rhs)
    }
}

impl<T: Into<Position>> std::ops::Sub<T> for Position {
    type Output = Position;

    fn sub(self, rhs: T) -> Self::Output {
        let rhs = rhs.into() * -1;
        self + rhs
    }
}

impl Into<(Word, Word)> for &Position {
    fn into(self) -> (Word, Word) {
        (self.0, self.1)
    }
}

impl Into<(Word, Word)> for Position {
    fn into(self) -> (Word, Word) {
        (&self).into()
    }
}

impl From<&(Word, Word)> for Position {
    fn from(tuple: &(Word, Word)) -> Position {
        Position(tuple.0, tuple.1)
    }
}

impl From<(Word, Word)> for Position {
    fn from(tuple: (Word, Word)) -> Self {
        (&tuple).into()
    }
}
