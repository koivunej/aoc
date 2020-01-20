use std::rc::Rc;
use std::fmt;
use std::fmt::Write;
use std::convert::TryFrom;
use std::collections::{HashMap, HashSet};
use intcode::{Word, InvalidProgram, util::{parse_stdin_program_n_lines, GameDisplay, Position, Direction}, Program, Registers, ExecutionState};
use itertools::Itertools;

fn main() {
    let data = parse_stdin_program_n_lines(Some(1));

    let mut game = Game::new(&data);
    let mut taboos = HashSet::new();

    loop {
        match game.play(&taboos) {
            Ok(password) => println!("part1: {}", password),
            Err(GameFailure::InfiniteLoop(why))
            | Err(GameFailure::UnexpectedHalt(why))
            | Err(GameFailure::Stuck(why)) => {
                println!("learned new taboo item: {:?} because \"{}\"", game.last_picked_up_item(), why);
                println!("---\n{}", game.read_buffer);
                println!("---");
                taboos.insert(game.last_picked_up_item().unwrap().into());
            }
            Err(e) => panic!("unexpected {:?}", e),
        }

        game.reset_from(&data);
    }
}

#[derive(Debug)]
enum GameFailure {
    InvalidProgram(InvalidProgram),
    InvalidCardinalDirection(String),
    UnexpectedHalt(String),
    InfiniteLoop(String),
    Stuck(String),
}

impl From<InvalidProgram> for GameFailure {
    fn from(ip: InvalidProgram) -> Self {
        Self::InvalidProgram(ip)
    }
}


#[derive(Clone, Debug)]
enum EjectionReason {
    Light,
    Heavy,
}

#[derive(Clone, Debug)]
struct Ejection {
    reason: EjectionReason,
    ejected_from: Box<Room>,
}

#[derive(Debug, Clone)]
struct Room {
    name: Rc<String>,
    desc: String,
    doors: Vec<CardinalDirection>,
    items: Vec<Rc<String>>,
    ejection: Option<Ejection>,
}

impl Room {
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    VisitedRoom,
    UnvisitedRoom,
    Wall,
    Space,
    Drone,
}

impl Default for Tile {
    fn default() -> Self { Tile::Space }
}

impl fmt::Display for Tile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Tile::VisitedRoom => '.',
            Tile::UnvisitedRoom => '?',
            Tile::Wall => '#',
            Tile::Space => ' ',
            Tile::Drone => 'd',
        };

        write!(fmt, "{}", s)
    }
}

struct Game {
    prog: Program<'static>,
    regs: Option<Registers>,
    read_buffer: String,
    write_buffer: Option<String>,
    last_item: Option<String>,
}

impl Game {
    fn new(data: &[Word]) -> Self {
        Game {
            prog: Program::from(data.to_vec()).with_memory_expansion(),
            regs: Some(Default::default()),
            read_buffer: String::new(),
            write_buffer: Some(String::new()),
            last_item: None,
        }
    }

    fn reset_from(&mut self, data: &[Word]) {
        self.prog.reset_from(data);
        self.regs = Some(Default::default());
        self.read_buffer.clear();
        if let Some(b) = self.write_buffer.as_mut() {
            b.clear();
        } else {
            unreachable!("write buffer was lost?");
        }
        self.last_item = None;
    }

    fn last_picked_up_item(&self) -> Option<&str> {
        self.last_item.as_ref().map(String::as_str)
    }

    //fn explore(&mut self, taboo_items: &HashSet<String>, map: &mut Map) -> Result<String, GameFailure> {}

