use crate::env::Environment;
use crate::{IO, DecodedOperation};
use crate::error::{InvalidProgram, ProgramError};
use crate::instr::{Operation, OpCode, ParameterModes};
use std::convert::TryFrom;

pub struct Program<'a> {
    mem: &'a mut [isize],
}

enum State {
    Running(/* instruction_pointer */ usize),
    WaitingInput(Input),
    WaitingToOutput(Output, isize),
    HaltedAt(/* instruction_pointer */ usize),
}

pub struct Input {
    instruction_pointer: usize,
    parameters: ParameterModes,
}

pub struct Output {
    instruction_pointer: usize,
    hidden: bool,
}

pub enum ExecutionState {
    HaltedAt(usize),
    InputIO(Input),
    OutputIO(Output, isize),
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
                return Ok(State::WaitingInput(Input { instruction_pointer: ip, parameters: pvs }));
            }
            OpCode::Print => {
                let value = pvs.mode(0).read(self.mem[ip + 1], &self.mem)?;
                return Ok(State::WaitingToOutput(Output { instruction_pointer: ip, hidden: false }, value));
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

    pub fn eval_from_instruction(&mut self, mut ip: usize) -> Result<ExecutionState, InvalidProgram> {
        loop {
            ip = match self.step(ip)? {
                State::Running(jump_to) => jump_to,
                State::HaltedAt(ip) => return Ok(ExecutionState::HaltedAt(ip)),
                State::WaitingInput(io) => return Ok(ExecutionState::InputIO(io)),
                State::WaitingToOutput(io, val) => return Ok(ExecutionState::OutputIO(io, val)),
            };
        }
    }

    pub fn handle_input_completion(&mut self, input: Input, value: isize) -> Result<usize, InvalidProgram> {
        let Input { instruction_pointer: ip, parameters } = input;
        parameters.mode(0).write(value, self.mem[ip + 1], &mut self.mem)
            .map_err(|e| ProgramError::from(e).at(ip))?;
        Ok(ip + 2)
    }

    pub fn handle_output_completion(&mut self, output: Output) -> usize {
        let Output { instruction_pointer: ip, hidden: _hidden } = output;
        ip + 2
    }

    pub fn wrap(mem: &'a mut [isize]) -> Program<'a> {
        Program { mem }
    }

    /// Returns Ok(instruction_pointer) for the halt instruction
    pub fn wrap_and_eval(data: &mut [isize]) -> Result<usize, InvalidProgram> {
        Self::wrap_and_eval_with_env(data, &mut Environment::default())
    }

    pub fn wrap_and_eval_with_env(
        data: &mut [isize],
        env: &mut Environment,
    ) -> Result<usize, InvalidProgram> {
        let mut p = Program {
            mem: data,
        };
        p.eval_with_env(env)
    }

    fn eval_with_env(&mut self, env: &mut Environment) -> Result<usize, InvalidProgram> {
        // I feel like this could be an instance property but it does not necessarily need to be?
        let mut ip = 0;
        loop {
            ip = match self.eval_from_instruction(ip)? {
                ExecutionState::HaltedAt(ip) => return Ok(ip),
                ExecutionState::InputIO(io) => {
                    let input = env.input().map_err(|e| e.at(io.instruction_pointer))?;
                    self.handle_input_completion(io, input)?
                },
                ExecutionState::OutputIO(io, value) => {
                    env.output(value).map_err(|e| e.at(io.instruction_pointer))?;
                    self.handle_output_completion(io)
                },
            };
        }
    }

}


