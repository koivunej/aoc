use std::str::FromStr;
use std::collections::HashMap;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    let locked = stdin.lock();

    let lines = locked.lines().map(Result::unwrap).collect::<Vec<_>>();

    println!("{:?}", stage1(&lines));

    println!("{:?}", stage2(&lines));
}

fn stage1<T: AsRef<str>>(lines: &[T]) -> Option<usize> {

    let central_point = Point { x: 1, y: 1 };
    let mut canvas = TwoDimensionalWorld::default();
    let mut collisions = Vec::new();

    for (color, path) in lines.iter().enumerate() {
        let pcs = path.as_ref().split(',')
            .map(PenCommand::from_str)
            .map(Result::unwrap);
        let segments = ContinuousLineFromPen::line_segments_from(
            central_point.clone(),
            pcs);

        canvas.paint(segments, &color, &mut collisions);
    }

    collisions.sort_by_key(|(_, _, p)| p.manhattan_distance(&central_point));

    collisions.first().cloned().map(|(_color, _steps, p)| p)
        .map(|p| p.manhattan_distance(&canvas.central_port().unwrap()))
}

fn stage2<T: AsRef<str>>(lines: &[T]) -> Option<usize> {

    let central_point = Point { x: 1, y: 1 };
    let mut canvas = TwoDimensionalWorld::default();
    let mut collisions = Vec::new();

    for (color, path) in lines.iter().enumerate() {
        let pcs = path.as_ref().split(',')
            .map(PenCommand::from_str)
            .map(Result::unwrap);
        let segments = ContinuousLineFromPen::line_segments_from(
            central_point.clone(),
            pcs);

        canvas.paint(segments, &color, &mut collisions);
    }

    collisions.sort_by_key(|(_, steps, _)| *steps);

    collisions.first().cloned().map(|(_color, steps, _p)| steps)
}

#[derive(Default)]
struct TwoDimensionalWorld {
    first_color: HashMap<Point<usize>, (Color, Steps)>,
    bounds: Option<(Point<usize>, Point<usize>)>,
    first: Option<Point<usize>>,
}

type Steps = usize;

// which wire
type Color = usize;

impl TwoDimensionalWorld {
    /// The steps in collisions is the combined total steps so far to collide in that position
    fn paint<I>(&mut self, lss: I, color: &Color, collisions: &mut Vec<(Color, Steps, Point<usize>)>)
        where I: Iterator<Item = LineSegment<usize>>
    {

        let mut last = None;
        let mut steps = 0;
        for ls in lss {
            for p in ls.iter_including_start() {

                self.bounds = match self.bounds.take() {
                    Some(b) => Some((b.0.min(p), b.1.max(p))),
                    None => Some((p, p)),
                };

                // had a bug here
                let (new_last, last_changed) = match last.take() {
                    Some((0, x)) if x == p => (Some((1, p.clone())), false),
                    Some((_, x)) if x == p => panic!("Too many duplicate points: {:?}", p),
                    Some((_, _))
                    | None => (Some((0, p.clone())), true),
                };

                last = new_last;

                match (self.first, self.first_color.get(&p)) {
                    (Some(first), Some((first_color, first_steps)))
                        if first != p && first_color != color =>
                    {
                        collisions.push((
                            first_color.clone(),
                            dbg!(first_steps.clone()) + dbg!(steps.clone()),
                            p.clone()
                        ));
                    }
                    _ => {},
                }

                self.first = self.first.or(Some(p));

                self.first_color.insert(p.clone(), (color.clone(), steps.clone()));

                steps += if last_changed { 1 } else { 0 };
            }
        }
    }

    fn bounds(&self) -> &Option<(Point<usize>, Point<usize>)> {
        &self.bounds
    }

    fn central_port(&self) -> &Option<Point<usize>> {
        &self.first
    }
}

#[derive(PartialEq, Debug)]
struct LineSegment<T>(Point<T>, Point<T>);

impl LineSegment<usize> {
    fn iter_including_start(&self) -> PointIterator {
        PointIterator(self, None)
    }

    fn iter(&self) -> std::iter::Skip<PointIterator> {
        self.iter_including_start().skip(1)
    }
}

struct PointIterator<'a>(&'a LineSegment<usize>, Option<Point<usize>>);

impl<'a> Iterator for PointIterator<'a> {
    type Item = Point<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = &(self.0).0;
        let end = &(self.0).1;

        if self.1.as_ref() == Some(end) {
            // last call returned the end point, that was it.
            return None;
        }

        self.1 = Some(match self.1.take() {
            Some(last) => {
                let dx = end.x as isize - last.x as isize;
                let dy = end.y as isize - last.y as isize;

                if dx.abs() > dy.abs() {
                    Point { x: (last.x as isize + dx.signum()) as usize, y: last.y }
                } else if dy.abs() > dx.abs() {
                    Point { x: last.x, y: (last.y as isize + dy.signum()) as usize }
                } else {
                    unreachable!();
                }
            },
            None => start.clone(),
        });

        self.1.clone()
    }
}

impl<T> From<(Point<T>, Point<T>)> for LineSegment<T> {
    fn from((start, end): (Point<T>, Point<T>)) -> Self {
        Self(start, end)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Default, Copy)]
struct Point<T> { x: T, y: T }

impl<T> From<(T, T)> for Point<T> {
    fn from((x, y): (T, T)) -> Self {
        Self { x, y }
    }
}

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

