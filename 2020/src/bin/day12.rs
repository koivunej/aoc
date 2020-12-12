use std::convert::TryFrom;
use std::fmt;
use std::io::BufRead;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let stdin = std::io::stdin();

    let mut adapter = Adapter::<_, Op>::new(stdin.lock());

    let (part_one, part_two) = adapter
        .try_fold(
            (Ship::default(), WaypointGuidedShip::default()),
            |(mut a, mut b), op| match op {
                Ok(op) => {
                    a.execute(op);
                    b.execute(op);
                    Ok((a, b))
                }
                Err(e) => Err(e),
            },
        )
        .unwrap();

    let part_one = part_one.manhattan_distance();
    let part_two = part_two.manhattan_distance();

    println!("{}", part_one);
    println!("{}", part_two);
    assert_ne!(part_one, 997);
    // had misunderstood how left and right work
    assert_eq!(part_one, 923);
    // rotations took some time thanks to imaging a contrived coordinate system
    assert_eq!(part_two, 24769);

    Ok(())
}

struct Adapter<I, T> {
    input: I,
    buffer: String,
    _type_of_t: std::marker::PhantomData<T>,
}

impl<I: BufRead, T: FromStr> Adapter<I, T> {
    fn new(input: I) -> Self {
        Adapter {
            input,
            buffer: String::new(),
            _type_of_t: Default::default(),
        }
    }
}

#[derive(Debug)]
enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A: fmt::Display, B: fmt::Display> fmt::Display for Either<A, B> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Either::Left(e) => write!(fmt, "{}: {}", std::any::type_name::<A>(), &e),
            Either::Right(e) => write!(fmt, "{}: {}", std::any::type_name::<B>(), &e),
        }
    }
}

impl<A: std::error::Error + 'static, B: std::error::Error + 'static> std::error::Error
    for Either<A, B>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Either::Left(e) => Some(e),
            Either::Right(e) => Some(e),
        }
    }
}

impl<I, T> Iterator for Adapter<I, T>
where
    I: BufRead,
    T: FromStr + 'static,
    T::Err: 'static,
{
    type Item = Result<T, Either<T::Err, std::io::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();
        let read = self.input.read_line(&mut self.buffer);
        match read {
            Ok(0) => None,
            Ok(_) => match T::from_str(self.buffer.trim()) {
                Ok(t) => Some(Ok(t)),
                Err(e) => Some(Err(Either::Left(e))),
            },
            Err(e) => Some(Err(Either::Right(e))),
        }
    }
}

#[derive(Debug)]
struct Ship {
    position: (i16, i16),
    direction: i16,
}

impl Default for Ship {
    fn default() -> Ship {
        Ship {
            position: (0, 0),
            direction: 270,
        }
    }
}

impl Ship {
    fn execute(&mut self, op: Op) {
        use Op::*;
        match op {
            Move(cdir, amt) => self.do_move(cdir, amt),
            Turn(LeftRight::Left, amt) => {
                self.direction += amt;
            }
            Turn(LeftRight::Right, amt) => {
                self.direction -= amt;
            }
            Forward(amt) => {
                let dir = self.direction.rem_euclid(360);
                let cdir = CardinalDirection4::try_from(dir).expect("uneven");
                self.do_move(cdir, amt)
            }
        }
    }

    fn do_move(&mut self, cdir: CardinalDirection4, amt: i16) {
        let diff = cdir.to_coordinate_difference();
        let diff = (diff.0 * amt, diff.1 * amt);
        self.position = (self.position.0 + diff.0, self.position.1 + diff.1);
    }

    fn manhattan_distance(self) -> i16 {
        self.position.0.abs() + self.position.1.abs()
    }
}

#[derive(Debug)]
struct WaypointGuidedShip {
    waypoint: (i16, i16),
    position: (i16, i16),
}

impl Default for WaypointGuidedShip {
    fn default() -> WaypointGuidedShip {
        WaypointGuidedShip {
            waypoint: (-10, -1),
            position: (0, 0),
        }
    }
}

impl WaypointGuidedShip {
    fn execute(&mut self, op: Op) {
        use Op::*;
        match op {
            Move(cdir, amt) => {
                let diff = cdir.to_coordinate_difference();
                let diff = (diff.0 * amt, diff.1 * amt);
                self.waypoint = (self.waypoint.0 + diff.0, self.waypoint.1 + diff.1);
            }
            Turn(d, amt) => {
                // R90 -- clockwise 90
                // from: (-10, -4)
                // to:   ( -4, 10)

                assert_eq!(amt % 90, 0);
                let times = amt / 90;

                for _ in 0..times {
                    // apologies, not good with geometry
                    if d == LeftRight::Right {
                        self.waypoint = (self.waypoint.1, -self.waypoint.0);
                    } else {
                        self.waypoint = (-self.waypoint.1, self.waypoint.0);
                    }
                }
            }
            Forward(amt) => {
                let d = (self.waypoint.0 * amt, self.waypoint.1 * amt);
                self.position = (self.position.0 + d.0, self.position.1 + d.1);
            }
        }
    }

    fn manhattan_distance(self) -> i16 {
        self.position.0.abs() + self.position.1.abs()
    }
}

#[derive(Clone, Copy)]
enum Op {
    Move(CardinalDirection4, i16),
    Turn(LeftRight, i16),
    Forward(i16),
}

impl FromStr for Op {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let amount = s[1..].parse::<i16>().map_err(|_| ())?;

        Ok(match s.as_bytes()[0] {
            b'N' | b'W' | b'S' | b'E' => Op::Move(CardinalDirection4::from_str(&s[0..1])?, amount),
            b'L' | b'R' => Op::Turn(LeftRight::from_str(&s[0..1])?, amount),
            b'F' => Op::Forward(amount),
            _ => return Err(()),
        })
    }
}

// TODO: move to lib
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum CardinalDirection4 {
    North,
    West,
    South,
    East,
}

impl CardinalDirection4 {
    fn to_coordinate_difference(&self) -> (i16, i16) {
        use CardinalDirection4::*;
        match self {
            North => (0, -1),
            West => (1, 0),
            South => (0, 1),
            East => (-1, 0),
        }
    }
}

impl TryFrom<i16> for CardinalDirection4 {
    type Error = i16;

    fn try_from(heading: i16) -> Result<Self, Self::Error> {
        use CardinalDirection4::*;
        Ok(match heading {
            0 => North,
            90 => West,
            180 => South,
            270 => East,
            x => return Err(x),
        })
    }
}

impl FromStr for CardinalDirection4 {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use CardinalDirection4::*;
        Ok(match s.as_bytes()[0] {
            b'N' => North,
            b'W' => West,
            b'S' => South,
            b'E' => East,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum LeftRight {
    Left,
    Right,
}

impl FromStr for LeftRight {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use LeftRight::*;
        Ok(match s.as_bytes()[0] {
            b'L' => Left,
            b'R' => Right,
            _ => return Err(()),
        })
    }
}
