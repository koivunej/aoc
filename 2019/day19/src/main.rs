use std::convert::TryFrom;
use std::fmt;
use intcode::{parse_stdin_program, Memory, Program, Registers, ExecutionState, Word, util::{GameDisplay, Position}};

fn main() {
    let program = parse_stdin_program();
    let mut drones = DroneDeployer::new(program.as_slice());
    let p1_queries = {
        let mut p1 = PartWrapper { inner: &mut drones, queries: 0 };
        println!("part1: {}", part1(&mut p1, (50, 50)));
        p1.into_statistics()
    };

    let p2_queries = {
        let mut p2 = PartWrapper { inner: &mut drones, queries: 0 };
        println!("part2: {}", part2(&mut p2, 100));
        p2.into_statistics()
    };

    // this was 112 and 11599 upon submission without recheck and diagnostics
    println!("queries: {:?}", (p1_queries, p2_queries));
}

trait Queryable {
    fn query(&mut self, pos: Position) -> Tile;
}

impl<'a, T: Queryable> Queryable for &'a mut T {
    fn query(&mut self, pos: Position) -> Tile {
        (*self).query(pos)
    }
}

struct PartWrapper<Q> {
    inner: Q,
    queries: usize,
}

impl<Q: Queryable> Queryable for PartWrapper<Q> {
    fn query(&mut self, pos: Position) -> Tile {
        let ret = self.inner.query(pos);
        self.queries += 1;
        ret
    }
}

impl<Q> PartWrapper<Q> {
    fn into_statistics(self) -> usize {
        self.queries
    }
}

struct DroneDeployer<'a> {
    original: &'a [Word],
    memory: Option<Memory<'static>>,
}

impl<'a> DroneDeployer<'a> {
    fn new(program: &'a [Word]) -> Self {
        Self {
            original: program,
            memory: Some(Memory::from(program).with_memory_expansion()),
        }
    }
}

impl<'a> Queryable for DroneDeployer<'a> {
    fn query(&mut self, p: Position) -> Tile {
        let mut mem = self.memory.take().expect("Memory should still be in place");
        mem.reset_from(self.original);
        let mut program = Program::from(mem);

        let pos = [p.x(), p.y()];
        let mut pos = pos.into_iter();

        let mut regs = Some(Registers::default());
        let mut output = None;
        loop {
            regs = Some(match program.eval_from_instruction(regs.take().unwrap()).unwrap() {
                ExecutionState::Paused(_regs) => unreachable!("Pausing not implemented yet?"),
                ExecutionState::HaltedAt(_regs) => {
                    self.memory = Some(program.unwrap());

                    if let Some(output) = output.take() {
                        if output == 1 {
                            return Tile::Beam;
                        } else if output == 0 {
                            return Tile::Empty;
                        } else {
                            panic!("Invalid output from program: {}", output);
                        }
                    } else {
                        unreachable!("Halted without output for {:?}", p);
                    }
                },
                ExecutionState::InputIO(io) => {
                    let input = pos.next().unwrap();
                    program.handle_input_completion(io, *input).unwrap()
                },
                ExecutionState::OutputIO(io, value) => {
                    assert!(output.is_none());
                    output = Some(value);
                    program.handle_output_completion(io)
                },
            });
        }
    }
}

fn query<Q: Queryable>(queryable: &mut Q, size: (usize, usize)) -> GameDisplay<Tile> {
    // go down until no pull, then go right
    let mut first_x = None;

    let mut gd = GameDisplay::default();

    let mut work = Some(Position::default());

    while let Some(pos) = work.take() {

        if pos.y() == size.1 as Word {
            break;
        }

        let success = match queryable.query(pos) {
            Tile::Beam => {
                gd.insert(&pos.into(), Tile::Beam);
                true
            }
            _ => {
                false
            },
        };

        if success {
            first_x = first_x.or_else(|| Some(pos.x()));
        }

        let in_horiz_bounds = pos.x() < size.0 as Word;

        work = Some(match (success, in_horiz_bounds, &mut first_x) {
            (true, true, _)
            | (false, true, None) => { pos + (1, 0) },
            (_, _, ref mut x) => { ((x.take().unwrap_or(0), pos.y() + 1).into()) },
        });
    }

    //println!("{}", gd);
    //println!("{}", queries);

    gd
}

fn part1<Q: Queryable>(queryable: &mut Q, size: (usize, usize)) -> usize {
    let gd = query(queryable, size);
    gd.cells()
        .iter()
        .filter(|t| **t == Tile::Beam)
        .count()
}

