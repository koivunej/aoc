use std::fmt;
use std::convert::TryFrom;
use std::collections::{VecDeque, HashSet, HashMap};
use std::collections::hash_map::Entry;
use intcode::{Word, util::{parse_stdin_program_n_lines, GameDisplay}, Program, Registers, ExecutionState};

fn main() {
    let input = parse_stdin_program_n_lines(Some(1));
    let mut gd = ScaffoldProgram::new(input.clone()).print_map();
    {
        println!("part1: {}", alignment_parameters(&gd));
    }

    // for part2 probably:
    //  1. from initial parsed state (direction, pos) generate the commands
    //  2. go through the command list find the longest common substrings?
    //      * could be that the "at least once" is bad here, may need to insert some bogus "go back
    //      to here"

    let (main, a, b, c) = part2_program(&mut gd);
    println!("{:>8}: {}", "MAIN", Instructions(main.as_slice()));
    println!("{:>8}: {}", "A", Instructions(a.as_slice()));
    println!("{:>8}: {}", "B", Instructions(b.as_slice()));
    println!("{:>8}: {}", "C", Instructions(c.as_slice()));
    println!();

    println!("part2: {}", part2_dust_collected(main.as_slice(), a.as_slice(), b.as_slice(), c.as_slice(), input));
}

fn part2_dust_collected(main: &[Action], a: &[Action], b: &[Action], c: &[Action], mut data: Vec<Word>) -> Word {
    assert_eq!(data[0], 1);
    data[0] = 2;
    let mut program = Program::from(intcode::Memory::from(data).with_memory_expansion());
    let mut regs = Some(Registers::default());
    let input = format!("{}\n{}\n{}\n{}\nn\n", Instructions(main), Instructions(a), Instructions(b), Instructions(c));
    let mut input = input.chars();

    loop {
        regs = Some(match program.eval_from_instruction(regs.take().unwrap()).unwrap() {
            ExecutionState::HaltedAt(_) => todo!("no dust value was received?"),
            ExecutionState::Paused(regs) => unreachable!("Paused? {:?}", regs),
            ExecutionState::InputIO(io) => {
                let val: i64 = input.next().unwrap() as Word;
                program.handle_input_completion(io, val).unwrap()
            },
            ExecutionState::OutputIO(io, value) => {
                if value.abs() > 128 {
                    return value;
                }
                print!("{}", value as u8 as char);
                program.handle_output_completion(io)
            }
        });
    }
}

