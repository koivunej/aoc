use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt;
use intcode::{util::{parse_stdin_program_n_lines, GameDisplay}, Program, Environment, Word, ExecutionState, Registers};

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

impl Default for TileKind {
    fn default() -> Self {
        TileKind::Empty
    }
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

    let mut score = 0;

    let mut last_ball_pos: Option<((Word, Word), usize)> = None;
    let mut last_paddle_pos: Option<((Word, Word), usize)> = None;

    let mut round = 0;

    loop {
        regs = match program.eval_from_instruction(regs).unwrap() {
            ExecutionState::Paused(_regs) => unreachable!("Pausing not implemented yet?"),
            ExecutionState::HaltedAt(_regs) => break,
            ExecutionState::InputIO(io) => {

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
                        print!("{}{}{}", termion::clear::All, termion::cursor::Goto(1, 1), disp);
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