fn part2<Q: Queryable>(queryable: &mut Q, square: i64) -> i64 {
    fn search<Q: Queryable, P: Into<Position>>(queryable: &mut Q, mut pos: Position, increment: P) -> Option<Position> {

        let increment: Position = increment.into();
        let mut prev = None;

        loop {
            pos = pos + increment;

            match queryable.query(pos) {
                Tile::Beam => {},
                _ => return prev,
            }

            prev = Some(pos);
        }
    }

    let mut pos = Position::from((0, 0));

    loop {
        // in my input there was a single beam at (0,0) but nothing for a while. quite clever for
        // the day19, my solution is ... quite stupid here. this could have been just skipped by
        // setting pos = (10, 10)
        match queryable.query(pos) {
            Tile::Beam => {
                pos = pos + (1, 1);
            },
            _ => {
                loop {
                    if let Some(regained) = search(queryable, pos, (-1, 0)) {
                        //println!("regained {:?}", pos);
                        pos = regained + (1, 1);
                        break;
                    }

                    // this may or may not have been bruteforced and noticed to be 10 * (1, 1).
                    pos = pos + (10, 10);
                }
            }
        }

        if pos.x() > 10 && pos.y() > 10 {
            break;
        }
    }

    let diagnostics = square < 100;

    let mut gd = GameDisplay::default();
    loop {
        // first idea was to track upper and lower part but turns out those are hard to convert
        // back into a square, so I dropped tracking the lower part.
        pos = search(queryable, pos, (1, 0)).unwrap_or(pos);

        if pos.x() < square && pos.y() < square {
            // there is now a slight possibility of a square, not really though
            if diagnostics {
                println!("not trying yet at {:?}", pos);
                gd.insert(&pos.into(), DiagnosticTile::Interesting('n'));
            }
        } else {
            // first calculate the corner, but do not check it yet
            let mut corner = pos + (1 - square, 0);

            // then take a guess at the left bottom, which is just `pos + (1 - square, square - 1)`
            // but the corner is useful for debugging purposes for basing the guess
            let left_bottom_guess = corner + (0, square - 1);

            // i thought i needed to search left for a better left bottom but this should always be
            // the guess... my longest standing issue was in fact when using this search which I
            // had forgotten returns None if there is no better position, instead of returning the
            // search start point, the second argument.
            let left_bottom = search(queryable, left_bottom_guess, (-1, 0)).unwrap_or(left_bottom_guess);
            assert_eq!(left_bottom, left_bottom_guess);

            if diagnostics {
                if pos != (34, 20).into() {
                    gd.insert(&pos.into(), DiagnosticTile::Interesting('a'));
                    gd.insert(&corner.into(), DiagnosticTile::Interesting('c'));
                    gd.insert(&left_bottom_guess.into(), DiagnosticTile::Interesting('l'));
                } else {
                    // upper case the buggy row
                    gd.insert(&pos.into(), DiagnosticTile::Interesting('A'));
                    gd.insert(&corner.into(), DiagnosticTile::Interesting('C'));
                    gd.insert(&left_bottom_guess.into(), DiagnosticTile::Interesting('L'));
                }
            }

            if let Tile::Beam = queryable.query(left_bottom) {
                if diagnostics {
                    gd.insert(&left_bottom.into(), DiagnosticTile::Interesting('B'));
                }
                let first_corner = corner;
                corner = left_bottom + (0, -square + 1);

                println!("started with from {:?}, corner = {:?}, went to {:?}, searched left until {:?}, corner = {:?}",
                    pos,
                    first_corner,
                    left_bottom_guess,
                    left_bottom,
                    corner);

                let recheck = square < 100;

                let all_good = if recheck {

                    let mut all_good = true;

                    // i actually used this just for printing for the part2 test case; the recheck
                    // is unusable in with the real input as it starts from zero, and goes over.
                    //
                    // "recheck" as in, make sure all of the tiles in the square are beam.
                    for x in 0..corner.x() + 20 {
                        for y in 0..corner.y() + 20 {
                            let p = (x, y).into();
                            let tile = queryable.query(p);
                            let dtile = DiagnosticTile::from_corner(tile, corner, square, p);
                            if diagnostics {
                                match gd.get(&(x, y)) {
                                    Some(DiagnosticTile::Normal(Tile::Empty)) | None => {
                                        gd.insert(&(x, y), dtile);
                                    }
                                    _ => {},
                                }
                            }

                            if dtile.is_square() {
                                all_good &= tile == Tile::Beam;
                            }
                        }
                    }

                    all_good
                } else {
                    true
                };

                if diagnostics {
                    println!("{}", gd);
                }

                assert_eq!(queryable.query(pos), Tile::Beam);
                assert_ne!(queryable.query(pos + (1, 0)), Tile::Beam);
                assert_eq!(queryable.query(corner), Tile::Beam);

                if !all_good {
                    panic!("all tiles were not Beam");
                }

                let ret = corner.x() * 10_000 + corner.y();
                println!("found from {:?}, relative from corner {:?}", pos, pos - corner);
                // a "couple" of failed attempts, realized to start logging the submitted a bit
                // late. need to do this more.
                assert_ne!(ret, 18552014);
                assert_ne!(ret, 18331990);
                return ret;
            }
        }

        // so to follow the right edge ... i guess the first one is enough, and wouldn't cause
        // jumpy output if you log all the coordinates and plot them. perhaps I was trying to be
        // clever here.
        let next = &[(0, 1), (1, 1)];
        let step = next.iter()
            .map(|off| pos + off)
            .filter(|p| queryable.query(*p) == Tile::Beam)
            .next();

        pos = match step {
            Some(p) => p,
            None => {
                let offsets = &[
                    (-2,-2), (-1,-2), ( 0,-2), ( 1,-2), ( 2,-2),
                    (-2,-1), (-1,-1), ( 0,-1), ( 1,-1), ( 2,-1),
                    (-2, 0), (-1, 0), ( 0, 0), ( 1, 0), ( 2, 0),
                    (-2, 1), (-1, 1), ( 0, 1), ( 1, 1), ( 2, 1),
                    (-2, 2), (-1, 2), ( 0, 2), ( 1, 2), ( 2, 2),
                ];

                let mut gd = GameDisplay::default();

                for offset in offsets {
                    gd.insert(offset, queryable.query(pos + offset));
                }

                println!("{}", gd);
                panic!("lost")
            }
        };

        // one last extra check even though this has already been checked with the filter above.
        assert_eq!(queryable.query(pos), Tile::Beam);
    }
}

