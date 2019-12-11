use intcode::{parse_stdin_program, Word, Memory, Program, ExecutionState, Registers};
use std::collections::HashMap;

fn main() {
    let data = parse_stdin_program();

    println!("{}", stage1(&data[..]));
    println!("{}", stage2(&data[..]));
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn rotate_left(&self) -> Self {
        match *self {
            Direction::Up => Direction::Left,
            Direction::Right => Direction::Up,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
        }
    }

    fn rotate_right(&self) -> Self {
        match *self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
struct Point<T> {
    x: T,
    y: T,
}

impl Point<isize> {
    fn move_at(self, d: Direction) -> Self {
        let (dx, dy) = match d {
            Direction::Up => (0, -1),
            Direction::Right => (1, 0),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
        };

        Point { x: self.x + dx, y: self.y + dy }
    }
}

enum RobotState {
    PaintCommand,
    DirectionedMove,
}

#[derive(PartialEq, Eq, Debug)]
enum Color {
    White,
    Black,
}

// number of painted at least once areas
fn stage1(data: &[Word]) -> usize {
    registration_paint(data, Color::Black).0.len()
}

fn stage2(data: &[Word]) -> String {
    let (painted, min, max) = registration_paint(data, Color::White);

    // min is lower left corner, max is upper right

    let top_left = Point { x: min.x.min(max.x), y: min.y.max(max.y) };
    let lower_right = Point { x: min.x.max(max.x), y: max.y.min(min.y) };

    let mut ret = String::new();

    for y in lower_right.y ..= top_left.y {
        for x in top_left.x ..= lower_right.x {
            let color = painted.get(&Point { x, y }).unwrap_or(&Color::Black);
            ret += if color == &Color::Black { "X" } else { " " };
        }

        ret += "\n"
    }

    ret
}

fn registration_paint(data: &[Word], start_on: Color) -> (HashMap<Point<isize>, Color>, Point<isize>, Point<isize>) {
    let mut painted = HashMap::new();
    let mut program: Program = Memory::from(data)
        .with_memory_expansion()
        .into();

    let mut regs = Registers::default();

    let mut coords = Point { x: 0, y: 0 };
    painted.insert(coords, start_on);

    let mut min = coords;
    let mut max = coords;

    let mut direction = Direction::Up;

    let mut robot_state = RobotState::PaintCommand;

    loop {
        regs = match program.eval_from_instruction(regs).unwrap() {
            ExecutionState::Paused(regs) => regs,
            ExecutionState::HaltedAt(_regs) => break,
            ExecutionState::InputIO(io) => {
                let color = painted.get(&coords).unwrap_or(&Color::Black);
                program.handle_input_completion(
                    io,
                    (*color == Color::White) as Word,
                ).unwrap()
            },
            ExecutionState::OutputIO(io, value) => {
                assert!(value == 0 || value == 1);
                robot_state = match (robot_state, value) {
                    (RobotState::PaintCommand, x) => {
                        let color = if x == 0 {
                            Color::Black
                        } else {
                            Color::White
                        };
                        painted.insert(coords, color);
                        RobotState::DirectionedMove
                    },
                    (RobotState::DirectionedMove, x) => {
                        direction = if x == 0 {
                            direction.rotate_left()
                        } else {
                            direction.rotate_right()
                        };

                        let next_coords = coords.move_at(direction);
                        assert_ne!(next_coords, coords);
                        coords = next_coords;
                        min = Point { x: min.x.min(coords.x), y: min.y.min(coords.y) };
                        max = Point { x: max.x.max(coords.x), y: max.y.max(coords.y) };
                        RobotState::PaintCommand
                    },
                };

                program.handle_output_completion(io)
            }
        };
    }

    (painted, min, max)
}

#[test]
fn full_stage1() {
    intcode::with_parsed_program(|data| assert_eq!(stage1(data), 2883));
}

#[test]
fn full_stage2() {
    let expected =
"X XXXX    X   XXX  XX   XX XXXXX  XX    XXX
X XXXX XXXX XX X XX X XX X XXXX XX XXXX XXX
X XXXX   XX XX X XXXX XX X XXXX XXXXXX XXXX
X XXXX XXXX   XX XXXX   XX XXXX X  XX XXXXX
X XXXX XXXX XXXX XX X XXXX XXXX XX X XXXXXX
X    X    X XXXXX  XX XXXX    XX   X    XXX
";

    intcode::with_parsed_program(|data| assert_eq!(stage2(data), expected));
}