fn part2_program(gd: &mut GameDisplay<Tile>) -> (Vec<Action>, Vec<Action>, Vec<Action>, Vec<Action>) {
    println!("{}", gd);

    let robot_initially_at = gd.cells()
        .iter()
        .position(|t| if let &Tile::Robot(_) = t { true } else { false })
        .map(|index| gd.to_coordinates(index))
        .unwrap();

    let robot_initial_direction = gd.get(&robot_initially_at).and_then(Tile::robot_direction).unwrap();

    let visitable = gd.cells()
        .iter()
        .filter(|t| t.can_visit())
        .count();

    let mut work = binary_heap_plus::BinaryHeap::new_by(|a: &(_, _, _, _, HashSet<(Word, Word)>, _), b: &(_, _, _, _, HashSet<(Word, Word)>, _)| a.4.len().cmp(&b.4.len()));
    let intersections = HashMap::new();
    let mut seen = HashSet::new();
    seen.insert(robot_initially_at);

    work.push((robot_initially_at, robot_initial_direction, Vec::new(), intersections, seen, 3));

    let mut smallest = None;

    while let Some((pos, dir, actions_here, intersections, seen, gas)) = work.pop() {
        if seen.len() == visitable {
            smallest = match smallest.take() {
                None => { println!("found first solution: {}", actions_here.len()); Some(actions_here) },
                Some(other) if other.len() > actions_here.len() => { println!("found new best solution: {}", actions_here.len()); Some(actions_here) },
                Some(other) => Some(other)
            };
            continue;
        }

        let seen_before = seen.len();

        for (new_dir, mut new_pos) in frontier(gd, pos, dir)/*.inspect(|x| println!("frontier(_, {:?}, {:?}): {:?}", pos, dir, x))*/ {
            // this could probably be just consumed in a loop or folded to get count, the last step
            // and so on.
            // the steps +1 is because the frontier already took a step in the new_dir direction.
            let (final_pos, steps) = shortest_travel(gd, new_pos, new_dir).expect("frontier point should be travellable");

            let mut intersections = intersections.clone();
            let mut seen = seen.clone();

            let start_pos = new_pos;
            seen.insert(start_pos);

            for _ in 0..steps {
                new_pos = new_pos.step(new_dir);
                seen.insert(new_pos);
            }

            assert_eq!(new_pos, final_pos);

            if is_intersection(gd, new_pos) {
                match intersections.entry((new_pos, (dir, new_dir))) {
                    Entry::Occupied(mut oc) => {
                        if *oc.get() > 2 {
                            println!("filtering: path visited {:?} in {:?} more than {} times, visited {}", new_pos, new_dir.orientation(), *oc.get(), seen.len());
                            continue;
                        }
                        *oc.get_mut() += 1;
                    }
                    Entry::Vacant(vcnt) => { vcnt.insert(1); },
                }
            }

            let mut actions = actions_here.clone();
            actions.extend(ActionsBetweenDirections::from((dir, new_dir)));
            actions.push(Action::Move(steps + 1));

            let gas = if seen_before < seen.len() {
                gas
            } else if gas > 0 {
                gas - 1
            } else {
                continue;
            };

            work.push((new_pos, new_dir, actions, intersections, seen, gas));
        }
    }

    match smallest {
        Some(x) => {
            let instructions = Instructions(x.as_slice());
            instructions.compress()
        },
        None => unreachable!("no solutions"),
    }
}

fn shortest_travel(gd: &GameDisplay<Tile>, pos: (Word, Word), dir: Direction) -> Option<((Word, Word), usize)> {
    (1..)
        .map(|steps| (dir, steps))
        .scan(pos, |acc, (d, steps)| {
            if is_intersection(gd, *acc) {
                // do not travel over intersections, those are only crossed with frontier(..)
                None
            } else {
                *acc = acc.step(d);
                match gd.get(acc) {
                    Some(x) if x.can_visit() => Some((*acc, steps)),
                    _ => None
                }
            }
        })
        //.inspect(|x| println!("shortest_travel: {:?}", x))
        .last()
}

fn frontier<'a>(gd: &'a GameDisplay<Tile>, pos: (Word, Word), dir: Direction) -> impl Iterator<Item = (Direction, (Word, Word))> + 'a {
    [Direction::North, Direction::West, Direction::South, Direction::East]
        .into_iter()
        //.filter(|d| dir != **d)
        .map(move |dir| (*dir, pos.step(*dir)))
        .filter_map(move |(dir, pos_after_turning)| gd.get(&pos_after_turning).map(|t| (dir, pos_after_turning, t)))
        .filter_map(|(d, p, t)| match t {
            Tile::Scaffolding | Tile::Robot(_) => Some((d, p)),
            _ => None,
        })
        .filter(move |(new_dir, _)| *new_dir != dir.reverse())
}

struct Instructions<'a>(&'a [Action]);

impl<'a> fmt::Display for Instructions<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        let elements = self.combine_consecutive();

        let mut first = true;

        for action in elements {
            if !first {
                write!(fmt, ",")?;
            } else {
                first = false;
            }

            match action {
                Action::TurnLeft => write!(fmt, "L"),
                Action::TurnRight => write!(fmt, "R"),
                Action::Move(steps) => write!(fmt, "{}", steps),
                Action::Function(n) => write!(fmt, "{}", n),
            }?;
        }

        Ok(())
    }
}

