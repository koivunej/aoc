use intcode::{util::GameDisplay, Word};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::io::Read;
use ndarray::Array2;
use itertools::Either;

fn main() {
    let mut map = String::new();
    let stdin = std::io::stdin();
    let mut locked = stdin.lock();
    locked.read_to_string(&mut map).unwrap();

    let map = Map::from_str(&map).unwrap();

    println!("stage1: {}", steps_to_collect_all_keys(&map));
}

fn steps_to_collect_all_keys(m: &Map) -> usize {

    let vertices = m.gd.cells().iter()
        .enumerate()
        .filter_map(|(i, t)| match t {
            Tile::Wall => None,
            t => Some((m.gd.to_coordinates(i), t.clone())),
        })
        .collect::<Vec<_>>();

    // this index is used because the vertices dont include the walls so it complicates.
    // wasted time because the floyd warshall is something to the n^3 and the input seemed
    // to have lot of walls
    let vertices_index = vertices.iter()
        .enumerate()
        .map(|(i, (coords, _))| (coords, i))
        .collect::<HashMap<_, _>>();

    let edges = vertices.iter()
        .flat_map(|(c, t)| m.frontier_where(*c, t).map(move |(target, _tile)| (*c, target)))
        .map(|(from, to)| (vertices_index[&from], vertices_index[&to]));

    let mut dist = ndarray::Array2::<Option<u32>>::from_elem((vertices.len(), vertices.len()), None);
    let mut next = ndarray::Array2::<Option<usize>>::from_elem((vertices.len(), vertices.len()), None);

    let mut edge_count = 0;
    for (from, to) in edges {
        dist[(from, to)] = Some(1);
        dist[(to, from)] = Some(1);
        next[(from, to)] = Some(to);
        next[(to, from)] = Some(from);

        edge_count += 2;
    }

    for (i, _) in vertices.iter().enumerate() {
        next[(i, i)] = Some(i);
    }

    println!("calculating paths for {} vertices and {} edges", vertices.len(), edge_count);

    for k in 0..vertices.len() {
        println!("calculated {}/{}", k * vertices.len().pow(2), vertices.len().pow(3));
        for i in 0..vertices.len() {
            for j in 0..vertices.len() {

                if i == j || i == k || k == j {
                    continue;
                }

                let rhs = dist[(i, k)]
                    .and_then(|ik| dist[(k, j)]
                    .map(|kj| ik + kj));

                if rhs.is_none() {
                    continue;
                }

                let lhs = &mut dist[(i, j)];

                // not really sure why this floyd warshall imitation works but it seems to work

                if rhs.is_some() && (lhs.is_none() || lhs.unwrap() > rhs.unwrap()) {
                    let rhs = rhs.unwrap();
                    if lhs.unwrap_or(u32::max_value()) > rhs {
                        *lhs = Some(rhs);
                        next[(i, j)] = next[(i, k)];
                    }
                }
            }
        }
    }

    println!("got all paths");

    let all_paths = AllPaths {
        dist: &dist,
        next: &next,
        vertices: vertices.as_slice(),
        index: &vertices_index
    };

    let all_keys = m.poi.iter().filter(|(t, _)| t.is_key()).collect::<Vec<_>>();
    let all_keys_set = m.poi.iter().filter(|(t, _)| t.is_key()).fold(KeySet::default(), |ks, (t, _)| ks + t);
    println!("all_keys: {:?}", all_keys);

    // not sure how this could be formulated as a dynamic programming task
    // it is not always obvious which of the keys is a good last key to get.

    let mut frontier = BinaryHeap::new();

    frontier.push(cmp::Reverse(InterestingPath {
        steps: 0,
        keys: cmp::Reverse(KeySet::default()),
        pos: m.initial_position,
    }));

    let mut visited = HashSet::new();

    let mut solutions = BinaryHeap::new();

    while let Some(cmp::Reverse(ip)) = frontier.pop() {

        if let Some(cmp::Reverse(min)) = solutions.peek() {
            if *min < ip.steps {
                println!("pruning {:?} {:?} {} steps", ip.pos, ip.keys.0, ip.steps);
                continue;
            }
        }

        if !visited.insert(ip.clone()) {
            println!("pruning {:?} {:?} {} steps", ip.pos, ip.keys.0, ip.steps);
            continue;
        }

        println!("exploring with steps={}, keys: {:?}, pos: {:?}", ip.steps, ip.keys, ip.pos);

        let next_possible = all_keys.iter()
            .filter_map(|(tile, coord)| if !ip.keys.0.contains(tile) { Some(coord) } else { None })
            .map(|coord| all_paths.find_path(coord, &ip.pos, &ip.keys.0))
            .filter_map(|p| p.map(|(steps, pos, keys)| (steps - 1, pos, keys)));

        let mut any = false;
        let mut made_progress = false;

        for (more_steps, pos, keys) in next_possible {

            let steps = ip.steps + more_steps;

            if pos == ip.pos {
                println!("next_possible does not change pos: {:?} but steps = {} and keys = {:?}", pos, steps, keys);
                continue;
            }

            if keys == ip.keys.0 {
                println!("next_possible does not change keys: {:?} but steps = {} and pos = {:?}", keys, steps, pos);
                continue;
            }

            made_progress = true;

            match solutions.peek() {
                Some(cmp::Reverse(min)) if *min < steps => {
                    println!(" ---> pruning {:?} to {:?} for {:?} with {} steps", ip.pos, pos, keys, steps);
                    continue;
                },
                _ => {},
            }

            let ip = InterestingPath {
                steps,
                keys: cmp::Reverse(keys),
                pos
            };

            if visited.contains(&ip) {
                println!(" ---> pruning {:?} {:?} {} steps", ip.pos, ip.keys.0, ip.steps);
                continue;
            }

            println!(" ---> steps={}, keys: {:?}, pos: {:?}", steps, keys, pos);
            frontier.push(cmp::Reverse(ip));

            any = true;
        }

        if any && !made_progress {
            println!("did not make any progress from {:?}", ip);
            let possible = all_keys.iter()
                .filter_map(|(tile, coord)| if !ip.keys.0.contains(tile) { Some((tile, coord)) } else { None })
                .collect::<Vec<_>>();

            println!("possible next:");
            for x in possible {
                println!("  {:?}", x);
            }

            println!("next_possible:");
            for x in all_keys.iter()
                    .filter_map(|(tile, coord)| if !ip.keys.0.contains(tile) { Some(coord) } else { None })
                    .map(|coord| all_paths.find_path(coord, &ip.pos, &ip.keys.0))
                    .filter_map(|p| p.map(|(steps, pos, keys)| (steps - 1, pos, keys)))
            {
                println!("  {:?}", x);
            }

            panic!("could not make progress");
        }

        if !any {
            if !all_keys_set.subset_of(&ip.keys.0) {
                continue;
            }
            solutions.push(cmp::Reverse(ip.steps));
            println!("found solution: {}", ip.steps);
        }
    }

    match solutions.pop() {
        Some(cmp::Reverse(min)) => return min,
        None => unimplemented!("failed to find a single path through all keys"),
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct InterestingPath {
    // ord: greatest keys (most keys) ... probably
    keys: cmp::Reverse<KeySet>,
    // ord: the shortest steps
    steps: usize,
    // ord: not really caring about the position
    pos: (i64, i64),
}

struct AllPaths<'a, 'b> {
    dist: &'a Array2<Option<u32>>,
    next: &'a Array2<Option<usize>>,
    vertices: &'a [((Word, Word), Tile)],
    index: &'a HashMap<&'b (Word, Word), usize>,
}

impl<'a, 'b> AllPaths<'a, 'b> {
    // find path if it's possible with these keys
    fn find_path(&self, a: &(Word, Word), b: &(Word, Word), keys: &KeySet) -> Option<(usize, (Word, Word), KeySet)> {
        let u = self.index[a];
        let v = self.index[b];

        let mut path_keys = keys.clone();
        path_keys += &self.vertices[u].1;
        //let mut path = vec![self.vertices[u].0];
        let mut steps = 1;
        let pos = self.vertices[u].0;

        let mut u = match self.next[(u, v)] {
            Some(u) => u,
            None => {
                return None;
            }
        };

        path_keys += &self.vertices[u].1;
        steps += 1;

        loop {
            u = match self.next[(u, v)] {
                Some(u) => u,
                None => {
                    panic!("no subpath from {:?} to {:?}", self.vertices[u], self.vertices[v]);
                }
            };
            path_keys += &self.vertices[u].1;
            if !keys.can_open(&path_keys) {
                return None;
            }

            steps += 1;
            //path.push(self.vertices[u].0);
            if u == v {
                break;
            }
        }

        //Some((path, path_keys))
        Some((steps, pos, path_keys.only_keys()))
    }
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq)]
struct KeySet(u64);

use std::cmp;

impl PartialOrd for KeySet {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        let lhs = self.only_keys().0;
        let rhs = rhs.only_keys().0;

        lhs.partial_cmp(&rhs)
    }
}