#[allow(unused_assignments, unused_variables, dead_code)]
fn part2_bad<Q: Queryable>(queryable: &mut Q, square: usize) -> usize {
    // IDEA: go diagonally from last ... diagonal point
    // if nothing, query horizontal axis towards zero, stop at zero, continue next hopefully diagonal
    // if something, query horizontal axis towards infinity, get width to next empty
    //          then query vertical axis towards infinity, get height to next empty
    //
    // if the square is not enough, could we skip until pos + (width, height)?
    // if width and height are precisely enough probably check the square?
    // if width and height are more, keep track of too large diagnola, keep track of last good diagonal and go half into that?

    fn search<Q: Queryable, P: Into<Position>>(queryable: &mut Q, mut pos: Position, increment: P) -> Option<Position> {

        let increment: Position = increment.into();
        let mut prev = None;

        loop {
            pos = pos + increment;

            match queryable.query(pos) {
                Tile::Beam => {},
                _ => return prev,
            }

            prev = Some(pos);
        }
    }

    let mut pos = Position::default();

    let mut last_good_diagonal = None;
    let mut too_large_diagonal = None; // unsure if needed

    loop {
        print!("{:?}: ", pos);
        match queryable.query(pos) {
            Tile::Beam => {
                last_good_diagonal = Some(pos);
                /* search right */
                let right = search(queryable, pos, (1, 0));
                let down = search(queryable, pos, (0, 1));

                match (right, down) {
                    (Some(right), Some(down)) => {
                        let w = right.x() - pos.x();
                        let h = down.y() - pos.y();

                        if w == square as i64 && h == square as i64 {
                            return (10_000 * pos.x() + pos.y()) as usize;
                        } else if w > square as i64 && h > square as i64 {
                            too_large_diagonal = Some(pos);
                            println!("found too large diagonal: {}x{}", w, h);
                            break;
                        } else {
                            println!("skipping on square {}x{}", w, h);
                            pos = pos + (1, 1);
                            continue;
                        }
                    },
                    (w, h) => {
                        // narrow beam
                        println!("narrow {:?}x{:?}", w, h);
                        pos = w.or(h).unwrap_or(pos) + (1, 1);
                        continue;
                    }
                }
                /* search down */
            },
            _ => {
                /* search left to zero */
                if let Some(right_edge) = search(queryable, pos, (-1, 0)) {
                    last_good_diagonal = Some(right_edge);
                    // we are now on the right edge... search down?
                    if let Some(left_edge) = search(queryable, right_edge, (0, 1)) {
                        last_good_diagonal = Some(left_edge);
                        pos = (right_edge.x() + right_edge.x() - pos.x(), left_edge.y() + left_edge.y() - pos.y()).into();
                        println!("found both edges {:?} and {:?}, skipping", right_edge, left_edge);
                        continue;
                    } else {
                        println!("found right edge {:?}", right_edge);
                    }
                } else {
                    println!("did not find right edge? left_edge = {:?}", search(queryable, pos, (0, 1)));
                }
            },
        }
        pos = pos + (1, 1);
    }

    // not sure what the idea was here, perhaps to backtrace to a better corner?
    let last_good_diagonal = last_good_diagonal.unwrap();
    let too_large_diagonal = too_large_diagonal.unwrap();

    loop {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Beam,
    Empty,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiagnosticTile {
    Square(Tile),
    Normal(Tile),
    Interesting(char),
}

impl DiagnosticTile {
    fn is_square(&self) -> bool {
        match *self {
            DiagnosticTile::Square(_) => true,
            _ => false,
        }
    }

    fn from_corner(tile: Tile, corner: Position, sq: i64, pos: Position) -> DiagnosticTile {
        let d = pos - corner;
        let inside = 0 <= d.x() && d.x() < sq
            && 0 <= d.y() && d.y() < sq;

        if inside {
            DiagnosticTile::Square(tile)
        } else {
            DiagnosticTile::Normal(tile)
        }
    }
}

impl Default for DiagnosticTile {
    fn default() -> DiagnosticTile {
        DiagnosticTile::Normal(Tile::Empty)
    }
}

impl fmt::Display for DiagnosticTile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use DiagnosticTile::*;

        let ch = match *self {
            Square(Tile::Beam) => 'O',
            Square(_) => 'X',
            Normal(t) => { return write!(fmt, "{}", t); }
            Interesting(ch) => { return write!(fmt, "{}", ch); }
        };

        write!(fmt, "{}", ch)
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Unknown
    }
}