    fn play(&mut self, taboo_items: &HashSet<String>/*, known_rooms: &mut HashMap<Rc<String>, Room>*/) -> Result<String, GameFailure> {
        let mut inv: HashSet<Rc<String>> = HashSet::new();

        let mut work = VecDeque::new();

        let mut map = Map::default();

        let mut current_room = {
            let room = self.expect_room()?;

            for dir in &room.doors {
                work.push_back((Rc::clone(&room.name), *dir));
            }

            map.learn(None, &room.name, &room.doors);

            room.name
        };

        let mut path = Vec::new();

        while let Some((room_name, dir)) = work.pop_front() {

            path.clear();

            let exploring = if current_room != room_name {
                path.extend(map.path_to(&current_room, &room_name));
                false
            } else {
                path.push(dir);
                true
            };

            if !exploring {
                println!("moving from {:?} to {:?} with {} steps: {:?}", current_room.as_str(), room_name.as_str(), path.len(), path);
            } else {
                println!("exploring from {:?} to {}", current_room.as_str(), dir);
            }

            for dir in path.drain(..) {
                map.mark_visited(&current_room, dir);

                self.eval_input_by(|s| write!(s, "{}\n", dir).unwrap())?;

                let room = self.expect_room()?;

                if map.learn(Some((&current_room, dir)), &room.name, &room.doors) {
                    //println!("  {}: {:?}", room.name, room.desc);
                }

                for item in &room.items {
                    if taboo_items.contains(item.as_str()) {
                        continue;
                    }
                    if inv.contains(item) {
                        continue;
                    }
                    println!("  taking {:?}", item);
                    self.take_item(item)?;
                    inv.insert(item.clone());
                }

                let direction_back = dir.reverse();
                let gateway_to_checkpoint = room.desc == "In the next room, a pressure-sensitive floor will verify your identity.";

                for dir in &room.doors {

                    if map.is_visited(&room.name, *dir) {
                        //println!("  skipping already visited {} to {}", room.name, dir);
                        continue;
                    }

                    if *dir == direction_back {
                        continue;
                    }

                    if gateway_to_checkpoint && *dir != direction_back {
                        // just explore now, do not enter the checkpoints yet
                        continue;
                    }

                    // avoid bouncing by depth first
                    work.push_front((Rc::clone(&room.name), *dir));
                    //println!("  exploring later {} to {}", room.name, dir);
                }

                current_room = room.name;

                if gateway_to_checkpoint {
                    map.add_checkpoint(&current_room, dir);
                }
            }

            if !exploring {
                assert_eq!(current_room, room_name);
            } else {
                assert_ne!(current_room, room_name);
            }
        }

        {
            let (room, dir) = map.checkpoint_doors.iter().next().unwrap();
            let room = room.clone();
            let dir = dir.clone();

            path.extend(map.path_to(&current_room, &room));

            for dir in path.drain(..) {
                self.eval_input_by(|s| write!(s, "{}\n", dir).unwrap())?;
                let room = self.expect_room().unwrap();
                assert!(!map.learn(Some((&current_room, dir)), &room.name, &room.doors));
                current_room = room.name;
            }

            self.read_buffer.clear();
            self.eval_input_by(|s| write!(s, "inv\n").unwrap())?;
            self.read_until_prompt()?;
            println!("---\n{}", self.read_buffer);

            let mut items = Vec::with_capacity(inv.len());

            for item in inv.drain() {
                self.drop_item(&item)?;
                items.push(item);
            }

            let mut candidates = items.clone();

            let mut room = None;
            'out: for k in 1.. {
                for combination in (0..candidates.len()).combinations(k) {
                    for index in combination.iter() {
                        self.take_item(candidates[*index].as_str()).unwrap();
                    }

                    self.eval_input_by(|s| write!(s, "{}\n", dir).unwrap())?;
                    room = Some(self.expect_room().unwrap());

                    if room.as_ref().unwrap().ejection.is_none() {
                        println!("found combination: {}", candidates.iter().enumerate().filter_map(|(i, s)| if combination.contains(&i) { Some(s) } else { None }).join(","));
                        break 'out;
                    }

                    for index in combination {
                        self.drop_item(candidates[index].as_str()).unwrap();
                    }
                }
            }

            panic!("{:?}", room);
        }