impl Ord for KeySet {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

impl std::ops::Add<&Tile> for KeySet {
    type Output = KeySet;

    fn add(self, key: &Tile) -> KeySet {
        let bit = Self::to_bit(key).unwrap_or_else(|nk| panic!("add given non-key: {}", nk));

        if self.0 & bit == 0 {
            KeySet(self.0 | bit)
        } else {
            KeySet(self.0)
        }
    }
}

impl std::ops::AddAssign<&Tile> for KeySet {
    fn add_assign(&mut self, rhs: &Tile) {
        let bit = match Self::to_bit(rhs) {
            Ok(bit) => bit,
            Err(_) => return,
        };

        if self.0 & bit == 0 {
            self.0 |= bit;
        }
    }
}

impl std::ops::Sub<&KeySet> for KeySet {
    type Output = KeySet;

    fn sub(self, rhs: &KeySet) -> KeySet {
        KeySet(self.0 & !rhs.0)
    }
}

impl KeySet {
    fn to_bit(key: &Tile) -> Result<u64, &Tile> {
        match key {
            &Tile::Key(ch) => Ok(1 << (ch as u8 - b'a')),
            &Tile::Door(ch) => Ok(1 << 32 + ch as u8 - b'A'),
            x => Err(x),
        }
    }

    fn can_open(&self, doors: &KeySet) -> bool {
        let only_keys = self.only_keys().0;
        let only_doors = doors.only_doors().0;

        let shifted = only_doors >> 32;
        // println!("init({:?}, {:?}): only({:08x} and {:08x})", self, doors, only_keys, only_doors >> 26);
        only_keys & shifted == shifted
    }

