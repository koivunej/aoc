use intcode::{parse_stdin_program, Word, Memory, Program, ExecutionState, Registers};
use std::collections::HashMap;

fn main() {
    let data = parse_stdin_program();

    println!("{}", stage1(&data[..]));
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Point<T> {
    x: T,
    y: T,
}

impl Point<isize> {
    fn move_at(self, d: Direction) -> Self {
        let (dx, dy) = match d {
            Direction::Up => (0, 1),
            Direction::Right => (1, 0),
            Direction::Down => (0, -1),
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
    let mut painted = HashMap::new();
    let mut program: Program = Memory::from(data)
        .with_memory_expansion()
        .into();

    let mut regs = Registers::default();

    let mut coords = Point { x: 0, y: 0 };
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
                        RobotState::PaintCommand
                    },
                };

                program.handle_output_completion(io)
            }
        };
    }

    painted.len()
}