        todo!("ran out of work? probably ok to explore first, inv: {:?}", inv)
    }

    fn drop_item(&mut self, item: &str) -> Result<(), GameFailure> {
        self.eval_input_by(|s| write!(s, "drop {}\n", item).unwrap())?;
        self.read_until_prompt()?;

        let buffer = self.read_buffer.trim();

        let mut lines = buffer.lines();
        let confirmation = lines.next().unwrap();

        assert_eq!(confirmation.len(), "You drop the ".len() + item.len() + 1);
        assert!(confirmation.starts_with("You drop the "));
        assert!(confirmation.ends_with("."));

        Ok(())
    }

    fn read_until_prompt(&mut self) -> Result<(), GameFailure> {
        self.read_buffer.clear();
        loop {
            let was_nl = match self.eval_until_output()? {
                MaybeAscii::Ascii(x) => {
                    self.read_buffer.push(x as char);
                    x == b'\n'
                },
                x => unreachable!("Unexpected {:?}", x),
            };

            if self.read_buffer.ends_with("Command?\n") {
                return Ok(());
            }

            if was_nl {
                // this is exponential but ... should get it quite fast
                let mut all_lines = HashMap::new();

                for s in self.read_buffer.lines().filter(|s| !s.is_empty()) {
                    let count = all_lines.entry(s).or_insert(0);
                    *count += 1;
                    if *count > 2 {
                        // need to allow two repetitions for ejection from security check
                        return Err(GameFailure::InfiniteLoop(s.into()));
                    }
                }
            }
        }
    }

    fn expect_room(&mut self) -> Result<Room, GameFailure> {
        self.read_until_prompt()?;

        self.parse_room()
    }

    fn parse_room(&mut self) -> Result<Room, GameFailure> {
        let buffer = self.read_buffer.trim();

        let mut lines = buffer.lines().filter(|s| !s.is_empty()).peekable();

        let name = lines.next().unwrap();

        if name.ends_with(" You can't move!!") {
            return Err(GameFailure::Stuck(String::from(&name[..name.len() - " You can't move!!".len() - 1])));
        }

        let name = name.split("==")
            .skip(1)
            .next()
            .map(str::trim)
            .unwrap_or_else(|| panic!("unexpected response?\n{}", buffer));

        let desc = lines.next().unwrap();

        match lines.next().unwrap() {
            "Doors here lead:" => {},
            x => panic!("expected doors, found {:?}", x),
        }

        let mut doors = Vec::new();

        loop {
            if !lines.peek().unwrap().starts_with("- ") {
                break;
            }

            let next = lines.next().unwrap();

            let start = next.char_indices().skip(2).next().unwrap().0;

            let dir = CardinalDirection::try_from(&next[start..]).map_err(|_| GameFailure::InvalidCardinalDirection(String::from(next)))?;
            doors.push(dir);
        }

        let mut items = Vec::new();

        if lines.peek().map(|s| s.ends_with("you are ejected back to the checkpoint.")).unwrap() {
            let reason_line = lines.next().unwrap();
            let reason = if reason_line.contains("lighter") {
                EjectionReason::Light
            } else {
                assert!(reason_line.contains("heavier"), "unexpected reason: {:?}", reason_line);
                EjectionReason::Heavy
            };

            let name = Rc::new(String::from(name));
            let desc = String::from(desc);

            self.read_buffer = String::from(&self.read_buffer[(self.read_buffer.find(reason_line).unwrap() + reason_line.len())..]);

            let mut final_room = self.parse_room()?;
            final_room.ejection = Some(Ejection {
                reason,
                ejected_from: Box::new(Room {
                    name,
                    desc,
                    doors,
                    items: Vec::new(),
                    ejection: None,
                })
            });

            return Ok(final_room);
        }

        if lines.peek() != Some(&"Command?") {
            let items_header = lines.next().unwrap();

            assert_eq!(items_header, "Items here:");

            loop {
                if !lines.peek().unwrap().starts_with("- ") {
                    break;
                }

                let item = Rc::new(String::from(&lines.next().unwrap()[2..]));
                items.push(item);
            }
        }

        assert_eq!(lines.next(), Some("Command?"));

        Ok(Room {
            name: Rc::new(String::from(name)),
            desc: String::from(desc),
            doors,
            items,
            ejection: None,
        })
    }

    fn take_item(&mut self, item: &str) -> Result<(), GameFailure> {

        match self.last_item.as_mut() {
            Some(older) => {
                older.clear();
                older.push_str(item);
            },
            None => {
                self.last_item = Some(String::from(item));
            }
        }

        self.eval_input_by(|s| write!(s, "take {}\n", item).unwrap())?;

        self.read_until_prompt()?;

        let buffer = self.read_buffer.trim();

        let mut lines = buffer.lines().filter(|s| !s.is_empty());

        let confirmation = lines.next().unwrap();
        if confirmation.len() != "You take the ".len() + item.len() + 1 {
            panic!("{}", confirmation);
        }

        assert!(confirmation.starts_with("You take the "));
        assert!(confirmation.ends_with("."));

        assert_eq!(lines.next(), Some("Command?"));
        Ok(())
    }

    fn eval_input_by<F: FnOnce(&mut String)>(&mut self, formatter: F) -> Result<(), GameFailure> {
        let mut buffer = self.write_buffer.take().unwrap();
        buffer.clear();

        formatter(&mut buffer);

        let res = self.eval_input(&buffer);

        self.write_buffer = Some(buffer);

        res
    }

    fn eval_until_output(&mut self) -> Result<MaybeAscii, GameFailure> {
        loop {
            let output;
            self.regs = Some(match self.prog.eval_from_instruction(self.regs.take().unwrap())? {
                ExecutionState::Paused(_regs) => unreachable!("Pausing not implemented yet?"),
                ExecutionState::HaltedAt(_regs) => return Err(GameFailure::UnexpectedHalt(self.read_buffer.trim().replace("\n\n", " "))),
                ExecutionState::InputIO(_io) => unreachable!("Input was unexpected"),
                ExecutionState::OutputIO(io, value) => {
                    output = Some(value);
                    self.prog.handle_output_completion(io)
                },
            });

            if let Some(value) = output {
                return Ok(value.into());
            }
        }
    }

    fn eval_input(&mut self, input: &str) -> Result<(), GameFailure> {
        let mut consumed = 0;
        let mut chars = input.chars().peekable();
        loop {
            self.regs = Some(match self.prog.eval_from_instruction(self.regs.take().unwrap())? {
                ExecutionState::Paused(_regs) => unreachable!("Pausing not implemented yet?"),
                ExecutionState::HaltedAt(_regs) => return Err(GameFailure::UnexpectedHalt(self.read_buffer.trim().replace("\n\n", " "))),
                ExecutionState::InputIO(io) => {
                    consumed += 1;
                    self.prog.handle_input_completion(io, chars.next().unwrap() as Word)?
                },
                ExecutionState::OutputIO(_io, value) => unreachable!("Output was unexpected: {} with {} chars consumed of {:?}", value, consumed, &input[consumed..]),
            });

            if !chars.peek().is_some() {
                return Ok(());
            }
        }
    }
}

