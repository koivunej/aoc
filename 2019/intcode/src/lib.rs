use smallvec::SmallVec;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub enum OpCode {
    BinOp(BinOp),
    Store,
    Print,
    Jump(UnaryCondition),
    StoreCompared(BinaryCondition),
    Halt,
}

#[derive(Debug, PartialEq)]
pub enum UnaryCondition {
    OnTrue,
    OnFalse,
}

impl UnaryCondition {
    fn eval(&self, first: isize) -> bool {
        match *self {
            Self::OnTrue => first != 0,
            Self::OnFalse => first == 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BinaryCondition {
    OnLessThan,
    OnEq,
}

impl BinaryCondition {
    fn eval(&self, first: isize, second: isize) -> bool {
        match *self {
            Self::OnLessThan => first < second,
            Self::OnEq => first == second,
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct UnknownOpCode(isize);

impl TryFrom<isize> for OpCode {
    type Error = UnknownOpCode;
    fn try_from(u: isize) -> Result<Self, Self::Error> {
        Ok(match u % 100 {
            1 => OpCode::BinOp(BinOp::Add),
            2 => OpCode::BinOp(BinOp::Mul),
            3 => OpCode::Store,
            4 => OpCode::Print,
            5 => OpCode::Jump(UnaryCondition::OnTrue),
            6 => OpCode::Jump(UnaryCondition::OnFalse),
            7 => OpCode::StoreCompared(BinaryCondition::OnLessThan),
            8 => OpCode::StoreCompared(BinaryCondition::OnEq),
            99 => OpCode::Halt,
            x => {
                return Err(UnknownOpCode(x));
            }
        })
    }
}

struct ParameterModes {
    modes: SmallVec<[ParameterMode; 4]>,
}

#[derive(Debug)]
enum ParameterModeError {
    NegativeOpCode,
    InvalidMode(isize),
}

impl TryFrom<isize> for ParameterModes {
    type Error = ParameterModeError;

    fn try_from(raw: isize) -> Result<ParameterModes, Self::Error> {
        if !Self::instruction_has_modes(raw) {
            Ok(ParameterModes {
                modes: SmallVec::new(),
            })
        } else if raw > 0 {
            let mut shifted = raw / 100;
            let mut pm = ParameterModes {
                modes: SmallVec::new(),
            };
            while shifted > 0 {
                let rem = shifted % 10;
                if rem > 1 {
                    return Err(ParameterModeError::InvalidMode(rem));
                }
                pm.modes.push(if rem == 1 {
                    ParameterMode::Immediate
                } else {
                    ParameterMode::Address
                });

                shifted /= 10;
            }
            Ok(pm)
        } else {
            Err(ParameterModeError::NegativeOpCode)
        }
    }
}

static DEFAULT_PARAMETER_MODE: ParameterMode = ParameterMode::Address;

impl ParameterModes {
    fn mode(&self, index: usize) -> &ParameterMode {
        self.modes.get(index).unwrap_or(&DEFAULT_PARAMETER_MODE)
    }

    #[allow(dead_code)]
    fn is_default(&self) -> bool {
        // when none were specified we have only defaults
        self.modes.is_empty() || self.modes.iter().all(|pm| pm == &DEFAULT_PARAMETER_MODE)
    }

    fn instruction_has_modes(raw: isize) -> bool {
        raw > 100
    }

    fn all_must_equal_default(self) -> Self {
        if !self.modes.is_empty() {
            assert!(self.modes.iter().all(|pm| pm == &DEFAULT_PARAMETER_MODE));
        }
        self
    }

    fn at_most(mut self, count: usize) -> Self {
        assert!(self.modes.len() <= count);
        self.modes
            .extend(std::iter::repeat(DEFAULT_PARAMETER_MODE).take(count - self.modes.len()));

        self
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum ParameterMode {
    Address,
    Immediate,
}

impl ParameterMode {
    fn eval(&self, arg: isize, program: &[isize]) -> isize {
        match *self {
            ParameterMode::Address => {
                assert!(arg >= 0);
                program[arg as usize]
            }
            ParameterMode::Immediate => arg,
        }
    }

    fn store(&self, value: isize, arg: isize, program: &mut [isize]) {
        match *self {
            ParameterMode::Address => {
                assert!(arg >= 0);
                program[arg as usize] = value;
            }
            ParameterMode::Immediate => panic!("Cannot store on immediate"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BinOp {
    Add,
    Mul,
}

impl BinOp {
    fn eval(&self, lhs: isize, rhs: isize) -> isize {
        match *self {
            BinOp::Add => lhs.checked_add(rhs).expect("Add overflow"),
            BinOp::Mul => lhs.checked_mul(rhs).expect("Mul overflow"),
        }
    }
}

#[derive(Debug)]
pub struct InvalidProgram {
    pub instruction_pointer: usize,
    pub error: ProgramError,
}

impl InvalidProgram {
    fn unsupported(instruction_pointer: usize, o: OpCode) -> Self {
        Self {
            instruction_pointer,
            error: ProgramError::Unsupported(o),
        }
    }
}

#[derive(Debug)]
pub enum ProgramError {
    UnknownOpCode(isize),
    Unsupported(OpCode),
    NoMoreInput,
    CannotOutput,
}

impl From<(usize, UnknownOpCode)> for InvalidProgram {
    fn from((instruction_pointer, u): (usize, UnknownOpCode)) -> Self {
        let UnknownOpCode(op) = u;
        let error = ProgramError::UnknownOpCode(op);
        InvalidProgram {
            instruction_pointer,
            error,
        }
    }
}

impl From<(usize, ProgramError)> for InvalidProgram {
    fn from((instruction_pointer, error): (usize, ProgramError)) -> Self {
        InvalidProgram {
            instruction_pointer,
            error,
        }
    }
}

/// Configuration for the virtual machine; default will provide the minimum required.
#[derive(Default)]
pub struct Config {
    parameter_modes: bool,
}

impl Config {
    pub fn day05() -> Self {
        Config {
            parameter_modes: true,
            ..Self::default()
        }
    }

    fn validate(
        &self,
        raw: isize,
        ip: usize,
        op: Result<OpCode, UnknownOpCode>,
    ) -> Result<OpCode, InvalidProgram> {
        if !self.parameter_modes && ParameterModes::instruction_has_modes(raw) {
            return Err((ip, UnknownOpCode(raw)).into());
        }

        // first about allowing each op, ... too much boilerplate
        op.map_err(|e| (ip, e).into())
    }
}

#[derive(Debug)]
pub enum Environment {
    NoIO,
    Once(Option<isize>, Option<isize>),
    Collector(Option<isize>, Vec<isize>),
}

impl std::default::Default for Environment {
    fn default() -> Self {
        Self::NoIO
    }
}

impl Environment {
    fn input(&mut self, ip: usize) -> Result<isize, InvalidProgram> {
        match *self {
            Environment::NoIO => Err((ip, ProgramError::NoMoreInput).into()),
            Environment::Once(ref mut input, _) | Environment::Collector(ref mut input, _) => input
                .take()
                .ok_or_else(|| (ip, ProgramError::NoMoreInput).into()),
        }
    }

    fn output(&mut self, ip: usize, value: isize) -> Result<(), InvalidProgram> {
        match *self {
            Environment::NoIO => Err((ip, ProgramError::CannotOutput).into()),
            Environment::Once(_, ref mut output) => {
                if output.is_some() {
                    Err((ip, ProgramError::CannotOutput).into())
                } else {
                    *output = Some(value);
                    Ok(())
                }
            }
            Environment::Collector(_, ref mut collected) => Ok(collected.push(value)),
        }
    }

    pub fn once(input: Option<isize>) -> Self {
        Self::Once(input, None)
    }

    pub fn collector(input: Option<isize>) -> Self {
        Self::Collector(input, Vec::new())
    }

    pub fn unwrap_once(self) -> (Option<isize>, Option<isize>) {
        match self {
            Environment::Once(input, output) => (input, output),
            x => unreachable!("Was not once: {:?}", x),
        }
    }

    pub fn unwrap_collected(self) -> Vec<isize> {
        match self {
            Environment::Collector(_, collected) => collected,
            x => unreachable!("Was not collector: {:?}", x),
        }
    }
}

pub struct Program<'a> {
    prog: &'a mut [isize],
}

impl<'a> Program<'a> {
    fn eval(&mut self, env: &mut Environment, config: &Config) -> Result<usize, InvalidProgram> {
        let mut ip = 0;
        loop {
            let op = self.prog[ip];
            let next = OpCode::try_from(op);
            let next = config.validate(op, ip, next)?;

            let jump_target = match next {
                OpCode::Halt => return Ok(ip),
                OpCode::BinOp(b) => {
                    let pvs = ParameterModes::try_from(op)
                        .expect("Failed to deduce parameter modes")
                        .at_most(3);

                    let first = pvs.mode(0);
                    let second = pvs.mode(1);
                    let third = pvs.mode(2);

                    let res = b.eval(
                        first.eval(self.prog[ip + 1], &self.prog),
                        second.eval(self.prog[ip + 2], &self.prog),
                    );

                    third.store(res, self.prog[ip + 3], &mut self.prog);

                    ip + 4
                }
                OpCode::Store => {
                    // this cannot have parameter modes...
                    let pvs = ParameterModes::try_from(op)
                        .expect("Failed to deduce parameter modes")
                        .all_must_equal_default()
                        .at_most(1);

                    let target = pvs.mode(0);
                    let input = env.input(ip)?;
                    target.store(input, self.prog[ip + 1], &mut self.prog);

                    ip + 2
                }
                OpCode::Print => {
                    let pvs = ParameterModes::try_from(op)
                        .expect("Failed to deduce parameter modes")
                        .at_most(1);

                    let source = pvs.mode(0);
                    env.output(ip, source.eval(self.prog[ip + 1], &self.prog))?;

                    ip + 2
                }
                OpCode::Jump(cond) => {
                    let pvs = ParameterModes::try_from(op)
                        .expect("Failed to deduce parameter modes")
                        .at_most(2);

                    let cmp = pvs.mode(0).eval(self.prog[ip + 1], &self.prog);
                    let target = pvs.mode(1).eval(self.prog[ip + 2], &self.prog);

                    if cond.eval(cmp) {
                        target as usize
                    } else {
                        ip + 3
                    }
                }
                OpCode::StoreCompared(bincond) => {
                    let pvs = ParameterModes::try_from(op)
                        .expect("Failed to deduce parameter modes")
                        .at_most(3);

                    let first = pvs.mode(0).eval(self.prog[ip + 1], &self.prog);
                    let second = pvs.mode(1).eval(self.prog[ip + 2], &self.prog);
                    let target = pvs.mode(2);

                    let res = if bincond.eval(first, second) { 1 } else { 0 };
                    target.store(res, self.prog[ip + 3], &mut self.prog);

                    ip + 4
                }
            };

            ip = jump_target;
        }
    }

    /// Returns Ok(instruction_pointer) for the halt instruction
    pub fn wrap_and_eval(data: &mut [isize], config: &Config) -> Result<usize, InvalidProgram> {
        Self::wrap_and_eval_with_env(data, &mut Environment::default(), config)
    }

    pub fn wrap_and_eval_with_env(
        data: &mut [isize],
        env: &mut Environment,
        config: &Config,
    ) -> Result<usize, InvalidProgram> {
        let mut p = Program { prog: data };
        p.eval(env, config)
    }
}

#[derive(Debug)]
pub enum ParsingError {
    Io(std::io::Error, usize),
    Int(std::num::ParseIntError, usize, String),
}

pub fn parse_program<R: std::io::BufRead>(mut r: R) -> Result<Vec<isize>, ParsingError> {
    use std::str::FromStr;

    let mut data = vec![];
    let mut buffer = String::new();
    let mut line = 0;

    loop {
        buffer.clear();
        let bytes = r
            .read_line(&mut buffer)
            .map_err(|e| ParsingError::Io(e, line))?;

        if bytes == 0 {
            return Ok(data);
        }

        let parts = buffer.trim().split(',').map(isize::from_str);

        for part in parts {
            let part = match part {
                Ok(part) => part,
                Err(e) => return Err(ParsingError::Int(e, line, buffer)),
            };

            data.push(part);
        }

        line += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_program, BinOp, Config, Environment, OpCode, ParameterMode, ParameterModes, Program,
    };
    use std::convert::TryFrom;

    #[test]
    fn stage1_example() {
        let mut prog = vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50];

        let expected = &[3500isize, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50];

        Program::wrap_and_eval(&mut prog, &Config::default()).unwrap();

        assert_eq!(&prog[..], expected);
    }

    #[test]
    fn io_example() {
        let mut prog = vec![3, 0, 4, 0, 99];
        let expected = &[1, 0, 4, 0, 99];

        let mut env = Environment::Once(Some(1), None);

        Program::wrap_and_eval_with_env(&mut prog, &mut env, &Config::day05()).unwrap();

        let (input, output) = env.unwrap_once();

        assert_eq!(input, None);
        assert_eq!(output, Some(1));
        assert_eq!(&prog[..], expected);
    }

    #[test]
    fn parse_opcode_with_modes() {
        let input = 1002;

        assert_eq!(OpCode::try_from(input).unwrap(), OpCode::BinOp(BinOp::Mul));

        let pm = ParameterModes::try_from(input).unwrap();
        assert_eq!(pm.mode(0), &ParameterMode::Address);
        assert_eq!(pm.mode(1), &ParameterMode::Immediate);
        assert_eq!(pm.mode(2), &ParameterMode::Address);
    }

    #[test]
    fn day05_stage2_eq_ne_example() {
        let data = &[3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        assert_eq!(day05_stage2_example_scenario(data, 8), 1);
        assert_eq!(day05_stage2_example_scenario(data, 7), 0);
    }

    #[test]
    fn day05_stage2_lt_example() {
        let data = &[3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        assert_eq!(day05_stage2_example_scenario(data, 7), 1);
        assert_eq!(day05_stage2_example_scenario(data, 8), 0);
    }

    #[test]
    fn day05_stage2_eq_ne_example_immediate() {
        let data = &[3, 3, 1108, -1, 8, 3, 4, 3, 99];
        assert_eq!(day05_stage2_example_scenario(data, 8), 1);
        assert_eq!(day05_stage2_example_scenario(data, 7), 0);
    }

    #[test]
    fn day05_stage2_lt_example_immediate() {
        let data = &[3, 3, 1107, -1, 8, 3, 4, 3, 99];
        assert_eq!(day05_stage2_example_scenario(data, 7), 1);
        assert_eq!(day05_stage2_example_scenario(data, 8), 0);
    }

    #[test]
    fn day05_stage2_input_eq_0() {
        let addressed = &[3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        let immediate = &[3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        for code in &[&addressed[..], &immediate[..]] {
            assert_eq!(day05_stage2_example_scenario(code, 0), 0);
            assert_eq!(day05_stage2_example_scenario(code, 2), 1);
        }
    }

    fn day05_stage2_example_scenario(data: &[isize], input: isize) -> isize {
        let mut prog = data.to_vec();
        let mut env = Environment::once(Some(input));
        let conf = Config::day05();

        Program::wrap_and_eval_with_env(&mut prog, &mut env, &conf).unwrap();

        let (input, output) = env.unwrap_once();
        assert_eq!(input, None);
        output.unwrap()
    }

    #[test]
    fn day05_stage2_larger_example() {
        let code = &[
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];

        // careful exploration of the whole state space :)
        let params = &[(6, 999), (7, 999), (8, 1000), (9, 1001), (10, 1001)];

        for (input, expected) in params {
            let mut prog = code.to_vec();
            let mut env = Environment::collector(Some(*input));
            let conf = Config::day05();

            Program::wrap_and_eval_with_env(&mut prog, &mut env, &conf).unwrap();

            let output = env.unwrap_collected();
            assert_eq!(&output[..], &[*expected]);
        }
    }
}
