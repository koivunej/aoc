use std::convert::TryFrom;
use instr::{Operation, OpCode};

mod instr;
mod error;
mod util;
mod env;

pub use error::*;
pub use util::{ParsingError, parse_program};
pub use env::Environment;

/// Configuration for the virtual machine; default will provide the minimum required.
/// Basically betting for restrictions on the VM operation... Not sure why.
#[derive(Default)]
pub struct Config {
}

impl Config {
    pub fn day05() -> Self {
        Self::default()
    }
}

trait IO {
    fn input(&mut self) -> Result<isize, ProgramError>;
    fn output(&mut self, value: isize) -> Result<(), ProgramError>;
}

trait Params {
    type Parameter: Param;

    fn mode(&self, index: usize) -> &Self::Parameter;
}

trait Param {
    fn read(self, arg: isize, memory: &[isize]) -> Result<isize, InvalidReadAddress>;
    fn write(self, value: isize, arg: isize, memory: &mut [isize]) -> Result<(), BadWrite>;
}

trait DecodedOperation {
    type Parameters: Params;

    fn unpack(self) -> (instr::OpCode, Self::Parameters);
    fn default_parameters(&self) -> bool;
}

trait Decoder {
    type Operation: DecodedOperation;

    fn decode(&self, ip: usize, value: isize) -> Result<Self::Operation, InvalidProgram>;
}

pub struct Program<'a> {
    mem: &'a mut [isize],
    env: &'a mut Environment,
}

enum State {
    Running(usize),
    HaltedAt(usize),
}

impl<'a> Program<'a> {

    fn exec(&mut self, ip: usize, op: Operation) -> Result<State, ProgramError> {
        let (code, pvs) = op.unpack();
        let ip = match code {
            OpCode::Halt => return Ok(State::HaltedAt(ip)),
            OpCode::BinOp(b) => {
                let first = pvs.mode(0);
                let second = pvs.mode(1);
                let third = pvs.mode(2);

                let res = b.eval(
                    first.read(self.mem[ip + 1], &self.mem)?,
                    second.read(self.mem[ip + 2], &self.mem)?,
                );

                third.write(res, self.mem[ip + 3], &mut self.mem)?;

                ip + 4
            }
            OpCode::Store => {
                let target = pvs.mode(0);
                let input = self.env.input()?;
                target.write(input, self.mem[ip + 1], &mut self.mem)?;

                ip + 2
            }
            OpCode::Print => {
                let value = pvs.mode(0).read(self.mem[ip + 1], &self.mem)?;
                self.env.output(value)?;

                ip + 2
            }
            OpCode::Jump(cond) => {
                let cmp = pvs.mode(0).read(self.mem[ip + 1], &self.mem)?;
                let target = pvs.mode(1).read(self.mem[ip + 2], &self.mem)?;

                if cond.eval(cmp) {
                    if target < 0 {
                        return Err(ProgramError::NegativeJump(target));
                    }
                    target as usize
                } else {
                    ip + 3
                }
            }
            OpCode::StoreCompared(bincond) => {
                let first = pvs.mode(0).read(self.mem[ip + 1], &self.mem)?;
                let second = pvs.mode(1).read(self.mem[ip + 2], &self.mem)?;
                let target = pvs.mode(2);

                let res = if bincond.eval(first, second) { 1 } else { 0 };
                target.write(res, self.mem[ip + 3], &mut self.mem)?;

                ip + 4
            }
        };

        Ok(State::Running(ip))
    }

    fn step(&mut self, ip: usize) -> Result<State, InvalidProgram> {
        self.decode(ip)
            .and_then(|op| self.exec(ip, op))
            .map_err(|e| e.at(ip))
    }

    fn decode(&self, ip: usize) -> Result<Operation, ProgramError> {
        Operation::try_from(self.mem[ip])
            .map_err(ProgramError::from)
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
        _config: &Config,
    ) -> Result<usize, InvalidProgram> {
        let mut p = Program {
            mem: data,
            env,
        };
        p.eval()
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, Environment, Program};
    use super::instr::{BinOp, OpCode, ParameterMode, ParameterModes};
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

        let output = env.unwrap_input_consumed_once();

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
