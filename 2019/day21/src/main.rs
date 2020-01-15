#![allow(dead_code)]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use std::marker::PhantomData;
use intcode::{Word, util::parse_stdin_program_n_lines, ExecutionState, Program, Registers};
use std::fmt;

fn main() {

    let data = parse_stdin_program_n_lines(Some(1));

    let part1 = random_stage::<Part1>(&data[..]);
    println!("stage1: {}", part1);

    let part2 = random_stage::<Part2>(&data[..]);
    println!("stage2: {}", part2);
    assert_eq!(part1, 19350258);
    assert_eq!(part2, 1142627861);
}

fn random_stage<T>(data: &[Word]) -> Word
    where Standard: Distribution<Op<T>>,
          T: Mode + Clone,
{
    let mut script_buffer = String::new();
    let mut output_buffer = String::new();

    let mut rng = rand::thread_rng();

    // initially thought about enumerating all possible programs but failed to do that with
    // permutohedron.. next idea: figure out packing mechanism to/from bytes and just generate
    // random numbers?

    let mut population = Vec::with_capacity(100);
    let mut testcases = Vec::new();

    let mut min_score = 1000;
    let mut max_score = 0;
    let mut round = 0;
    let mut prev_max_score = 0;
    let mut stale_rounds = 0;

    loop {
        let mut round_min = 10000;
        let mut round_max = 0;
        let mut mix_min = 10000;
        let mut mix_max = 0;

        let mut random = 0;
        let mut mixed = 0;

        while population.len() < population.capacity() {

            let program = rng.gen::<SpringScript<T>>();

            match score(&testcases, data, program.0.as_slice(), &mut script_buffer, &mut output_buffer) {
                Ok(x) => { println!("found on RANDOM on round {}", round); return x; },
                Err((testcase, score)) => {
                    if let Some(testcase) = testcase {
                        testcases.push(testcase);
                    }

                    min_score = min_score.min(score);
                    max_score = max_score.max(score);

                    round_min = round_min.min(score);
                    round_max = round_max.max(score);

                    random += 1;

                    population.push((program, score));
                },
            }
        }

        population.sort_by_key(|(_, score)| -(*score as isize));

        population.truncate(20);

        let remaining = population.len();

        for i in 0..remaining {

            {
                let orig = &population[i];
                let mut cloned = SpringScript::clone(&orig.0);
                cloned.mutate_one(&mut rng);

                match score(&testcases, data, cloned.as_slice(), &mut script_buffer, &mut output_buffer) {
                    Ok(x) => { println!("found on round {}", round); return x; },
                    Err((testcase, score)) => {
                        if let Some(testcase) = testcase {
                            testcases.push(testcase);
                        }
                        min_score = min_score.min(score);
                        max_score = max_score.max(score);

                        mix_min = mix_min.min(score);
                        mix_max = mix_max.max(score);

                        mixed += 1;
                        population.push((cloned, score));
                    },
                }
            }

            {
                let orig = &population[i];
                let mut cloned = SpringScript::clone(&orig.0);
                cloned.mutate_size(&mut rng);

                match score(&testcases, data, cloned.as_slice(), &mut script_buffer, &mut output_buffer) {
                    Ok(x) => { println!("found on round {}", round); return x; },
                    Err((testcase, score)) => {
                        if let Some(testcase) = testcase {
                            testcases.push(testcase);
                        }
                        min_score = min_score.min(score);
                        max_score = max_score.max(score);

                        mix_min = mix_min.min(score);
                        mix_max = mix_max.max(score);

                        mixed += 1;
                        population.push((cloned, score));
                    },
                }
            }
        }

        // mixing impl was tested up to 10000 rounds with no avail with length mixing, mutation
        // 0.15 and mixing of two... no luck. IDEA: try windowed mixing of 4 maybe?
        //
        // simply mutating gets the job done in ~200..50000 rounds, almost all best solutions come from
        // mutation. thanks birkenfeld.
        if prev_max_score != max_score {
            println!("{:<8}: min: {:>3}|{:>3}|{:<3} max: {:>3}|{:>3}|{:<3} random: {:<3} mixed: {:<3}", round, min_score, round_min, mix_min, max_score, round_max, mix_max, random, mixed);
            prev_max_score = max_score;
        } else {
            stale_rounds += 1;

            if stale_rounds > 100 {
                // this will cut down the time wasted significantly
                // 10 is too slow and might not give a result
                population.truncate(2);
                stale_rounds = 0;
            }
        }
        round += 1;
    }
}

#[inline(never)]
fn debug_print(i: usize, script: &str, output: &str) {
    println!("--- {}", i);
    println!("{}", script);
    println!("{}", output.trim());
    println!("---");
}

trait Mode {
    fn command() -> &'static str;
}