impl PenCommand {
    fn to_line_segment_from(&self, start: Point<usize>) -> LineSegment<usize> {
        let (dx, dy): (isize, isize) = match self.0 {
            Direction::Up => (0, 1),
            Direction::Right => (1, 0),
            Direction::Down => (0, -1),
            Direction::Left => (-1, 0),
        };
        let amount = self.1 as isize;

        let (dx, dy) = (amount * dx, amount * dy);

        let end = Point { x: (start.x as isize + dx) as usize, y: (start.y as isize + dy) as usize };

        LineSegment(start, end)
    }
}

struct ContinuousLineFromPen<I> {
    inner: I,
    last: Point<usize>,
}

impl<I: Iterator<Item = PenCommand>> ContinuousLineFromPen<I> {
    fn line_segments_from(start: Point<usize>, iter: I) -> Self {
        Self {
            inner: iter,
            last: start,
        }
    }
}

impl<I: Iterator<Item = PenCommand>> Iterator for ContinuousLineFromPen<I> {
    type Item = LineSegment<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(pc) => {
                let ls = pc.to_line_segment_from(self.last.clone());
                self.last = ls.1.clone();
                Some(ls)
            },
            None => None,
        }
    }
}

trait ManhattanDistance {
    fn manhattan_distance(&self, to: &Self) -> usize;
}

impl ManhattanDistance for Point<usize> {
    fn manhattan_distance(&self, to: &Self) -> usize {
        ((to.x as isize - self.x as isize).abs()
            + (to.y as isize - self.y as isize).abs()) as usize
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

#[test]
fn pen_draw_example() {
    let start = Point { x: 1, y: 1 };

    let expected = &[
        (2, 1),
        (3, 1),
        (4, 1),
        (5, 1),
        (6, 1),
        (7, 1),
        (8, 1),
        (9, 1)
    ];

    let points = PenCommand(Direction::Right, 8)
        .to_line_segment_from(start)
        .iter()
        .collect::<Vec<Point<usize>>>();

    assert_eq!(points, expected.iter().cloned().map(Point::from).collect::<Vec<Point<usize>>>());
}

#[test]
fn line_segment_points_should_be_unique() {
    let ls = LineSegment((9usize, 6).into(), (4, 6).into());

    let mut last = None;

    for p in ls.iter() {
        last = match last.take() {
            Some(x) if x == p => panic!("got duplicate point: {:?}", p),
            Some(_)
            | None => Some(p),
        };
    };
}

#[test]
fn pen_full_line_example() {
    let start = Point { x: 1, y: 1 };

    let expected: &[LineSegment<usize>] = &[
        LineSegment((1, 1).into(), (9, 1).into()),
        LineSegment((9, 1).into(), (9, 6).into()),
        LineSegment((9, 6).into(), (4, 6).into()),
        LineSegment((4, 6).into(), (4, 3).into())
    ];

    let pcs = "R8,U5,L5,D3".split(',').map(PenCommand::from_str).map(Result::unwrap);

    let actual = ContinuousLineFromPen::line_segments_from(start, pcs).collect::<Vec<_>>();

    assert_eq!(&actual[..], &expected[..]);
}

#[test]
fn two_color_world_collisions() {

    let start = Point { x: 1, y: 1 };

    let lines = &[
        "R8,U5,L5,D3",
        "U7,R6,D4,L4",
    ];

    let expected = vec![
        (0, 30, Point { x: 7, y: 6 }),
        (0, 40, Point { x: 4, y: 4 }),
    ];

    let mut canvas = TwoDimensionalWorld::default();
    let mut collisions = Vec::new();

    for (color, path) in lines.iter().enumerate() {
        let pcs = path.split(',')
            .map(PenCommand::from_str)
            .map(Result::unwrap);
        let segments = ContinuousLineFromPen::line_segments_from(
            start.clone(),
            pcs);

        canvas.paint(segments, &color, &mut collisions);

        if color == 0 {
            assert!(collisions.is_empty());
        }
    }

    assert_eq!(collisions, expected);

    let bounds = canvas.bounds().clone().unwrap();

    assert_eq!(((1, 1).into(), (9, 6).into()), bounds);
}

#[test]
fn full_simplest_stage1_example() {

    let closest = stage1(&[
        "R8,U5,L5,D3",
        "U7,R6,D4,L4",
    ]);

    assert_eq!(6, closest.unwrap());
}

#[test]
fn full_stage1_example1() {

    let closest = stage1(&[
        "R75,D30,R83,U83,L12,D49,R71,U7,L72",
        "U62,R66,U55,R34,D71,R55,D58,R83",
    ]);

    assert_eq!(159, closest.unwrap());
}

#[test]
fn full_stage1_example2() {

    let closest = stage1(&[
        "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
        "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7",
    ]);

    assert_eq!(135, closest.unwrap());
}

#[test]
fn simplest_stage2_example() {
    let closest = stage2(&[
        "R8,U5,L5,D3",
        "U7,R6,D4,L4",
    ]);

    assert_eq!(30, closest.unwrap());
}

#[test]
fn full_stage2_example1() {

    let closest = stage2(&[
        "R75,D30,R83,U83,L12,D49,R71,U7,L72",
        "U62,R66,U55,R34,D71,R55,D58,R83",
    ]);

    assert_eq!(610, closest.unwrap());
}

#[test]
fn full_stage2_example2() {

    let closest = stage2(&[
        "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
        "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7",
    ]);

    assert_eq!(410, closest.unwrap());
}
