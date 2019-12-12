use std::str::FromStr;
use std::fmt;
use std::io::Read;
//use itertools::Itertools;

fn main() {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    let mut locked = stdin.lock();

    locked.read_to_string(&mut buffer).unwrap();

    let initial = System {
        time: 0,
        bodies: buffer.lines()
            .map(Body::from_str)
            .collect::<Result<_, _>>()
            .unwrap()
    };

    let s = initial
        .clone()
        .into_iter()
        .nth(1000)
        .unwrap();

    println!("stage1: {}", s.total_energy());

    let mut initial_again = initial.clone();
    initial_again.step_until_eq(&initial);

    println!("stage2: {}", initial_again.time);
}

#[derive(Clone, PartialEq)]
struct System {
    time: u64,
    bodies: Vec<Body>,
}

impl fmt::Debug for System {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "After {} steps:", self.time)?;
        for body in &self.bodies {
            writeln!(fmt, "{:?}", body)?;
        }
        writeln!(fmt, "")
    }
}

impl System {
    fn total_energy(&self) -> i32 {
        self.bodies.iter()
            .map(Body::total_energy)
            .sum()
    }

    fn step(&self) -> System {

        let mut bodies = Vec::with_capacity(self.bodies.len());

        for a in self.bodies.iter() {
            let mut vel = a.vel.clone();
            for b in self.bodies.iter() {
                for ((a, b), v) in a.pos.iter().zip(b.pos.iter()).zip(vel.iter_mut()) {
                    *v = *v + (b - a).signum();
                }
            }

            let mut pos = a.pos.clone();

            for (v, p) in vel.iter().zip(pos.iter_mut()) {
                *p = *p + v;
            }

            bodies.push(Body { pos, vel });
        }

        System { time: self.time + 1, bodies }
    }

    #[allow(dead_code)]
    fn step_mut(&mut self, tmp: &mut Vec<Body>) {
        // too slow

        tmp.clear();

        for a in self.bodies.iter() {
            let mut vel = a.vel.clone();
            for b in self.bodies.iter() {
                for ((a, b), v) in a.pos.iter().zip(b.pos.iter()).zip(vel.iter_mut()) {
                    *v = *v + (b - a).signum();
                }
            }

            let mut pos = a.pos.clone();

            for (v, p) in vel.iter().zip(pos.iter_mut()) {
                *p = *p + v;
            }

            tmp.push(Body { pos, vel });
        }

        std::mem::swap(&mut self.bodies, tmp);

        if (self.time + 1) % 1_000_000 == 0 {
            println!("{}", self.time + 1);
        }
        self.time += 1;
    }

    fn step_until_eq(&mut self, other: &Self) {
        use num::Integer;

        // could not do this without a hint ... threading is really extra for this

        let (a, b, c) = if false {
            (self.partition_off(0, other), self.partition_off(1, other), self.partition_off(2, other))
        } else {
            // this might faster by a millisecond
            let a = self.partition_off_thread(0, other);
            let b = self.partition_off_thread(1, other);
            let c = self.partition_off_thread(2, other);

            (a.join().unwrap(), b.join().unwrap(), c.join().unwrap())
        };

        let ((a, xs), (b, ys), (c, zs)) = (a, b, c);

        // each of the axes is periodic and we need to find suitable time when they all align
        let time = a.lcm(&b.lcm(&c));

        self.time += time;

        for i in 0..4 {
            self.bodies[i].pos = [xs[i], ys[i], zs[i]];
            self.bodies[i].vel = [0, 0, 0];
        }
    }

    fn partition_off_thread(&self, axis: usize, other: &Self) -> std::thread::JoinHandle<(u64, [i32; 4])> {
        let other = other.clone();

        let mut cs = [0i32; 4];
        cs.copy_from_slice(self.bodies.iter().map(|b| b.pos[axis]).collect::<Vec<_>>().as_slice());
        let mut vcs = [0i32; 4];
        vcs.copy_from_slice(self.bodies.iter().map(|b| b.vel[axis]).collect::<Vec<_>>().as_slice());
        let mut expected = [0i32; 4];
        expected.copy_from_slice(other.bodies.iter().map(|b| b.pos[axis]).collect::<Vec<_>>().as_slice());

        std::thread::spawn(move || Self::run_axis_period(cs, vcs, expected, [0, 0, 0, 0]))
    }

    fn run_axis_period(mut cs: [i32; 4], mut vcs: [i32; 4], expected_pos: [i32; 4], expected_vel: [i32; 4]) -> (u64, [i32; 4]) {
        for steps in 1u64.. {
            for i in 0..4 {
                for j in 0..4 {
                    vcs[i] += (cs[j] - cs[i]).signum();
                }
            }

            for i in 0..4 {
                cs[i] += vcs[i];
            }

            if vcs == expected_vel && cs == expected_pos {
                return (steps, cs);
            }
        }

        unreachable!()
    }

