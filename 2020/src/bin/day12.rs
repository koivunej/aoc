use std::convert::TryFrom;
use std::io::BufRead;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = String::new();

    let mut ship = Ship::default();
    let mut other = WaypointGuidedShip::default();

    loop {
        buffer.clear();

        let read = stdin.read_line(&mut buffer)?;

        if read == 0 {
            break;
        }

        let op = buffer.trim().parse::<Op>().unwrap();
        ship.execute(op);
        other.execute(op);

        println!("{:?}", other);
    }

    let part_one = ship.manhattan_distance();
    let part_two = other.manhattan_distance();

    println!("{}", part_one);
    println!("{}", part_two);
    assert_ne!(part_one, 997);
    // had misunderstood how left and right work
    assert_eq!(part_one, 923);
    // rotations took some time thanks to imaging a contrived coordinate system
    assert_eq!(part_two, 24769);

    Ok(())
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