impl<'a> Instructions<'a> {
    fn combine_consecutive(&self) -> impl Iterator<Item = Action> + 'a {
        self.0.iter()
            .map(Option::Some)
            .chain(std::iter::repeat(None).take(1))
            .scan(None, |state, action| {
                match (state.take(), action) {
                    (Some(Action::Move(n)), Some(Action::Move(m))) => {
                        *state = Some(Action::Move(n + m));
                        return Some(None);
                    },
                    (Some(x), Some(y)) => {
                        *state = Some(*y);
                        return Some(Some(x));
                    },
                    (None, Some(Action::Move(n))) => {
                        *state = Some(Action::Move(*n));
                        return Some(None);
                    }
                    (None, Some(y)) => Some(Some(*y)),
                    (Some(x), None) => Some(Some(x)),
                    (None, None) => None,
                }
            })
            .filter_map(|x| x)
            //.inspect(|x| println!("item: {:?}", x))
    }

    fn compress(&self) -> (Vec<Action>, Vec<Action>, Vec<Action>, Vec<Action>) {
        let limit = 20;

        let base = self.combine_consecutive().collect::<Vec<_>>();

        if 2 * base.len() - 1 < limit {
            (base, Vec::new(), Vec::new(), Vec::new())
        } else {

            let mut work = VecDeque::new();

            work.push_back(Compression { main: base, a: Vec::new(), b: Vec::new(), c: Vec::new() });

            let mut good = Vec::new();

            while let Some(Compression { main, a, b, c }) = work.pop_front() {

                if !c.is_empty() && 2 * main.len() - 1 < limit {
                    if let None = main.iter().position(|a| match a { Action::Function(_) => false, _ => true }) {
                        good.push(Compression { main, a, b, c });
                    }
                    continue;
                } else if !a.is_empty() && !b.is_empty() && !c.is_empty() {
                    continue;
                }

                //println!("{:>3}: [{}] [{}] [{}] [{}]", main.len(), Instructions(main.as_slice()), Instructions(a.as_slice()), Instructions(b.as_slice()), Instructions(c.as_slice()));

                let start = match main.iter().position(|x| match x { Action::Function(_) => false, _ => true }) {
                    Some(x) => x,
                    None => continue,
                };

                let end = main[start..].iter().position(|x| match x { Action::Function(_) => true, _ => false }).map(|x| start + x).unwrap_or(main.len());

                let interesting_a = &[Action::TurnRight, Action::Move(8), Action::TurnRight, Action::Move(8)];
                let interesting_b = &[Action::TurnRight, Action::Move(4), Action::TurnRight, Action::Move(4)];

                let interesting = a.as_slice() == interesting_a && b.as_slice() == interesting_b;

                for l in (2..=end - start).into_iter() {
                    let slice = &main[start..start + l];
                    //println!("start = {}, l = {}, looking for {}", start, l, Instructions(slice));

                    let mut find_start = start;

                    let mut segments = Vec::new();
                    //segments.push(find_start..find_start+l);

                    while find_start + l <= main.len() {
                        let haystack = &main[find_start..find_start + l];
                        assert_eq!(slice.len(), haystack.len());
                        if haystack == slice {
                            //println!("found {} at {}", Instructions(slice), find_start);
                            segments.push(find_start..find_start + l);
                            find_start += l;
                        } else {
                            find_start += 1;
                        }
                    }

                    if segments.len() > 0 {
                        let act = Action::Function(
                            match (&a[..], &b[..], &c[..]) {
                                ([], _, _) => 'A',
                                (_, [], _) => 'B',
                                (_, _, []) => 'C',
                                _ => unreachable!(),
                            }
                        );

                        let mut new_main = Vec::new();

                        let mut i = 0;
                        let mut replacements = 0;
                        for segment in segments {
                            //println!("segment: {:?}, i={}", segment, i);
                            while i < segment.start {
                                new_main.push(main[i]);
                                i += 1;
                            }
                            new_main.push(act);
                            i += l;
                            assert_eq!(i, segment.end);
                            replacements += 1;
                        }

                        while i < main.len() {
                            new_main.push(main[i]);
                            i += 1;
                        }

                        let repeated = main[start..start + l].to_vec();
                        if interesting {
                            //println!("copied\n{:>10}: {}\n{:>10}: {}\n{:>10}: {}", "old main", Instructions(main.as_slice()), "new main", Instructions(new_main.as_slice()), "repeated", Instructions(repeated.as_slice()));
                        }
                        assert_eq!(new_main.len() + replacements * (repeated.len() - 1), main.len());

                        let (new_a, new_b, new_c) = match (&a[..], &b[..], &c[..]) {
                            ([], _, _) => (repeated, Vec::new(), Vec::new()),
                            (_, [], _) => (a.clone(), repeated, Vec::new()),
                            (_, _, []) => (a.clone(), b.clone(), repeated),
                            (_, _, _) => unreachable!(),
                        };

                        work.push_back(Compression { main: new_main, a: new_a, b: new_b, c: new_c });
                    }
                }
            }

            good.sort_by_key(|c| {
                let total = (c.main.len() + c.a.len() + c.b.len() + c.c.len()) as isize;
                let avg = total / 4;

                (c.main.len() as isize - avg).pow(2)
                    + (c.a.len() as isize - avg).pow(2)
                    + (c.b.len() as isize - avg).pow(2)
                    + (c.c.len() as isize - avg).pow(2)
            });

            //good.truncate(1);

            let Compression { main, a, b, c } = good.remove(0);

            (main, a, b, c)
        }
    }
}

