use std::convert::TryFrom;

enum OpCode {
    BinOp(BinOp),
    Halt,
}

impl OpCode {
    fn len(&self) -> usize {
        match *self {
            OpCode::BinOp(_) => 4,
            OpCode::Halt => 1,
        }
    }
}

#[derive(Debug)]
enum BinOp {
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

struct UnknownOpCode(isize);

impl TryFrom<isize> for OpCode {
    type Error = UnknownOpCode;
    fn try_from(u: isize) -> Result<Self, Self::Error> {
        Ok(match u {
            1 => OpCode::BinOp(BinOp::Add),
            2 => OpCode::BinOp(BinOp::Mul),
            99 => OpCode::Halt,
            x => { return Err(UnknownOpCode(x)); },
        })
    }
}

#[derive(Debug)]
pub struct InvalidProgram {
    pub instruction_pointer: usize,
    pub error: ProgramError,
}

#[derive(Debug)]
pub enum ProgramError {
    UnknownOpCode(isize),
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
            (Err(e), _, _) => Err((ip, e).into()),
        }
    }
}

pub struct Program<'a> {
    prog: &'a mut [isize],
}

impl<'a> Program<'a> {
    fn indirect_value(&self, index: usize) -> isize {
        let index = index % self.prog.len();
        let addr = self.prog[index];
        assert!(addr >= 0);
        let val = self.prog[addr as usize];
        val
    }

    fn indirect_store(&mut self, index: usize, value: isize) {
        let index = index % self.prog.len();
        let addr = self.prog[index];
        assert!(addr >= 0);
        // probably shouldn't panic?
        self.prog[addr as usize] = value;
    }

    fn eval(&mut self, config: &Config) -> Result<usize, InvalidProgram> {
        let mut ip = 0;
        loop {
            let next = OpCode::try_from(self.prog[ip]);
            let next = config.validate(ip, next)?;
            let skipped = match next {
                OpCode::Halt => return Ok(ip),
                OpCode::BinOp(b) => {

                    let res = b.eval(
                        self.indirect_value(ip + 1),
                        self.indirect_value(ip + 2));

                    self.indirect_store(ip + 3, res);

                    OpCode::BinOp(b).len()
                },
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
    use super::Program;

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

        Program::wrap_and_eval(&mut prog);

        assert_eq!(&prog[..], expected);
    }
}
