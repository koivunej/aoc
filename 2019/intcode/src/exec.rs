use crate::env::Environment;
use crate::{IO, DecodedOperation};
use crate::error::{InvalidProgram, ProgramError};
use crate::instr::{Operation, OpCode};
use std::convert::TryFrom;

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
        self.mem.get(ip)
            .ok_or_else(|| ProgramError::InvalidReadAddress(ip as isize))
            .and_then(|value| self.decode(*value))
            .and_then(|op| self.exec(ip, op))
            .map_err(|e| e.at(ip))
    }

    fn decode(&self, value: isize) -> Result<Operation, ProgramError> {
        Ok(Operation::try_from(value)?)
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
    pub fn wrap_and_eval(data: &mut [isize], config: &crate::Config) -> Result<usize, InvalidProgram> {
        Self::wrap_and_eval_with_env(data, &mut Environment::default(), config)
    }

    pub fn wrap_and_eval_with_env(
        data: &mut [isize],
        env: &mut Environment,
        _config: &crate::Config,
    ) -> Result<usize, InvalidProgram> {
        let mut p = Program {
            mem: data,
            env,
        };
        p.eval()
    }
}


