use std::str::FromStr;
fn main() {
    // N wires
    // some origin (possibly 1,1 for coords zero at lower left, lower right)
    // they may have N crossings but we are interested in the closest one to the origin
}

struct Wire<T>(Vec<LineSegment<T>>);

impl<T> std::default::Default for Wire<T> {
    fn default() -> Self {
        Wire(vec![])
    }
}

impl<T> Wire<T> {
    fn begin(starting_point: &Point<T>) -> WireBuilder<T> {
        unimplemented!();
    }
}

struct WireBuilder<T> {
    starting_point: Point<T>,
    wire: Option<Wire<T>>,
}

impl<T> WireBuilder<T> {
    fn push(self, command: PenCommand) -> Wire<T> {
        unimplemented!()
        /*let mut w = Wire::default();
        w.push((self.starting_point, command.travel_from(self.starting_point)));
        w*/
    }
}

struct LineSegment<T>(Point<T>, Point<T>);

impl<T> From<(Point<T>, Point<T>)> for LineSegment<T> {
    fn from((start, end): (Point<T>, Point<T>)) -> Self {
        Self(start, end)
    }
}

struct Point<T> { x: T, y: T }

#[derive(Debug, PartialEq, Clone, Copy)]
enum Direction { Up, Right, Down, Left }

#[derive(Debug, PartialEq)]
enum ParseDirectionError {
    InvalidCharacter,
    WrongLength,
}

impl FromStr for Direction {
    type Err = ParseDirectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "U" => Direction::Up,
            "R" => Direction::Right,
            "D" => Direction::Down,
            "L" => Direction::Left,
            x if x.len() == 1 => return Err(ParseDirectionError::InvalidCharacter),
            _ => return Err(ParseDirectionError::WrongLength),
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct PenCommand(Direction, usize);

#[derive(Debug, PartialEq)]
enum ParsePenCommandError {
    EmptyInput,
    Direction(ParseDirectionError),
    Amount(std::num::ParseIntError),
}

impl From<ParseDirectionError> for ParsePenCommandError {
    fn from(e: ParseDirectionError) -> Self {
        ParsePenCommandError::Direction(e)
    }
}

impl From<std::num::ParseIntError> for ParsePenCommandError {
    fn from(e: std::num::ParseIntError) -> Self {
        ParsePenCommandError::Amount(e)
    }
}

impl FromStr for PenCommand {
    type Err = ParsePenCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let c = chars.next().ok_or(ParsePenCommandError::EmptyInput)?;
        let (head, tail) = s.split_at(c.len_utf8());
        let dir = Direction::from_str(head)?;
        let amount = usize::from_str(tail)?;
        Ok(PenCommand(dir, amount))
    }
}

#[test]
fn pencommand_examples() {

    let invalid = "a".parse::<u8>().unwrap_err();

    let values = &[
        ("R8", Ok(PenCommand(Direction::Right, 8))),
        ("U5", Ok(PenCommand(Direction::Up, 5))),
        ("L4", Ok(PenCommand(Direction::Left, 4))),
        ("D3", Ok(PenCommand(Direction::Down, 3))),
        ("A", Err(ParsePenCommandError::Direction(ParseDirectionError::InvalidCharacter))),
        ("RA", Err(ParsePenCommandError::Amount(invalid))),
    ];

    for (input, expected) in values.iter() {
        assert_eq!(PenCommand::from_str(input), *expected);
    }
}