#[derive(Default)]
struct Map {
    room_indices: HashMap<Rc<String>, usize>,
    index_names: Vec<Rc<String>>,
    doors: HashMap<usize, Vec<Door>>,
    checkpoint_doors: HashMap<Rc<String>, CardinalDirection>,
}

#[derive(Debug, Clone, Copy)]
enum Door {
    Visited(usize),
    Unvisited,
    Missing,
}

impl Door {
    fn upgrade(&mut self, room: usize) -> bool {
        match self {
            Door::Unvisited => { *self = Door::Visited(room); true },
            Door::Visited(x) if *x == room => { false },
            Door::Visited(y) => panic!("Cannot change known door {} to {}", y, room),
            Door::Missing => panic!("Cannot learn missing door as {}", room),
        }
    }

    fn accessible(&mut self) {
        match self {
            Door::Missing => *self = Door::Unvisited,
            _ => {}
        }
    }
}

impl Map {

    fn mark_visited(&mut self, room: &Rc<String>, dir: CardinalDirection) {
        //todo!()
    }

    fn is_visited(&mut self, room: &Rc<String>, dir: CardinalDirection) -> bool {
        let index = self.room_indices.get(room)
            .expect("visited room should be known?");

        match self.doors[index][dir.as_index()] {
            Door::Visited(_) => true,
            Door::Unvisited => false,
            Door::Missing => panic!("should not be able to query visitability of {:?} to {}", room.as_str(), dir),
        }
    }

    fn add_checkpoint(&mut self, room: &Rc<String>, dir: CardinalDirection) {
        self.checkpoint_doors.insert(Rc::clone(room), dir);
    }

