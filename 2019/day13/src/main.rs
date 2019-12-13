use std::collections::HashSet;
use std::fmt;
use intcode::{parse_stdin_program, Program, Environment, Word};

fn main() {
    let data = parse_stdin_program();

    println!("stage1: {}", stage1(&data[..]));
    println!("stage2: {}", stage2(&data[..]));
}

fn stage1(data: &[Word]) -> usize {
    let mut data = data.to_vec();
    let mut env = Environment::collector(None);

    Program::wrap(&mut data)
        .with_memory_expansion()
        .eval_with_env(&mut env)
        .unwrap();

    let collected = env.unwrap_collected();

    let mut uniq = HashSet::new();
    for p in collected.chunks(3).filter(|chunk| chunk[2] == 2).map(|chunk| (chunk[0], chunk[1])) {
        uniq.insert(p);
    }

    uniq.len()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TileKind {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball
}

impl TileKind {
    fn is_indestructible(&self) -> bool {
        match *self {
            TileKind::Wall | TileKind::Paddle | TileKind::Ball => true,
            _ => false
        }
    }
}

impl fmt::Display for TileKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use TileKind::*;
        let ch = match *self {
            Empty  => ' ',
            Wall   => '█',
            Block  => 'X',
            Paddle => 'P',
            Ball   => 'O',
        };
        write!(fmt, "{}", ch)
    }
}

#[derive(Default)]
struct GameDisplay {
    cells: Vec<TileKind>,
    // coordinates of the left top corner or bottom?
    smallest_coordinates: (Word, Word),
    width: usize,
    height: usize,
}

impl fmt::Display for GameDisplay {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let width = self.width();
        let mut offset = 0;

        let mut any_newline = false;

        for offset in 0..self.cells.len() {
            if offset > 0 && offset % width == 0 {
                writeln!(fmt, "")?;
                any_newline = true;
            }
            write!(fmt, "{}", self.cells.get(offset).unwrap_or(&TileKind::Empty))?;
        }

        if any_newline && false {
            // not sure if this was a good idea after all
            writeln!(fmt, "")?;
        }

        Ok(())
    }
}

impl GameDisplay {
    /*fn contains(&self, p: &(Word, Word)) -> bool {
        let (x, y) = *p;
        x >= self.smallest_coordinates.0
            && y >= self.smallest_coordinates.1
            && x < self.smallest_coordinates.0 + self.width as Word
            && y < self.smallest_coordinates.1 + self.height as Word
    }*/

