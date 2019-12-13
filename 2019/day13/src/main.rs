use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt;
use intcode::{util::parse_stdin_program_n_lines, Program, Environment, Word, ExecutionState, Registers};

fn main() {
    let data = parse_stdin_program_n_lines(Some(1));

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
    #[allow(dead_code)]
    fn is_indestructible(&self) -> bool {
        match *self {
            TileKind::Wall | TileKind::Paddle | TileKind::Ball => true,
            _ => false
        }
    }
}

impl TryFrom<Word> for TileKind {
    type Error = Word;

    fn try_from(w: Word) -> Result<Self, Self::Error> {
        Ok(match w {
            0 => TileKind::Empty,
            1 => TileKind::Wall,
            2 => TileKind::Block,
            3 => TileKind::Paddle,
            4 => TileKind::Ball,
            x => return Err(x),
        })
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

    fn width(&self) -> usize {
        self.width
    }

    #[allow(dead_code)]
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
                //if self.cells[index].is_indestructible() && self.cells[index] != t {
                //    panic!("Attempting to overwrite indestructible {:?} at {:?} with {:?}", self.cells[index], p, t);
                //}
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
                            self.cells.push(TileKind::Empty);
                        }
                    }
                    self.cells.extend(next.drain(..self.width));
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
        }
    }

    #[allow(dead_code)]
    fn get(&self, p: &(Word, Word)) -> Option<&TileKind> {
        self.to_index(p)
            .and_then(|index| self.cells.get(index))
    }
}

fn stage2(data: &[Word]) -> Word {
    use std::collections::VecDeque;
    println!();

    let mut data = data.to_vec();
    data[0] = 2; // infinite coins

    let mut program = Program::wrap(&mut data)
        .with_memory_expansion();

    let mut disp = GameDisplay::default();
    let mut regs = Registers::default();

    let mut buffer = VecDeque::new();
    let mut input_buffer = String::new();

    let mut score = 0;
    let mut render = true;

    let mut last_ball_pos: Option<((Word, Word), usize)> = None;
    let mut last_paddle_pos: Option<((Word, Word), usize)> = None;

    let mut round = 0;

    loop {
        regs = match program.eval_from_instruction(regs).unwrap() {
            ExecutionState::Paused(_regs) => unreachable!("Pausing not implemented yet?"),
            ExecutionState::HaltedAt(_regs) => break,
            ExecutionState::InputIO(io) => {

                // NOT stage2
                /*
                let points_remaining: usize = disp.cells.iter().filter(|t| **t == TileKind::Block)
                    .enumerate()
                    .map(|(i, _)| i + 19)
                    .sum();

                println!("points remaining: {}", points_remaining);
                */

                /*
                println!("Left/Middle/Right? ");
                let input: Word;

                loop {
                    input_buffer.clear();
                    let _bytes = std::io::stdin().read_line(&mut input_buffer).unwrap();

                    input = match input_buffer.trim() {
                        "L" | "l" => -1,
                        "M" | "m" => 0,
                        "R" | "r" => 1,
                        _ => continue,
                    };
                    break;
                }*/

                println!("Score: {}", score);

                let dx = match (last_ball_pos, last_paddle_pos) {
                    (Some((bp, _)), Some((pp, _))) => bp.0 - pp.0,
                    _ => 0,
                };

                round += 1;
                program.handle_input_completion(io, dx.signum()).unwrap()
            },
            ExecutionState::OutputIO(io, value) => {

                buffer.push_back(value);

                while buffer.len() >= 3 {
                    let x = buffer.pop_front().unwrap();
                    let y = buffer.pop_front().unwrap();
                    let value = buffer.pop_front().unwrap();

                    if x == -1 && y == 0 {
                        score = value;
                        continue;
                    }

                    let prev = *disp.get(&(x, y))
                        .unwrap_or(&TileKind::Empty);

                    let kind = TileKind::try_from(value).unwrap();

                    let mut render = false;

                    match (prev, kind) {
                        (_, TileKind::Ball) => { render = true; last_ball_pos = Some(((x, y), round)); },
                        (_, TileKind::Paddle) => { render = true; last_paddle_pos = Some(((x, y), round)); },
                        (TileKind::Ball, TileKind::Empty) => render = false,
                        _ => {},
                    }

                    disp.insert(&(x, y), kind);

                    if render && last_ball_pos.as_ref().map(|(_, rnd)| *rnd) == last_paddle_pos.as_ref().map(|(_, rnd)| *rnd) {
                        println!("{}", disp);
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }

                program.handle_output_completion(io)
            },
        };
    }

    println!();

    score
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
