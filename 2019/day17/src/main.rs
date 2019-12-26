use std::fmt;
use std::convert::TryFrom;
use std::collections::{VecDeque, HashSet};
use intcode::{Word, util::{parse_stdin_program, GameDisplay}, Program, Registers, ExecutionState};

fn main() {
    let mut prog = ScaffoldProgram::new(parse_stdin_program());
    let gd = prog.print_map();
    println!("part1: {}", alignment_parameters(&gd));
}

fn alignment_parameters(gd: &GameDisplay<Tile>) -> i64 {
    let mut pos = (0, 0);
    loop {
        if let Some(Tile::Scaffolding) = gd.get(&pos) {
            break;
        }
        pos.0 += 1;
    }

    let mut visited = HashSet::new();
    let mut pending: VecDeque<(Option<(Word, Word)>, (Word, Word))> = VecDeque::new();

    pending.push_back((None, pos));

    let mut candidates = Vec::new();

    while let Some((prev, pos)) = pending.pop_front() {

        if !visited.insert(pos) {
            continue;
        }

        let next = [
            (0, 1),
            (1, 0),
            (0, -1),
            (-1, 0)
        ].into_iter()
            .map(|d| (pos.0 + d.0, pos.1 + d.1))
            .filter(|p| Some(*p) != prev)
            .filter_map(|p| gd.get(&p).and_then(|t| if t.can_visit() { Some(p) } else { None }));

        let mut count = 0;
        for n in next {
            pending.push_back((Some(pos), n));
            count += 1;
        }

        if count > 1 {
            candidates.push(pos.0 * pos.1);
        }
    }

    candidates.into_iter().sum()
}

#[derive(Clone)]
struct ScaffoldProgram {
    program: Program<'static>,
    regs: Option<Registers>,
}

impl ScaffoldProgram {
    fn new(data: Vec<Word>) -> Self {
        let mem = intcode::Memory::from(data).with_memory_expansion();

        let program = Program::from(mem);

        Self {
            program,
            regs: Some(Registers::default()),
        }
    }

    fn print_map(mut self) -> GameDisplay<Tile> {
        let mut gd = GameDisplay::default();
        let mut pos = (0, 0);
        loop {
            self.regs = Some(match self.program.eval_from_instruction(self.regs.take().unwrap()).unwrap() {
                ExecutionState::HaltedAt(regs) => return gd,
                ExecutionState::Paused(regs) => unreachable!("Paused? {:?}", regs),
                ExecutionState::InputIO(io) => {
                    unreachable!("No input expected in print_map?");
                    /*
                    let val: i64 = dir.into();
                    self.program.handle_input_completion(io, val).unwrap()
                    */
                },
                ExecutionState::OutputIO(io, value) => {
                    let value = value as u8;
                    if value == b'\n' {
                        pos = (0, pos.1 + 1);
                    } else {
                        gd.insert(&pos, Tile::try_from(value).unwrap());
                        pos.0 += 1;
                    }
                    self.program.handle_output_completion(io)
                }
            });
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Tile {
    Scaffolding,
    Empty,
    Robot(Direction),
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Direction {
    North,
    West,
    South,
    East,
}

impl Default for Tile {
    fn default() -> Self { Tile::Empty }
}

impl Tile {
    fn can_visit(&self) -> bool {
        match *self {
            Tile::Empty => false,
            _ => true,
        }
    }
}

impl TryFrom<u8> for Tile {
    type Error = char;

    fn try_from(ascii: u8) -> Result<Self, Self::Error> {
        Ok(match ascii {
            b'#' => Tile::Scaffolding,
            b'.' => Tile::Empty,
            b'^' => Tile::Robot(Direction::North),
            b'>' => Tile::Robot(Direction::West),
            b'<' => Tile::Robot(Direction::East),
            b'v' => Tile::Robot(Direction::South),
            x => return Err(ascii as char),
        })
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let ch = match *self {
            Tile::Scaffolding => '#',
            Tile::Empty => '.',
            Tile::Robot(Direction::North) => '^',
            Tile::Robot(Direction::West) => '>',
            Tile::Robot(Direction::East) => '<',
            Tile::Robot(Direction::South) => 'v',
        };

        write!(fmt, "{}", ch)
    }
}