    fn learn(&mut self, came_from: Option<(&Rc<String>, CardinalDirection)>, room_name: &Rc<String>, doors: &[CardinalDirection]) -> bool {
        let index = self.room_indices.get(room_name).copied();

        let (new_room, index) = match index {
            Some(x) => (false, x),
            None => {
                let index = self.room_indices.len();
                self.room_indices.insert(Rc::clone(room_name), index);
                self.index_names.push(Rc::clone(room_name));
                (true, index)
            }
        };

        let current_doors = &mut self.doors.entry(index).or_insert_with(|| vec![Door::Missing; 4]);

        for dir in doors {
            let slot = &mut current_doors[dir.as_index()];
            slot.accessible();
        }

        if let Some((prev, came_from)) = came_from {
            let prev_index = self.room_indices[prev];
            {
                let slot = &mut current_doors[came_from.reverse().as_index()];
                if slot.upgrade(prev_index) {
                    //println!("  learned from {:?} {} gets us to {:?}", prev.as_str(), came_from, room_name.as_str());
                }
            }
            {
                let doors = self.doors.get_mut(&prev_index).unwrap();
                let slot = &mut doors[came_from.as_index()];
                if slot.upgrade(index) {
                    //println!("  learned from {:?} {} gets us to {:?}", room_name.as_str(), came_from.reverse(), prev.as_str());
                }
            }
        }

        new_room
    }

    fn path_to(&self, from: &Rc<String>, to: &Rc<String>) -> Vec<CardinalDirection> {
        // from == to when exploring a room for the first time
        if from == to {
            vec![]
        } else {
            use std::collections::BinaryHeap;
            use std::collections::hash_map::Entry;
            use std::cmp;
            // monkeyd from day15 again
            let mut ret = Vec::new();
            let mut dist: HashMap<usize, usize> = HashMap::new();
            let mut prev: HashMap<usize, (usize, CardinalDirection)> = HashMap::new();

            #[derive(PartialEq, Debug, Clone, Copy, Eq, Ord)]
            struct Work {
                steps_here: usize,
                room_index: usize,
                came_from: Option<CardinalDirection>,
            }

            impl PartialOrd for Work {
                fn partial_cmp(&self, rhs: &Work) -> Option<cmp::Ordering> {
                    Some(
                        self.steps_here.cmp(&rhs.steps_here).reverse()
                            .then_with(|| self.room_index.cmp(&rhs.room_index))
                            .then_with(|| self.came_from.map(CardinalDirection::as_index).cmp(&rhs.came_from.map(CardinalDirection::as_index)))
                    )
                }
            }

            {
                let mut work: BinaryHeap<Work> = BinaryHeap::new();
                //println!("path_to: {} -> {}", from, to);

                let from = self.to_index(from);
                let to = self.to_index(to);

                work.push(Work { steps_here: 0, room_index: from, came_from: None });

                while let Some(Work { steps_here, room_index, came_from: _came_from }) = work.pop() {

                    //println!("  path_to:  {:>3}, {:?} ({}), {:?}", steps_here, self.index_names[room_index].as_str(), room_index, came_from);

                    match dist.entry(room_index) {
                        Entry::Vacant(vcnt) => {
                            vcnt.insert(steps_here);
                        },
                        Entry::Occupied(mut o) => {
                            if *o.get() >= steps_here {
                                *o.get_mut() = steps_here;
                            } else {
                                //println!("already visited {:?} with lower dist {} than {} from {:?}", room_index, o.get(), steps_here, prev[&room_index]);
                                continue;
                            }
                        }
                    }

                    if room_index == to {
                        let mut backwards = room_index;

                        while backwards != from {
                            let previous = prev.remove(&backwards).unwrap();
                            ret.push(previous.1);
                            backwards = previous.0;
                        }
                        ret.reverse();

                        // could put the door we need to go to into work, but that requires another type
                        // which does not compare on other elements...
                        return ret;
                    }

                    let doors = self.doors[&room_index].iter().enumerate()
                        .filter_map(|(i, door)| match door {
                            Door::Visited(x) => Some((i, x)),
                            _ => None,
                        })
                        .map(|(i, x)| (CardinalDirection::try_from(i).unwrap(), x));

                    for (dir, &next_room) in doors {
                        //println!("    path_to: {} => {:?} ({})", dir, self.index_names[next_room].as_str(), next_room);
                        let steps_here = steps_here + 1;

                        if steps_here < *dist.get(&next_room).unwrap_or(&usize::max_value()) {
                            dist.insert(next_room, steps_here);
                            prev.insert(next_room, (room_index, dir));
                            work.push(Work { steps_here, room_index: next_room, came_from: Some(dir) });
                        }
                    }
                }
            }

            unreachable!("did not find path from {}...{}", from, to);
        }
    }