    fn to_index(&self, p: &(Word, Word)) -> Option<usize> {
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

    fn to_coordinates(&self, index: &usize) -> (Word, Word) {
        unimplemented!()
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn insert(&mut self, p: &(Word, Word), t: TileKind) {
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

            println!("thinking of growing for {:?} with {}x{} and 0 at {:?}", (x, y), self.width, self.height, self.smallest_coordinates);

            let (mut dx, mut dy) = (x - self.smallest_coordinates.0, y - self.smallest_coordinates.1);

            println!("maybe need to grow {:?}", (dx, dy));

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

            println!("need to grow: {:?}", (dx, dy));
            println!("{:?}", self.cells);

            if dx != 0 {
                // we need to grow columns
                let mut next = Vec::new();
                let next_len = (self.width + dx.abs() as usize) * self.height;
                next.reserve(next_len);
                std::mem::swap(&mut self.cells, &mut next);

                while self.cells.len() < next_len {
                    if dx < 0 {
                        for _ in 0..dx.abs() {
                            self.cells.push(TileKind::Empty);
                        }
                    }
                    self.cells.extend(next.drain(dbg!(..self.width)));
                    if dx > 0 {
                        for _ in 0..dx.abs() {
                            self.cells.push(TileKind::Empty);
                        }
                    }
                }

                if dx < 0 {
                    self.smallest_coordinates.0 += dx;
                }
                self.width += dx.abs() as usize;
                continue;
            }

            if dy != 0 {
                // need to prepend lines
                let mut next = vec![TileKind::Empty; self.width() * dy.abs() as usize];
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
                continue;
            }

            dbg!((dx, dy));

            unimplemented!()
        }
    }

    fn get(&self, p: &(Word, Word)) -> Option<&TileKind> {
        self.to_index(p)
            .and_then(|index| self.cells.get(index))
    }
}

fn stage2(data: &[Word]) -> Word {
    let mut data = data.to_vec();
    data[0] = 2; // infinite coins

    unimplemented!();
}

#[test]
fn gamedisplay_grows() {
    let mut gd = GameDisplay::default();

    gd.insert(&(1, 1), TileKind::Ball);
    assert_eq!(gd.smallest_coordinates, (1, 1));
    assert_eq!(format!("{}", gd).as_str(), "O");
    assert_eq!((gd.width(), gd.height()), (1, 1));

    gd.insert(&(0, 0), TileKind::Block);
    assert_eq!(gd.smallest_coordinates, (0, 0));
    assert_eq!((gd.width(), gd.height()), (2, 2));
    assert_eq!(format!("{}", gd).as_str(), "X \n O");

    gd.insert(&(1, 0), TileKind::Block);
    assert_eq!((gd.width(), gd.height()), (2, 2));
    assert_eq!(format!("{}", gd).as_str(), "XX\n O");

    gd.insert(&(2, 0), TileKind::Block);
    assert_eq!((gd.width(), gd.height()), (3, 2));
    assert_eq!(format!("{}", gd).as_str(), "XXX\n O ");

    gd.insert(&(2, 2), TileKind::Block);
    assert_eq!((gd.width(), gd.height()), (3, 3));
    assert_eq!(format!("{}", gd).as_str(), "XXX\n O \n  X");

    gd.insert(&(-1, -1), TileKind::Block);
    assert_eq!(gd.smallest_coordinates, (-1, -1));
    assert_eq!((gd.width(), gd.height()), (4, 4));
    assert_eq!(format!("{}", gd).as_str(), "X   \n XXX\n  O \n   X");

    assert_eq!(Some(&TileKind::Block), gd.get(&(-1, -1)));
    assert_eq!(Some(&TileKind::Empty), gd.get(&( 0, -1)));
    assert_eq!(Some(&TileKind::Empty), gd.get(&( 1, -1)));
    assert_eq!(Some(&TileKind::Empty), gd.get(&( 2, -1)));

    assert_eq!(Some(&TileKind::Empty), gd.get(&(-1,  0)));
    assert_eq!(Some(&TileKind::Block), gd.get(&( 0,  0)));
    assert_eq!(Some(&TileKind::Block), gd.get(&( 1,  0)));
    assert_eq!(Some(&TileKind::Block), gd.get(&( 2,  0)));

    assert_eq!(Some(&TileKind::Empty), gd.get(&(-1,  1)));
    assert_eq!(Some(&TileKind::Empty), gd.get(&( 0,  1)));
    assert_eq!(Some(&TileKind::Ball),  gd.get(&( 1,  1)));
    assert_eq!(Some(&TileKind::Empty), gd.get(&( 2,  1)));

    assert_eq!(Some(&TileKind::Empty), gd.get(&(-1,  2)));
    assert_eq!(Some(&TileKind::Empty), gd.get(&( 0,  2)));
    assert_eq!(Some(&TileKind::Empty), gd.get(&( 1,  2)));
    assert_eq!(Some(&TileKind::Block), gd.get(&( 2,  2)));

    for y in -1..4 {
        assert_eq!(None, gd.get(&(-2, y)));
        assert_eq!(None, gd.get(&( 3, y)));
    }

    for x in -2..4 {
        assert_eq!(None, gd.get(&(x, -2)));
        assert_eq!(None, gd.get(&(x,  3)));
    }
}
