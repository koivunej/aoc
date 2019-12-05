use std::convert::TryFrom;
use smallvec::SmallVec;

#[derive(Debug)]
pub enum OpCode {
    BinOp(BinOp),
    Store,
    Print,
    Halt,
}

#[doc(hidden)]
pub struct UnknownOpCode(isize);

impl TryFrom<isize> for OpCode {
    type Error = UnknownOpCode;
    fn try_from(u: isize) -> Result<Self, Self::Error> {
        Ok(match u {
            1 => OpCode::BinOp(BinOp::Add),
            2 => OpCode::BinOp(BinOp::Mul),
            3 => OpCode::Store,
            4 => OpCode::Print,
            99 => OpCode::Halt,
            x => { return Err(UnknownOpCode(x)); },
        })
    }
}

struct ParameterModes {
    modes: SmallVec<[ParameterMode; 4]>,
}

impl TryFrom<isize> for ParameterModes {
    type Error = ();

    fn try_from(data: isize) -> Result<ParameterModes, Self::Error> {
        if data < 100 {
            Ok(ParameterModes { modes: SmallVec::new() })
        } else {
            unimplemented!();
        }
    }
}

static DEFAULT_PARAMETER_MODE: ParameterMode = ParameterMode::Address;

impl ParameterModes {
    fn mode(&self, index: usize) -> &ParameterMode {
        self.modes.get(index).unwrap_or(&DEFAULT_PARAMETER_MODE)
    }
}

enum ParameterMode {
    Address,
    Immediate
}

impl ParameterMode {
    fn eval(&self, arg: isize, program: &[isize]) -> isize {
        match *self {
            ParameterMode::Address => {
                assert!(arg >= 0);
                program[arg as usize]
            },
            ParameterMode::Immediate => arg,
        }
    }

    fn store(&self, value: isize, arg: isize, program: &mut [isize]) {
        match *self {
            ParameterMode::Address => {
                assert!(arg >= 0);
                program[arg as usize] = value;
            },
            ParameterMode::Immediate => panic!("Cannot store on immediate"),
        }
    }
}

impl OpCode {
    fn len(&self) -> usize {
        match *self {
            OpCode::BinOp(_) => 4,
            OpCode::Store => 3,
            OpCode::Print => 2,
            OpCode::Halt => 1,
        }
    }
}

#[derive(Debug)]
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
}

impl From<(usize, UnknownOpCode)> for InvalidProgram {
    fn from((instruction_pointer, u): (usize, UnknownOpCode)) -> Self {
        let UnknownOpCode(op) = u;
        let error = ProgramError::UnknownOpCode(op);
        InvalidProgram {
            instruction_pointer,
            error
        }
    }
}

/// Configuration for the virtual machine; default will provide the minimum required.
#[derive(Default)]
pub struct Config {
    allow_op3: bool,
    allow_op4: bool,
}

impl Config {
    fn validate(&self, ip: usize, op: Result<OpCode, UnknownOpCode>) -> Result<OpCode, InvalidProgram> {
        match (op, self.allow_op3, self.allow_op4) {
            (Ok(x @ OpCode::Halt), _, _)
            | (Ok(x @ OpCode::BinOp(_)), _, _) => Ok(x),

            (Ok(x @ OpCode::Store), allow, _)
            | (Ok(x @ OpCode::Print), _, allow) => {
                if allow {
                    Ok(x)
                } else {
                    Err(InvalidProgram::unsupported(ip, x))
                }
            },

            (Err(e), _, _) => Err((ip, e).into()),
        }
    }
}

pub struct Program<'a> {
    prog: &'a mut [isize],
}

impl<'a> Program<'a> {
    fn eval(&mut self, config: &Config) -> Result<usize, InvalidProgram> {
        let mut ip = 0;
        loop {
            let next = OpCode::try_from(self.prog[ip]);
            let next = config.validate(ip, next)?;
            let skipped = match next {
                OpCode::Halt => return Ok(ip),
                OpCode::BinOp(b) => {

                    let pvs = ParameterModes::try_from(self.prog[ip])
                        .expect("Failed to deduce parameter modes");

                    let first = pvs.mode(0);
                    let second = pvs.mode(1);
                    let third = pvs.mode(2);

                    let res = b.eval(
                        first.eval(self.prog[ip + 1], &self.prog),
                        second.eval(self.prog[ip + 2], &self.prog));

                    third.store(res, self.prog[ip + 3], &mut self.prog);

                    OpCode::BinOp(b).len()
                },
                x => unimplemented!("{:?}", x),
            };

            ip = (ip + skipped) % self.prog.len();
        }
    }

    /// Returns Ok(instruction_pointer) for the halt instruction
    pub fn wrap_and_eval(data: &mut [isize], config: &Config) -> Result<usize, InvalidProgram> {
        let mut p = Program { prog: data };
        p.eval(config)
    }
}

#[cfg(test)]
mod tests {
    use super::{Program, Config};

    #[test]
    fn stage1_example() {
        let mut prog = vec![
            1, 9, 10, 3,
            2, 3, 11, 0,
            99,
            30, 40, 50];

        let expected = &[
            3500isize, 9, 10, 70,
            2, 3, 11, 0,
            99,
            30, 40, 50];

        Program::wrap_and_eval(&mut prog, &Config::default());

        assert_eq!(&prog[..], expected);
    }
}