    fn to_index(&self, room: &Rc<String>) -> usize {
        self.room_indices.get(room).copied().unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MaybeAscii {
    Ascii(u8),
    Other(Word),
}

impl From<Word> for MaybeAscii {
    fn from(w: Word) -> MaybeAscii {
        match w {
            x @ 0..=127 => MaybeAscii::Ascii(x as u8),
            y => MaybeAscii::Other(y),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
enum CardinalDirection {
    North,
    East,
    South,
    West,
}

impl CardinalDirection {
    fn as_direction(self) -> Direction {
        self.into()
    }

    fn as_index(self) -> usize {
        use CardinalDirection::*;

        match self {
            North => 0,
            East => 1,
            South => 2,
            West => 3,
        }
    }

    fn reverse(&self) -> CardinalDirection {
        use CardinalDirection::*;
        match *self {
            North => South,
            West => East,
            South => North,
            East => West,
        }
    }
}

impl Into<Position> for CardinalDirection {
    fn into(self) -> Position {
        let dir: Direction = self.into();
        dir.into()
    }
}

impl TryFrom<usize> for CardinalDirection {
    type Error = usize;
    fn try_from(i: usize) -> Result<Self, Self::Error> {
        use CardinalDirection::*;
        Ok(match i {
            0 => North,
            1 => East,
            2 => South,
            3 => West,
            x => return Err(x),
        })
    }
}

impl Into<Direction> for CardinalDirection {
    fn into(self) -> Direction {
        use CardinalDirection::*;
        use Direction::*;
        match self {
            North => Up,
            East => Right,
            South => Down,
            West => Left,
        }
    }
}

impl<'a> TryFrom<&'a str> for CardinalDirection {
    type Error = &'a str;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Ok(match s {
            "north" => CardinalDirection::North,
            "east"  => CardinalDirection::East,
            "south" => CardinalDirection::South,
            "west"  => CardinalDirection::West,
            _ => return Err(s),
        })
    }
}

impl fmt::Display for CardinalDirection {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            CardinalDirection::North => "north",
            CardinalDirection::East => "east",
            CardinalDirection::South => "south",
            CardinalDirection::West => "west",
        };

        write!(fmt, "{}", s)
    }
}

use std::collections::VecDeque;

struct StdIO {
    stdin: std::io::Stdin,
    buffer: VecDeque<u8>,
    line: String,
}

impl StdIO {
    #[allow(dead_code)]
    fn new() -> Self {
        StdIO {
            stdin: std::io::stdin(),
            buffer: VecDeque::new(),
            line: String::new(),
        }
    }
}

impl intcode::IO for StdIO {
    fn input(&mut self) -> Result<Word, intcode::ProgramError> {
        use std::io::BufRead;

        while self.buffer.is_empty() {
            self.line.clear();
            match self.stdin.lock().read_line(&mut self.line) {
                Ok(0) => panic!("stdin eof"),
                Ok(_) => {
                    let trimmed = self.line.trim().as_bytes();

                    if trimmed.is_empty() {
                        continue;
                    }
                    self.buffer.extend(trimmed);
                    self.buffer.push_back(b'\n');
                }
                Err(e) => panic!("stdin read failed: {}", e),
            }
        }

        Ok(self.buffer.pop_front().unwrap() as Word)
    }

    fn output(&mut self, value: Word) -> Result<(), intcode::ProgramError> {
        print!("{}", value as u8 as char);
        Ok(())
    }
}