    fn contains(&self, key: &Tile) -> bool {
        match Self::to_bit(key) {
            Ok(bit) => self.0 & bit == bit,
            Err(_) => false,
        }
    }

    fn subset_of(&self, rhs: &KeySet) -> bool {
        (self.0 & rhs.0) == self.0
    }

    fn key_count(&self) -> usize {
        let keys = 0xffff_ffff;
        (self.0 & keys).count_ones() as usize
    }

    fn only_keys(&self) -> Self {
        let keys = 0xffff_ffff;
        KeySet(self.0 & keys)
    }

    fn only_doors(&self) -> Self {
        let doors = !0x0000_0000_ffff_ffff;
        KeySet(self.0 & doors)
    }
}

#[test]
fn keyset_opening() {
    let mut ks = KeySet::default();

    ks += &Tile::Key('a');

    let ks = ks;

    let doors = KeySet::default() + &Tile::Door('A');

    assert!(!KeySet::default().can_open(&doors));
    assert!(ks.can_open(&doors));

    assert!(ks.subset_of(&(ks + &Tile::Door('A'))));
    assert!(ks.subset_of(&(ks + &Tile::Key('b'))));
    assert!(!ks.subset_of(&(KeySet::default() + &Tile::Key('b'))));
}

#[test]
fn keyset_contains() {
    let mut ks = KeySet::default();

    ks += &Tile::Key('a');

    assert!(ks.contains(&Tile::Key('a')));
    assert!(!KeySet::default().contains(&Tile::Key('a')));
}

#[test]
fn keyset_for_all_keys_and_doors() {
    let mut ks = KeySet::default();
    let mut all = KeySet::default();

    for ascii in b'a'..=b'z' {
        let ascii = ascii as char;
        let prev = ks.clone();

        let key = Tile::Key(ascii);
        let door = KeySet::default() + &Tile::Door(ascii.to_ascii_uppercase());

        assert!(!ks.can_open(&door));

        ks += &Tile::Key(ascii);

        assert_ne!(prev, ks);
        assert!(ks.can_open(&door), "{:?} should open {:?}", ks, door);

        all += &Tile::Key(ascii);
        all += &Tile::Door(ascii.to_ascii_uppercase());
    }

    assert!(all.only_keys().subset_of(&ks), "{:?} should be subset of {:?}", all.only_keys(), ks);

}

impl fmt::Debug for KeySet {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 0 {
            return write!(fmt, "0");
        }

        write!(fmt, "\"")?;

        for ch in b'a'..=b'z' {
            let bit = 1 << (ch - b'a');

            if self.0 & bit != 0 {
                write!(fmt, "{}", ch as char)?;
            }
        }
        for ch in b'A'..=b'Z' {
            let bit = 1 << (32 + ch - b'A');
            if self.0 & bit != 0 {
                write!(fmt, "{}", ch as char)?;
            }
        }

        write!(fmt, "\"")
    }
}

/// We can explain our travers with this?
struct Waypoint((Word, Word), Tile);

struct Map {
    gd: GameDisplay<Tile>,
    poi: HashMap<Tile, (Word, Word)>,
    initial_position: (Word, Word),
}