    fn partition_off(&self, axis: usize, other: &Self) -> (u64, [i32; 4]) {

        let mut cs = [0i32; 4];
        cs.copy_from_slice(self.bodies.iter().map(|b| b.pos[axis]).collect::<Vec<_>>().as_slice());

        let mut vcs = [0i32; 4];
        vcs.copy_from_slice(self.bodies.iter().map(|b| b.vel[axis]).collect::<Vec<_>>().as_slice());

        let mut expected = [0i32; 4];
        expected.copy_from_slice(other.bodies.iter().map(|b| b.pos[axis]).collect::<Vec<_>>().as_slice());

        Self::run_axis_period(cs, vcs, expected, [0, 0, 0, 0])
    }

    fn into_iter(self) -> Steps {
        Steps { s: self }
    }
}

struct Steps {
    s: System
}


impl Iterator for Steps {
    type Item = System;

    fn next(&mut self) -> Option<System> {
        let ret = Some(self.s.clone());
        self.s = self.s.step();
        ret
    }
}

#[derive(Clone, PartialEq)]
struct Body {
    pos: [i32; 3],
    vel: [i32; 3],
}

impl fmt::Debug for Body {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "<{:>3?} {:>3?}>", self.pos, self.vel)
    }
}

impl Body {
    fn total_energy(&self) -> i32 { self.potential_energy() * self.kinetic_energy() }
    fn potential_energy(&self) -> i32 { Self::energy(&self.pos) }
    fn kinetic_energy(&self) -> i32 { Self::energy(&self.vel) }

    fn energy(vals: &[i32]) -> i32 {
        vals.iter().copied().map(i32::abs).sum()
    }
}

#[derive(Debug, PartialEq)]
enum BodyParsingError {
    ExtraElements,
    MissingElements,
    Form,
    InvalidNum(std::num::ParseIntError),
}

impl From<std::num::ParseIntError> for BodyParsingError {
    fn from(e: std::num::ParseIntError) -> Self {
        BodyParsingError::InvalidNum(e)
    }
}

impl FromStr for Body {
    type Err = BodyParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use BodyParsingError::*;

        let mut split = s.split(',');

        let x = split.next().ok_or(MissingElements)?;
        let y = split.next().ok_or(MissingElements)?.trim();
        let z = split.next().ok_or(MissingElements)?.trim();

        if split.next().is_some() {
            return Err(ExtraElements);
        }

        // well this is horrific

        let x = i32::from_str(x.split('=').skip(1).next().ok_or(Form)?)?;
        let y = i32::from_str(y.split('=').skip(1).next().ok_or(Form)?)?;
        let z = z.split('=').skip(1).next()
            .and_then(|s| s.split('>').next()).ok_or(Form)
            .and_then(|s| s.parse().map_err(BodyParsingError::from))?;

        Ok(Body {
            pos: [x, y, z],
            vel: [0, 0, 0],
        })
    }
}

impl From<&([i32; 3], [i32; 3])> for Body {
    fn from((pos, vel): &([i32; 3], [i32; 3])) -> Self {
        Body { pos: *pos, vel: *vel }
    }
}

#[test]
fn parse_body() {
    let input = "<x=-1, y=0, z=2>";
    assert_eq!([-1, 0, 2], Body::from_str(input).unwrap().pos)
}

#[test]
fn example_system1() {
    let input = "<x=-1, y=0, z=2>
<x=2, y=-10, z=-7>
<x=4, y=-8, z=8>
<x=3, y=5, z=-1>";

    let s = System { time: 0, bodies: input.lines().map(|s| s.parse::<Body>()).collect::<Result<_, _>>().unwrap() }.into_iter().nth(10).unwrap();

    let expected = System {
        time: 10,
        bodies: [
            ([2, 1, -3], [-3,-2, 1]),
            ([1,-8,  0], [-1, 1, 3]),
            ([3,-6,  1], [ 3, 2,-3]),
            ([2, 0,  4], [ 1,-1,-1])
        ].into_iter().map(Body::from).collect()
    };

    assert_eq!(s, expected);
    assert_eq!(s.total_energy(), 179);
}

#[test]
fn example_system2() {
    let input = "<x=-8, y=-10, z=0>
<x=5, y=5, z=10>
<x=2, y=-7, z=3>
<x=9, y=-8, z=-3>";

    let s = System { time: 0, bodies: input.lines().map(|s| s.parse::<Body>()).collect::<Result<_, _>>().unwrap() }.into_iter().nth(100).unwrap();

    let expected = System {
        time: 100,
        bodies: [
            ([  8,-12,-9], [-7,  3, 0]),
            ([ 13, 16, -3], [ 3,-11,-5]),
            ([-29,-11, -1], [-3,  7, 4]),
            ([ 16,-13, 23], [ 7,  1, 1])
        ].into_iter().map(Body::from).collect()
    };

    assert_eq!(s, expected);
    assert_eq!(s.total_energy(), 1940);
}

#[test]
fn step_until_initial() {
    let input =
"<x=-1, y=0, z=2>
<x=2, y=-10, z=-7>
<x=4, y=-8, z=8>
<x=3, y=5, z=-1>";

    let mut s = System { time: 0, bodies: input.lines().map(|s| s.parse::<Body>()).collect::<Result<_, _>>().unwrap() };

    let initial = s.clone();

    s.step_until_eq(&initial);

    assert_eq!(s.time, 2772, "{:?}", s);
}