struct Compression {
    main: Vec<Action>,
    a: Vec<Action>,
    b: Vec<Action>,
    c: Vec<Action>,
}

impl fmt::Debug for Compression {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{{ main: {}, a: {}, b: {}, c: {} }}",
            Instructions(self.main.as_slice()),
            Instructions(self.a.as_slice()),
            Instructions(self.b.as_slice()),
            Instructions(self.c.as_slice()))
    }
}

fn is_intersection(gd: &GameDisplay<Tile>, pos: (Word, Word)) -> bool {
    use Direction::*;

    [North, West, South, East]
        .into_iter()
        .map(|d| pos.step(*d))
        .filter_map(|p| gd.get(&p))
        .filter(|t| t.can_visit())
        .count() == 4
}

trait Coordinates {
    fn step(&self, d: Direction) -> Self;
}

impl Coordinates for (Word, Word) {
    fn step(&self, d: Direction) -> Self {
        match d {
            Direction::North => (self.0, self.1 - 1),
            Direction::West => (self.0 + 1, self.1),
            Direction::South => (self.0, self.1 + 1),
            Direction::East => (self.0 - 1, self.1),
        }
    }
}

impl Direction {
    fn orientation(&self) -> Orientation {
        use Direction::*;
        match *self {
            North | South => Orientation::Vertical,
            East | West => Orientation::Horizontal,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Orientation {
    Horizontal,
    Vertical
}


struct ActionsBetweenDirections {
    from: Direction,
    to: Direction,
    last: Direction,
}

impl From<(Direction, Direction)> for ActionsBetweenDirections {
    fn from((from, to): (Direction, Direction)) -> Self {
        ActionsBetweenDirections {
            from,
            to,
            last: from,
        }
    }
}

impl Iterator for ActionsBetweenDirections {
    type Item = Action;

    fn next(&mut self) -> Option<Self::Item> {
        use Direction::*;
        if self.last == self.to {
            return None;
        }
        let (next_from, ret) = match (self.from, self.to) {
            (North, West)
                | (North, South)
                | (West, South)
                | (West, East)
                | (South, East)
                | (East, North) => (self.last.to_right(), Some(Action::TurnRight)),
            (North, East)
                | (West, North)
                | (South, North)
                | (South, West)
                | (East, West)
                | (East, South) => (self.last.to_left(), Some(Action::TurnLeft)),
            z => unreachable!("not sure how to get here: {:?}", z),
        };

        self.last = next_from;
        //println!("{:?}..={:?}: {:?} and {:?}", self.from, self.to, self.last, ret);
        ret
    }
}

#[test]
fn test_actions_between_directions() {
    use Direction::*;
    let all = [North, West, South, East];
    for dir in all.iter().copied() {
        assert!(ActionsBetweenDirections::from((dir, dir)).next().is_none());
    }

    let data = [
        (North, West, 1),
        (West, South, 1),
        (South, East, 1),
        (East, North, 1),
        (North, East, 1),
        (East, South, 1),
        (South, West, 1),
        (North, South, 2),
        (West, East, 2),
        (South, North, 2),
        (East, West, 2)
    ];

    for (from, to, expected) in data.into_iter() {
        let iter = ActionsBetweenDirections::from((*from, *to));
        assert_eq!(iter.count(), *expected, "{:?}..={:?}", from, to);

    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    TurnLeft,
    TurnRight,
    Move(usize),
    Function(char),
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
        let program = Program::from(intcode::Memory::from(data).with_memory_expansion());

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
                ExecutionState::HaltedAt(_) => return gd,
                ExecutionState::Paused(regs) => unreachable!("Paused? {:?}", regs),
                ExecutionState::InputIO(_io) => unreachable!("No input expected in print_map?"),
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
enum Direction {
    West,
    North,
    East,
    South,
}

impl Direction {
    fn to_left(&self) -> Direction {
        use Direction::*;
        match *self {
            North => East,
            West => North,
            South => West,
            East => South,
        }
    }

    fn to_right(&self) -> Direction {
        use Direction::*;
        match *self {
            North => West,
            West => South,
            South => East,
            East => North,
        }
    }

    fn reverse(&self) -> Direction {
        use Direction::*;
        match *self {
            North => South,
            South => North,
            West => East,
            East => West,
        }
    }
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

    fn robot_direction(&self) -> Option<Direction> {
        match *self {
            Tile::Robot(d) => Some(d),
            _ => None,
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
            x => return Err(x as char),
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

#[test]
fn part2_example() {
    let input = "\
#######...#####
#.....#...#...#
#.....#...#...#
......#...#...#
......#...###.#
......#.....#.#
^########...#.#
......#.#...#.#
......#########
........#...#..
....#########..
....#...#......
....#...#......
....#...#......
....#####......";

    let mut gd = input.chars()
        .scan((0, 0), |pos, ch| {

            if ch == '\n' {
                *pos = (0, pos.1 + 1);
                Some(None)
            } else {
                let old_pos = *pos;
                pos.0 += 1;

                Some(Some((old_pos, Tile::try_from(ch as u8).unwrap())))
            }
        })
        .filter_map(|x| x)
        .fold(GameDisplay::default(), |mut gd, (pos, tile)| {
            gd.insert(&pos, tile);
            gd
        });

    let intersections = gd.cells().iter()
        .enumerate()
        .filter(|(_, t)| t.can_visit())
        .map(|(i, tile)| (gd.to_coordinates(i), tile))
        .filter(|(p, _)| is_intersection(&gd, *p))
        //.inspect(|x| println!("intersection: {:?}", x))
        ;

    assert_eq!(intersections.clone().count(), 4);

    for (p, _) in intersections {
        for d in [Direction::North, Direction::West, Direction::South, Direction::East].into_iter() {
            // being able to head to all but the direction we came from sounds good
            assert_eq!(frontier(&gd, p, *d).count(), 3)
        }
    }

    assert_eq!(shortest_travel(&gd, (1, 6), Direction::West).unwrap(), ((6, 6), 5));

    let (main, a, b, c) = part2_program(&mut gd);

    assert_eq!(format!("{}", Instructions(main.as_slice())).as_str(), "A,B,C,B,A,C");
    assert_eq!(format!("{}", Instructions(a.as_slice())).as_str(), "R,8,R,8");
    assert_eq!(format!("{}", Instructions(b.as_slice())).as_str(), "R,4,R,4");
    assert_eq!(format!("{}", Instructions(c.as_slice())).as_str(), "R,8,L,6,L,2");
}

#[test]
fn format_instructions() {
    let input = Instructions(&[Action::TurnLeft, Action::Move(2), Action::Move(4), Action::TurnRight]);

    assert_eq!(format!("{}", input).as_str(), "L,6,R");
}