#[derive(Debug, PartialEq, Clone)]
struct Part1;
#[derive(Debug, PartialEq, Clone)]
struct Part2;

fn score<T: Mode>(testcases: &[Vec<bool>], data: &[Word], ops: &[Op<T>], script: &mut String, output: &mut String) -> Result<Word, (Option<Vec<bool>>, usize)> {

    // this idea is from https://github.com/birkenfeld/advent19/blob/master/src/bin/day21.rs didn't
    // think it was necessary but it is fast

    //let mut failed_testcase = None;
    'out: for (i, ground) in testcases.iter().enumerate() {
        let mut x = 0;

        while x < ground.len() {
            if !ground[x] {
                // dropped out
                //failed_testcase = Some(i);
                //break 'out;
                return Err((None, i));
            }

            let mut j = false;
            let mut t = false;

            for Op(instr, left, right, _) in ops.iter() {
                let left = match left {
                    ReadableRegister::RW(ReadWriteRegister::Jump) => j,
                    ReadableRegister::RW(ReadWriteRegister::Temporary) => t,
                    ReadableRegister::RO(offset) => ground.get(x + offset.distance()).copied().unwrap_or(true),
                };

                let target = match right {
                    ReadWriteRegister::Jump => &mut j,
                    ReadWriteRegister::Temporary => &mut t,
                };

                let right = *target;

                *target = match instr {
                    Instruction::And => left & right,
                    Instruction::Or => left | right,
                    Instruction::Not => !left,
                };
            }

            if j {
                x += 4;
            } else {
                x += 1;
            }
        }
    }

    match test(data, ops, script, output) {
        Some(x) => Ok(x),
        None => {
            let testcase = output.trim()
                .lines()
                .last()
                .unwrap()
                .chars()
                .map(|ch| ch == '#')
                .collect();

            Err((Some(testcase), testcases.len()))
        }
    }
}

fn test<T: Mode>(data: &[Word], ops: &[Op<T>], script: &mut String, output: &mut String) -> Option<Word> {
    use std::fmt::Write;

    let mut data = data.to_vec();
    let mut program = Program::wrap(&mut data)
        .with_memory_expansion();

    let mut regs = Registers::default();

    script.clear();
    for op in ops {
        write!(script, "{}\n", op).unwrap();
    }

    write!(script, "{}\n", T::command()).unwrap();

    let mut chars = script.chars();
    output.clear();

    loop {
        regs = match program.eval_from_instruction(regs).unwrap() {
            ExecutionState::Paused(_regs) => unreachable!("Pausing not implemented yet?"),
            ExecutionState::HaltedAt(_regs) => {
                // maybe use regs as score? or maybe should analyze the executed instructions?
                return None;
            },
            ExecutionState::InputIO(io) => {
                program.handle_input_completion(io, chars.next().unwrap() as Word).unwrap()
            },
            ExecutionState::OutputIO(io, value) => {

                if value > 127 {
                    println!("{}", script);
                    return Some(value);
                }

                output.push(value as u8 as char);
                program.handle_output_completion(io)
            },
        };
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ReadWriteRegister {
    Temporary,
    Jump,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ReadOnlyRegister {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
}

impl ReadOnlyRegister {
    fn distance(&self) -> usize {
        match *self {
            Self::A => 1,
            Self::B => 2,
            Self::C => 3,
            Self::D => 4,
            Self::E => 5,
            Self::F => 6,
            Self::G => 7,
            Self::H => 8,
            Self::I => 9,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ReadableRegister {
    RW(ReadWriteRegister),
    RO(ReadOnlyRegister),
}

impl Mode for Part1 {
    fn command() -> &'static str {
        "WALK"
    }
}

impl Mode for Part2 {
    fn command() -> &'static str {
        "RUN"
    }
}

#[derive(Debug, Clone)]
struct SpringScript<P>(Vec<Op<P>>);

impl<P> SpringScript<P>
    where Standard: Distribution<Op<P>>
{
    fn mutate_one<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let target = rng.gen_range(0, self.len());
        let next = rng.gen();

        if rng.gen_bool(0.33) {
            self[target].0 = next.0;
        } else if rng.gen_bool(0.5) {
            self[target].1 = next.1;
        } else {
            self[target].2 = next.2;
        }
    }

    fn mutate_size<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        if self.len() > 1 && self.len() < 15 {
            if rng.gen_bool(0.5) {
                self.insert_random(rng);
            } else {
                self.pop_random(rng);
            }
        } else if self.len() == 15 {
            self.pop_random(rng);
        } else {
            self.insert_random(rng);
        }
    }

    fn insert_random<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let pos = rng.gen_range(0, self.len());
        self.insert(pos, rng.gen());
    }

    fn pop_random<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let pos = rng.gen_range(0, self.len());
        self.remove(pos);
    }
}

