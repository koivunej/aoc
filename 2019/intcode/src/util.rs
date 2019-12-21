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
            assert!(y < self.width);
            (x as Word, y as Word)
        }

        pub fn len(&self) -> usize {
            self.cells.len()
        }
    }

    impl<T: Default + Clone> GameDisplay<T> {

        pub fn insert(&mut self, p: &(Word, Word), t: T) {
            if self.cells.is_empty() {
                self.smallest_coordinates = *p;
                self.cells.push(t);
                self.width = 1;
                self.height = 1;
                return;
            }

            loop {

                // this could be Result<index, OutsideCoordinates::Before(Word, Word)> where err would be "how much outside"
                if let Some(index) = self.to_index(p) {
                    self.cells[index] = t;
                    return;
                }

                let (x, y) = *p;
                let (mut dx, mut dy) = (x - self.smallest_coordinates.0, y - self.smallest_coordinates.1);

                if dx > 0 {
                    dx -= self.width as Word;
                    dx += 1;
                    // this can become zero if we didn't need to grow there
                    assert!(dx >= 0);
                }

                if dy > 0 {
                    dy -= self.height as Word;
                    dy += 1;
                    // same as with dx and the zero
                    assert!(dy >= 0);
                }

                assert!(dx != 0 || dy != 0, "Both directions became zero for {:?} when {}x{} and {:?}", p, self.width, self.height, self.smallest_coordinates);

                if dx != 0 {
                    // we need to grow columns
                    let mut next = Vec::new();
                    let next_len = (self.width + dx.abs() as usize) * self.height;
                    next.reserve(next_len);
                    std::mem::swap(&mut self.cells, &mut next);

                    while self.cells.len() < next_len {
                        if dx < 0 {
                            for _ in 0..dx.abs() {
                                self.cells.push(T::default());
                            }
                        }
                        self.cells.extend(next.drain(..self.width));
                        if dx > 0 {
                            for _ in 0..dx.abs() {
                                self.cells.push(T::default());
                            }
                        }
                    }

                    if dx < 0 {
                        self.smallest_coordinates.0 += dx;
                    }
                    self.width += dx.abs() as usize;
                    continue;
                }

                // need to prepend lines
                let mut next = vec![T::default(); self.width() * dy.abs() as usize];
                if dy < 0 {
                    next.reserve(self.cells.len());
                    std::mem::swap(&mut self.cells, &mut next);
                    // names get confusing here but for a while "next" contains our previous cells
                    // which are then moved to the end of the game board
                }
                self.cells.extend(next.into_iter());

                if dy < 0 {
                    self.smallest_coordinates.1 += dy;
                }
                self.height += dy.abs() as usize;
            }
        }

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
            println!("{} ({}, {})", i, x, y);
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
}

pub use gamedisplay::GameDisplay;