impl TryFrom<char> for Tile {
    type Error = char;

    fn try_from(ch: char) -> Result<Self, Self::Error> {
        Ok(match ch {
            '#' | 'O' => Tile::Beam,
            '.' => Tile::Empty,
            ch => return Err(ch),
        })
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let ch = match *self {
            Tile::Beam => '#',
            Tile::Empty => '.',
            Tile::Unknown => '?',
        };

        write!(fmt, "{}", ch)
    }
}

#[cfg(test)]
struct QueryableMap(usize, GameDisplay<Tile>);

#[cfg(test)]
impl From<GameDisplay<Tile>> for QueryableMap {
    fn from(gd: GameDisplay<Tile>) -> Self {
        QueryableMap(0, gd)
    }
}

#[cfg(test)]
impl Queryable for QueryableMap {
    fn query(&mut self, pos: Position) -> Tile {
        self.0 += 1;
        self.1.get(&pos.into()).cloned().map(|t| match t { Tile::Unknown => Tile::Empty, x => x }).unwrap_or(Tile::Empty)
    }
}

#[test]
fn test_example() {
    let input = "\
#.........
.#........
..##......
...###....
....###...
.....####.
......####
......####
.......###
........##";

    let truth = {
        let mut gd: GameDisplay<Tile> = GameDisplay::default();
        gd.parse_grid_lines(&input, (0, 0)).unwrap();
        assert_eq!(format!("{}", gd).trim(), input);
        gd
    };

    let mut q = QueryableMap(0, truth);

    assert_eq!(27, part1(&mut q, (10, 10)));
    assert_eq!(45, q.0);
}

#[test]
fn test_second_example() {
    let input = "\
#.......................................
.#......................................
..##....................................
...###..................................
....###.................................
.....####...............................
......#####.............................
......######............................
.......#######..........................
........########........................
.........#########......................
..........#########.....................
...........##########...................
...........############.................
............############................
.............#############..............
..............##############............
...............###############..........
................###############.........
................#################.......
.................########OOOOOOOOOO.....
..................#######OOOOOOOOOO#....
...................######OOOOOOOOOO###..
....................#####OOOOOOOOOO#####
.....................####OOOOOOOOOO#####
.....................####OOOOOOOOOO#####
......................###OOOOOOOOOO#####
.......................##OOOOOOOOOO#####
........................#OOOOOOOOOO#####
.........................OOOOOOOOOO#####
..........................##############
..........................##############
...........................#############
............................############
.............................###########";

    let truth = {
        let mut gd: GameDisplay<Tile> = GameDisplay::default();
        gd.parse_grid_lines(&input, (0, 0)).unwrap();
        gd
    };

    let mut q = QueryableMap(0, truth);

    assert_eq!(part2(&mut q, 10), 250020);
}
