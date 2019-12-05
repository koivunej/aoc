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

impl OpCode {
    fn parameters(&self) -> usize {
        match *self {
            OpCode::BinOp(_) => 3,
            OpCode::Store => 1,
            OpCode::Print => 1,
            OpCode::Jump(_) => 2,
            OpCode::StoreCompared(_) => 2,
            OpCode::Halt => 0,
        }
    }

    fn only_default_parameters(&self) -> bool {
        if let OpCode::Store = *self {
            true
        } else {
            false
        }
    }
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

impl TryFrom<isize> for OpCode {
    type Error = DecodingError;
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
                return Err(DecodingError::UnknownOpCode(x));
            }
        })
    }
}

#[derive(Debug)]
struct ParameterModes {
    modes: SmallVec<[ParameterMode; 4]>,
}

impl TryFrom<isize> for ParameterModes {
    type Error = DecodingError;

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
                    return Err(DecodingError::InvalidParameterMode(rem));
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
            unreachable!("Negative values must be handled before calling this method");
        }
    }
}

#[derive(Debug)]
pub struct Operation(OpCode, ParameterModes);

#[derive(Debug)]
pub enum DecodingError {
    UnknownOpCode(isize),
    InvalidParameterMode(isize),
    TooManyParameters(isize),
}

impl TryFrom<isize> for Operation {
    type Error = DecodingError;

    fn try_from(raw: isize) -> Result<Self, Self::Error> {
        if raw < 0 {
            return Err(DecodingError::UnknownOpCode(raw));
        }

        let op = OpCode::try_from(raw)?;
        let mut pvs = ParameterModes::try_from(raw)?
            .at_most(op.parameters())
            .map_err(|_| DecodingError::TooManyParameters(raw))?;

        if op.only_default_parameters() {
            pvs = pvs
                .all_must_equal_default()
                .map_err(|_| DecodingError::InvalidParameterMode(raw))?;
        }

        Ok(Operation(op, pvs))
    }
}

static DEFAULT_PARAMETER_MODE: ParameterMode = ParameterMode::Address;

impl ParameterModes {
    fn mode(&self, index: usize) -> &ParameterMode {
        self.modes.get(index).unwrap_or(&DEFAULT_PARAMETER_MODE)
    }

    fn is_default(&self) -> bool {
        self.modes.is_empty() || self.modes.iter().all(|pm| pm == &DEFAULT_PARAMETER_MODE)
    }

    fn instruction_has_modes(raw: isize) -> bool {
        raw > 100
    }

    fn all_must_equal_default(self) -> Result<Self, ()> {
        if self.modes.is_empty() || self.modes.iter().all(|pm| pm == &DEFAULT_PARAMETER_MODE) {
            Ok(self)
        } else {
            Err(())
        }
    }

    fn at_most(mut self, count: usize) -> Result<Self, ()> {
        use std::iter::repeat;
        if self.modes.len() <= count {
            self.modes
                .extend(repeat(DEFAULT_PARAMETER_MODE).take(count - self.modes.len()));
            Ok(self)
        } else {
            Err(())
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum ParameterMode {
    Address,
    Immediate,
}

impl ParameterMode {
    fn eval(self, arg: isize, program: &[isize]) -> isize {
        match self {
            ParameterMode::Address => {
                assert!(arg >= 0);
                program[arg as usize]
            }
            ParameterMode::Immediate => arg,
        }
    }

    fn store(self, value: isize, arg: isize, program: &mut [isize]) {
        match self {
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

#[derive(Debug)]
pub enum ProgramError {
    Decoding(DecodingError),
    Unsupported(Operation),
    NoMoreInput,
    CannotOutput,
    NegativeJump(isize),
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
/// Basically betting for restrictions on the VM operation... Not sure why.
#[derive(Default)]
pub struct Config {
    parameter_modes: bool,
}

impl Config {
    pub fn day05() -> Self {
        Config {
            parameter_modes: true,
        }
    }

    fn validate(&self, ip: usize, op: Operation) -> Result<Operation, InvalidProgram> {
        if !self.parameter_modes && !op.1.is_default() {
            return Err((ip, ProgramError::Unsupported(op)).into());
        }

        // first about allowing each op, ... too much boilerplate
        Ok(op)
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
            Environment::Collector(_, ref mut collected) => {
                collected.push(value);
                Ok(())
            }
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
    mem: &'a mut [isize],
    env: &'a mut Environment,
    config: &'a Config,
}

enum State {
    Running(usize),
    HaltedAt(usize),
}

impl<'a> Program<'a> {
    fn step(&mut self, ip: usize) -> Result<State, InvalidProgram> {
        let Operation(op, pvs) = self.decode(ip)?;

        let ip = match op {
            OpCode::Halt => return Ok(State::HaltedAt(ip)),
            OpCode::BinOp(b) => {
                let first = pvs.mode(0);
                let second = pvs.mode(1);
                let third = pvs.mode(2);

                let res = b.eval(
                    first.eval(self.mem[ip + 1], &self.mem),
                    second.eval(self.mem[ip + 2], &self.mem),
                );

                third.store(res, self.mem[ip + 3], &mut self.mem);

                ip + 4
            }
            OpCode::Store => {
                let target = pvs.mode(0);
                let input = self.env.input(ip)?;
                target.store(input, self.mem[ip + 1], &mut self.mem);

                ip + 2
            }
            OpCode::Print => {
                let source = pvs.mode(0);
                self.env
                    .output(ip, source.eval(self.mem[ip + 1], &self.mem))?;

                ip + 2
            }
            OpCode::Jump(cond) => {
                let cmp = pvs.mode(0).eval(self.mem[ip + 1], &self.mem);
                let target = pvs.mode(1).eval(self.mem[ip + 2], &self.mem);

                if cond.eval(cmp) {
                    if target < 0 {
                        return Err((ip, ProgramError::NegativeJump(target)).into());
                    }
                    target as usize
                } else {
                    ip + 3
                }
            }
            OpCode::StoreCompared(bincond) => {
                let first = pvs.mode(0).eval(self.mem[ip + 1], &self.mem);
                let second = pvs.mode(1).eval(self.mem[ip + 2], &self.mem);
                let target = pvs.mode(2);

                let res = if bincond.eval(first, second) { 1 } else { 0 };
                target.store(res, self.mem[ip + 3], &mut self.mem);

                ip + 4
            }
        };

        Ok(State::Running(ip))
    }

    fn decode(&self, ip: usize) -> Result<Operation, InvalidProgram> {
        Operation::try_from(self.mem[ip])
            .map_err(|dec| (ip, ProgramError::Decoding(dec)).into())
            .and_then(|op| self.config.validate(ip, op))
    }

    fn eval(&mut self) -> Result<usize, InvalidProgram> {
        let mut ip = 0;
        loop {
            ip = match self.step(ip)? {
                State::HaltedAt(ip) => return Ok(ip),
                State::Running(jump_to) => jump_to,
            };
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
        let mut p = Program {
            mem: data,
            env,
            config,
        };
        p.eval()
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
    use super::{BinOp, Config, Environment, OpCode, ParameterMode, ParameterModes, Program};
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
