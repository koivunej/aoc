use std::fmt;
use std::collections::{VecDeque, HashSet};
use intcode::{Word, util::{parse_stdin_program, GameDisplay}, Program, Registers, ExecutionState};

fn main() {
    let mut robot = Robot::new(parse_stdin_program());
    let mut gd: GameDisplay<Tile> = GameDisplay::default();

    gd.insert(robot.position(), Tile::Empty);

    let mut oxygen_at = None;
    {
        let mut work = VecDeque::new();
        work.push_back(*robot.position());

        while let Some(root) = work.pop_front() {

            // perhaps robot.travel(&root, &gd)?
            while root != *robot.position() {
                match path_to(&gd, robot.position(), &root) {
                    Some(directions) => {
                        for d in directions {
                            let prev = *robot.position();
                            let (moved_to, _) = robot.try_move(d);
                            assert_ne!(prev, moved_to);
                        }
                    },
                    None => panic!("Cannot get from {:?} to {:?}", robot.position(), root),
                }
            }

            let unexplored = Direction::all()
                .map(|d| (d, root.step_in_direction(&d)))
                .filter_map(|(d, p)| match gd.get(&p) {
                    Some(Tile::Unexplored) | None => Some((d, p)),
                    Some(_) => None,
                })
                .collect::<Vec<_>>();

            for (d, target) in unexplored {
                let (ended_up, tile) = robot.try_move(d);

                if tile == Tile::Oxygen {
                    assert!(oxygen_at.is_none());
                    oxygen_at = Some(ended_up);
                }

                if target == ended_up {
                    gd.insert(&target, tile);
                    // push to the same side as we are popping will decrease the amount of running
                    // around on the map so maybe depth first?
                    work.push_front(target);
                    let (back_at, _) = robot.try_move(d.reverse());
                    assert_eq!(back_at, root);
                } else {
                    gd.insert(&target, tile);
                }
            }
        }
    }

    println!("oxygen at: {:?}", oxygen_at);
    println!("robot moves: {}", robot.moves);

    println!("stage1: {}", path_to(&gd, &( 0, 0), oxygen_at.as_ref().unwrap()).unwrap().len());

    {
        // stage2 is probably just a dfs from the oxygen, mark the coordinates and ... push all new
        // marked ones to the queue?
        let mut frontier = VecDeque::new();
        let mut oxygen = HashSet::new();

        oxygen.insert(oxygen_at.unwrap());
        frontier.push_back((oxygen_at.unwrap(), 0));

        let mut prev_time = 0;

        let mut minutes = 0;

        while let Some((p1, time)) = frontier.pop_front() {

            oxygen.insert(p1);

            if prev_time != time {
                assert!(prev_time < time, "{} should be less than {}", prev_time, time);

                prev_time = time;
                minutes += 1;

                println!("{:>3} minutes ... {} slots oxygenated", minutes, oxygen.len());
            }

            let unoxinated = Direction::all()
                .map(|d| p1.step_in_direction(&d))
                .filter_map(|p| match gd.get(&p) {
                    Some(Tile::Empty) => Some(p),
                    Some(_) | None => None,
                })
                .filter(|p| !oxygen.contains(&p))
                .collect::<Vec<_>>();

            for p2 in unoxinated {
                frontier.push_back((p2, time + 1));
            }
        }

        println!("stage2: {}", minutes);
    }
}

/// Wasteful dijkstra ... could share the hashmaps across queries maybe?
fn path_to(gd: &GameDisplay<Tile>, pos: &(Word, Word), target: &(Word, Word)) -> Option<Vec<Direction>> {
    //println!("path_to: {:?} to {:?}", pos, target);
    use std::collections::{HashMap, BinaryHeap};
    use std::collections::hash_map::Entry;
    use std::cmp;

    let mut ret = Vec::new();

    let mut work = BinaryHeap::new();
    let mut dist = HashMap::new();
    let mut prev = HashMap::new();

    work.push(cmp::Reverse((0, *pos)));

    while let Some(cmp::Reverse((steps_here, p))) = work.pop() {

        //println!("path_to: popped {:?}", (p, steps_here));

        if p == *target {
            //println!("path_to: found target {:?}", p);

            let mut backwards = p;
            ret.push(p);

            while backwards != *pos {
                let previous = prev.remove(&backwards).unwrap();
                ret.push(previous);
                backwards = previous;
            }

            ret.reverse();

            let dirs = ret.windows(2)
                .map(|slice| {
                    let a = slice[0];
                    let b = slice[1];

                    let d = (b.0 - a.0, b.1 - a.1);

                    match d {
                        ( 0,-1) => Direction::Down,
                        ( 0, 1) => Direction::Up,
                        (-1, 0) => Direction::Left,
                        ( 1, 0) => Direction::Right,
                        x => unreachable!("cannot have this {:?} between {:?} and {:?}", x, a, b),
                    }
                }).collect();

            return Some(dirs);
        }

        match dist.entry(p) {
            Entry::Vacant(vcnt) => {
                vcnt.insert(steps_here);
            },
            Entry::Occupied(mut o) => {
                if *o.get() >= steps_here {
                    *o.get_mut() = steps_here;
                } else {
                    println!("already visited {:?} with lower dist {} than {} from {:?}", p, o.get(), steps_here, prev[&p]);
                    continue;
                }
            }
        }

        for (p2, dir) in adjacent(gd, &p) {
            let alt = steps_here + 1;

            if alt < *dist.get(&p2).unwrap_or(&usize::max_value()) {
                //println!("  {:?} --{:?}--> {:?}", p, dir, p2);
                dist.insert(p2, alt);
                prev.insert(p2, p);

                work.push(cmp::Reverse((alt, p2)));
            }
        }
    }

    None
}