impl Map {
    fn frontier<'a>(&'a self, pos: (Word, Word)) -> impl Iterator<Item = ((Word, Word), Tile)> + 'a {
        match self.gd.get(&pos) {
            Some(&Tile::Wall) | None => Either::Left(std::iter::empty()),
            Some(t) => Either::Right(self.frontier_where(pos, t))
        }
    }

    fn frontier_where<'a>(&'a self, pos: (Word, Word), t: &'a Tile) -> impl Iterator<Item = ((Word, Word), Tile)> + 'a {
        let deltas = &[
            ( 0, -1), // up
            ( 1,  0), // right
            ( 0,  1), // down
            (-1,  0) // left
        ];

        match t {
            &Tile::Wall => return Either::Left(std::iter::empty()),
            _ => {}
        }

        Either::Right(deltas.iter()
            .map(move |d| (pos.0 + d.0, pos.1 + d.1))
            .filter_map(move |p| {
                self.gd.get(&p).and_then(|t| match t {
                    &Tile::Wall => None,
                    _ => Some((p, t.clone()))
                })
            }))
    }

}

impl fmt::Display for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.gd.fmt(fmt)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum Tile {
    Empty,
    Wall,
    Portal,
    Key(char),
    Door(char),
}

impl Tile {
    fn is_key(&self) -> bool {
        match *self {
            Tile::Key(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let ch = match *self {
            Tile::Empty => '.',
            Tile::Wall => '#',
            Tile::Portal => '@',
            Tile::Key(ch) => ch,
            Tile::Door(ch) => ch,
        };

        write!(fmt, "{}", ch)
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Empty
    }
}

#[derive(Debug)]
struct InvalidTile(char);

impl TryFrom<char> for Tile {
    type Error = InvalidTile;

    fn try_from(ch: char) -> Result<Self, Self::Error> {
        Ok(match ch {
            '.' => Tile::Empty,
            '#' => Tile::Wall,
            '@' => Tile::Portal,
            ch @ 'a'..='z' => Tile::Key(ch),
            ch @ 'A'..='Z' => Tile::Door(ch),
            x => return Err(InvalidTile(x)),
        })
    }
}

impl FromStr for Map {
    type Err = InvalidTile;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use std::collections::hash_map::Entry;
        let (mut gd, mut poi) = s
            .trim()
            .chars()
            .scan((0i64, 0i64), |mut acc, ch| match ch {
                '\n' => {
                    acc.1 += 1;
                    acc.0 = 0;
                    Some((*acc, None))
                }
                _ => {
                    let old = *acc;
                    acc.0 += 1;
                    Some((old, Some(ch)))
                }
            })
            .filter_map(|(pos, ch)| match ch {
                Some(ch) => Some((pos, ch)),
                None => None,
            })
            .map(|(pos, ch)| (pos, Tile::try_from(ch)))
            .fold(Ok((GameDisplay::default(), HashMap::new())), |gd, (pos, tile)| {
                let (mut gd, mut poi) = gd.unwrap();
                let tile = tile?;

                match &tile {
                    &Tile::Portal
                    | &Tile::Door(_)
                    | &Tile::Key(_) => {
                        let old = poi.insert(tile.clone(), pos.clone());
                        if let Some(x) = old {
                            panic!("duplicate coordinates for tile {:?}: {:?} and {:?}", tile, pos, x);
                        }
                    },
                    _ => {}
                }

                gd.insert(&pos, tile);
                Ok((gd, poi))
            })?;

        let initial_position = poi.remove(&Tile::Portal).expect("No portal '@' found on the map");
        gd.insert(&initial_position, Tile::Empty);

        Ok(Map {
            gd,
            poi,
            initial_position,
        })
    }
}

#[test]
fn parse_first_map() {
    let s = "\
#########
#b.A.@.a#
#########";

    let m = Map::from_str(s).unwrap();

    assert_eq!(s.trim().replace("@", "."), format!("{}", m).trim());
    assert_eq!((5, 1), m.initial_position);
    assert_eq!(Some(&(7, 1)), m.poi.get(&Tile::Key('a')));
    assert_eq!(Some(&(3, 1)), m.poi.get(&Tile::Door('A')));
    assert_eq!(Some(&(1, 1)), m.poi.get(&Tile::Key('b')));

    assert_eq!(&Tile::Wall, m.gd.get(&(0, 0)).unwrap());
}

#[test]
fn full_first_example() {
    let s = "\
#########
#b.A.@.a#
#########";

    let mut m = Map::from_str(s).unwrap();

    assert_eq!(steps_to_collect_all_keys(&mut m), 8);
}

#[test]
fn first_multiple_choice() {
    let s = "\
########################
#f.D.E.e.C.b.A.@.a.B.c.#
######################.#
#d.....................#
########################";

    let mut m = Map::from_str(s).unwrap();

    assert_eq!(steps_to_collect_all_keys(&mut m), 86);
}

#[test]
fn second_multiple_choice() {
    let s = "\
########################
#...............b.C.D.f#
#.######################
#.....@.a.B.c.d.A.e.F.g#
########################";

    let mut m = Map::from_str(s).unwrap();

    assert_eq!(steps_to_collect_all_keys(&mut m), 132);
}