impl<P> From<Vec<Op<P>>> for SpringScript<P> {
    fn from(v: Vec<Op<P>>) -> Self {
        SpringScript(v)
    }
}

impl<P> std::ops::Deref for SpringScript<P> {
    type Target = Vec<Op<P>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P> std::ops::DerefMut for SpringScript<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<P> fmt::Display for SpringScript<P> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for op in &self.0 {
            write!(fmt, "{}\n", op)?;
        }
        Ok(())
    }
}

impl<P> Distribution<SpringScript<P>> for Standard
    where Standard: Distribution<Op<P>>
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SpringScript<P> {
        let len = 1 + rng.gen_range(0, 15);

        let mut v = Vec::with_capacity(len);

        for _ in 0..len {
            v.push(rng.gen());
        }

        SpringScript(v)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Op<P>(Instruction, ReadableRegister, ReadWriteRegister, PhantomData<P>);

use rand::Rng;
use rand::distributions::{Distribution, Standard};

impl Distribution<Op<Part1>> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Op<Part1> {
        let seed = rng.gen_range(0u8, 3*6*2);

        Op::try_from(seed).unwrap()
    }
}

impl Distribution<Op<Part2>> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Op<Part2> {
        let seed = rng.gen_range(0u8, 3*11*2);

        Op::try_from(seed).unwrap()
    }
}

use std::convert::TryFrom;

impl TryFrom<u8> for Op<Part1> {
    type Error = u8;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        if val >= 3*6*2 {
            Err(val)
        } else {
            let instr = val % 3;
            let val = val / 3;
            let ro = val % 6;
            let val = val / 6;
            let rw = val % 2;

            let instr = Instruction::try_from(instr).unwrap();
            let ro = ReadableRegister::try_from(ro).unwrap();

            let rw = match rw {
                0 => ReadWriteRegister::Temporary,
                1 => ReadWriteRegister::Jump,
                x => unreachable!("out of range for rw: {}", x),
            };

            Ok(Op(instr, ro, rw, PhantomData))
        }
    }
}

impl TryFrom<u8> for Op<Part2> {
    type Error = u8;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        // so this is 66 ... 15 * 66 < u128...
        if val >= 3 * 11 * 2 {
            Err(val)
        } else {
            let instr = val % 3;
            let val = val / 3;
            let ro = val % 11;
            let val = val / 11;
            let rw = val % 2;

            let instr = Instruction::try_from(instr).unwrap();
            let ro = ReadableRegister::try_from(ro).unwrap();

            let rw = match rw {
                0 => ReadWriteRegister::Temporary,
                1 => ReadWriteRegister::Jump,
                x => unreachable!("out of range for rw: {}", x),
            };

            Ok(Op(instr, ro, rw, PhantomData))
        }
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
enum Instruction {
    And,
    Or,
    Not,
}

impl TryFrom<u8> for Instruction {
    type Error = u8;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        Ok(match val {
            0 => Self::And,
            1 => Self::Or,
            2 => Self::Not,
            x => return Err(x),
        })
    }
}

impl fmt::Display for ReadWriteRegister {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use ReadWriteRegister::*;
        let s = match *self {
            Temporary => "T",
            Jump => "J",
        };

        write!(fmt, "{}", s)
    }
}

impl fmt::Display for ReadOnlyRegister {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use ReadOnlyRegister::*;
        let s = match *self {
            A => "A",
            B => "B",
            C => "C",
            D => "D",
            E => "E",
            F => "F",
            G => "G",
            H => "H",
            I => "I",
        };

        write!(fmt, "{}", s)
    }
}

impl TryFrom<u8> for ReadOnlyRegister {
    type Error = u8;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        Ok(match val {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::E,
            5 => Self::F,
            6 => Self::G,
            7 => Self::H,
            8 => Self::I,
            x => return Err(x),
        })
    }
}

impl fmt::Display for ReadableRegister {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use ReadableRegister::*;
        match *self {
            RW(ref d) => write!(fmt, "{}", d),
            RO(ref d) => write!(fmt, "{}", d),
        }
    }
}

impl TryFrom<u8> for ReadableRegister {
    type Error = u8;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        Ok(match val {
            0 => Self::RW(ReadWriteRegister::Temporary),
            1 => Self::RW(ReadWriteRegister::Jump),
            x => Self::RO(ReadOnlyRegister::try_from(x - 2)?),
        })
    }
}

impl<T> fmt::Display for Op<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} {} {}", self.0, self.1, self.2)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Instruction::*;
        let s = match *self {
            And => "AND",
            Or => "OR",
            Not => "NOT",
        };

        write!(fmt, "{}", s)
    }
}