#[test]
fn test_path_to() {
    use Direction::*;

    let mut gd: GameDisplay<Tile> = GameDisplay::default();
    gd.insert(&(-1, 0), Tile::Wall);
    gd.insert(&(-1,-1), Tile::Wall);
    gd.insert(&( 0,-1), Tile::Wall);
    gd.insert(&( 2, 0), Tile::Wall);
    gd.insert(&( 2,-1), Tile::Wall);

    gd.insert(&( 0, 0), Tile::Empty); // right
    gd.insert(&( 1, 0), Tile::Empty); // down
    gd.insert(&( 1, 1), Tile::Empty); // down
    gd.insert(&( 1, 2), Tile::Empty); // down
    gd.insert(&( 1, 3), Tile::Empty); // down
    gd.insert(&( 1, 4), Tile::Empty); // down
    gd.insert(&( 2, 4), Tile::Empty); // down
    gd.insert(&( 3, 4), Tile::Empty); // down
    gd.insert(&( 4, 4), Tile::Empty); // down
    gd.insert(&( 4, 3), Tile::Empty); // down
    gd.insert(&( 4, 2), Tile::Empty); // down
    gd.insert(&( 4, 1), Tile::Empty); // down
    gd.insert(&( 1,-1), Tile::Empty); // down
    gd.insert(&( 1,-2), Tile::Empty); // down
    gd.insert(&( 2,-2), Tile::Empty); // right
    gd.insert(&( 3,-2), Tile::Empty); // right
    gd.insert(&( 3,-1), Tile::Empty);
    gd.insert(&( 3, 0), Tile::Empty);
    gd.insert(&( 4, 0), Tile::Empty);

    println!("{}", gd);

    assert_eq!(vec![Right, Down, Down, Right, Right, Up, Up, Right], path_to(&gd, &( 0, 0), &( 4, 0)).unwrap());
}

fn adjacent<'a>(gd: &'a GameDisplay<Tile>, pos: &'a (Word, Word)) -> impl Iterator<Item = ((Word, Word), Direction)> + 'a {
    Direction::all()
        .into_iter()
        .map(move |d| (pos.step_in_direction(&d), d))
        .filter_map(move |(p2, d)| gd.get(&p2).map(|t| (p2, d, t)))
        //.inspect(|x| println!("  c: {:?}", x))
        .filter_map(|(p2, d, t)| match t {
            &Tile::Empty | &Tile::Robot | &Tile::Oxygen => Some((p2, d)),
            _ => None,
        })
        //.inspect(|x| println!("  d: {:?}", x))
}

struct Robot {
    program: Program<'static>,
    regs: Option<Registers>,
    pos: (Word, Word),
    moves: usize,
}

impl Robot {
    fn new(data: Vec<Word>) -> Self {
        let mem = intcode::Memory::from(data).with_memory_expansion();

        let program = Program::from(mem);

        Robot {
            program,
            regs: Some(Registers::default()),
            pos: (0, 0),
            moves: 0,
        }
    }

    fn position(&self) -> &(Word, Word) {
        &self.pos
    }

    fn try_move(&mut self, dir: Direction) -> ((Word, Word), Tile) {
        loop {
            let mut ret = None;

            self.regs = Some(match self.program.eval_from_instruction(self.regs.take().unwrap()).unwrap() {
                ExecutionState::HaltedAt(regs) => unreachable!("Halted at: {:?}", regs),
                ExecutionState::Paused(regs) => unreachable!("Paused? {:?}", regs),
                ExecutionState::InputIO(io) => {
                    let val: i64 = dir.into();
                    //println!("robot <-- {}", val);
                    self.program.handle_input_completion(io, val).unwrap()
                },
                ExecutionState::OutputIO(io, value) => {
                    //println!("robot --> {}", value);
                    let moved = value != 0;
                    let found = value == 2;
                    let tile = if found { Tile::Oxygen } else if moved { Tile::Empty } else { Tile::Wall };

                    let prev = self.pos;

                    if moved {
                        self.pos = self.pos.step_in_direction(&dir);
                        self.moves += 1;
                    }

                    // println!("robot movement from {:?} to {:?} ended up {:?}", prev, dir, self.pos);

                    ret = Some((self.pos, tile));

                    self.program.handle_output_completion(io)
                },
            });

            if let Some((pos, tile)) = ret {
                return (pos, tile);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Right,
    Down,
    Left
}

impl Into<Word> for Direction {
    fn into(self) -> Word {
        match self {
            Direction::Up => 1,
            Direction::Right => 3,
            Direction::Down => 2,
            Direction::Left => 4,
        }
    }
}

impl Direction {
    fn all() -> impl Iterator<Item = Direction> {
        use Direction::*;

        [Up, Right, Down, Left].into_iter().copied()
    }

    fn reverse(&self) -> Direction {
        use Direction::*;
        match *self {
            Up => Down,
            Right => Left,
            Down => Up,
            Left => Right,
        }
    }
}

trait Coordinates {
    fn step_in_direction(&self, dir: &Direction) -> Self;
}

impl Coordinates for (Word, Word) {
    fn step_in_direction(&self, dir: &Direction) -> Self {
        match *dir {
            Direction::Up => (self.0, self.1 + 1),
            Direction::Right => (self.0 + 1, self.1),
            Direction::Down => (self.0, self.1 - 1),
            Direction::Left => (self.0 - 1, self.1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Wall,
    Empty,
    Oxygen,
    Robot,
    Unexplored
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Unexplored
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let ch = match *self {
            Tile::Wall => '#',
            Tile::Empty => ' ',
            Tile::Oxygen => 'o',
            Tile::Robot => 'R',
            Tile::Unexplored => '?'
        };

        write!(fmt, "{}", ch)
    }
}
