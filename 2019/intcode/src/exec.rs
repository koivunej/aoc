use crate::{Registers, Word, Memory};
use crate::env::Environment;
use crate::{IO, DecodedOperation};
use crate::error::{InvalidProgram, ProgramError};
use crate::instr::{Operation, OpCode, ParameterModes};
use std::convert::TryFrom;

#[derive(Clone)]
pub struct Program<'a> {
    mem: Memory<'a>,
}

enum State {
    Running(Registers),
    WaitingInput(Input),
    WaitingToOutput(Output, Word),
    HaltedAt(Registers),
}

pub struct Input {
    registers: Registers,
    parameters: ParameterModes,
}

impl Input {
    fn registers(&self) -> Registers { self.registers.clone() }
}

pub struct Output(Registers);

impl Output {
    fn registers(&self) -> Registers { self.0.clone() }
}

pub enum ExecutionState {
    Paused(Registers),
    HaltedAt(Registers),
    InputIO(Input),
    OutputIO(Output, Word),
    // here could be paused for coop scheduling?
}

impl std::default::Default for ExecutionState {
    fn default() -> Self {
        ExecutionState::Paused(Registers::default())
    }
}

impl<'a> From<Memory<'a>> for Program<'a> {
    fn from(mem: Memory<'a>) -> Self {
        Program {
            mem
        }
    }
}

impl<'a> Program<'a> {

    fn exec(&mut self, regs: Registers, op: Operation) -> Result<State, ProgramError> {
        let (code, pvs) = op.unpack();
        let regs = match code {
            OpCode::Halt => return Ok(State::HaltedAt(regs)),
            OpCode::BinOp(b) => {
                let first = pvs.mode(0);
                let second = pvs.mode(1);
                let third = pvs.mode(2);

                let res = b.eval(
                    first.read(self.mem[regs.ip_rel(1)], regs.relbase, &self.mem)?,
                    second.read(self.mem[regs.ip_rel(2)], regs.relbase, &self.mem)?,
                );

                third.write(res, self.mem[regs.ip_rel(3)], regs.relbase, &mut self.mem)?;

                regs.at_increment(4)
            }
            OpCode::Store => {
                return Ok(State::WaitingInput(Input { registers: regs, parameters: pvs }));
            }
            OpCode::Print => {
                let value = pvs.mode(0).read(self.mem[regs.ip_rel(1)], regs.relbase, &self.mem)?;
                return Ok(State::WaitingToOutput(Output(regs), value));
            }
            OpCode::Jump(cond) => {
                let cmp = pvs.mode(0).read(self.mem[regs.ip_rel(1)], regs.relbase, &self.mem)?;
                let target = pvs.mode(1).read(self.mem[regs.ip_rel(2)], regs.relbase, &self.mem)?;

                if cond.eval(cmp) {
                    if target < 0 {
                        return Err(ProgramError::NegativeJump(target));
                    }
                    regs.at(target as usize)
                } else {
                    regs.at_increment(3)
                }
            }
            OpCode::StoreCompared(bincond) => {
                let first = pvs.mode(0).read(self.mem[regs.ip_rel(1)], regs.relbase, &self.mem)?;
                let second = pvs.mode(1).read(self.mem[regs.ip_rel(2)], regs.relbase, &self.mem)?;
                let target = pvs.mode(2);

                let res = if bincond.eval(first, second) { 1 } else { 0 };
                target.write(res, self.mem[regs.ip_rel(3)], regs.relbase, &mut self.mem)?;

                regs.at_increment(4)
            },
            OpCode::AdjustRelative => {
                let added = pvs.mode(0).read(self.mem[regs.ip_rel(1)], regs.relbase, &self.mem)?;

                regs.with_relbase_increment(added)
                    .at_increment(2)
            }
        };

        Ok(State::Running(regs))
    }

    fn step(&mut self, registers: Registers) -> Result<State, InvalidProgram> {
        let reg_clone = registers.clone();
        self.mem.get(registers.instruction_pointer())
            .ok_or_else(|| ProgramError::InvalidReadAddress(registers.instruction_pointer() as Word))
            .and_then(|value| self.decode(*value))
            .and_then(move |op| self.exec(registers, op))
            .map_err(|e| e.at(reg_clone))
    }

    fn decode(&self, value: Word) -> Result<Operation, ProgramError> {
        Ok(Operation::try_from(value)?)
    }

    pub fn eval_from_instruction(&mut self, mut regs: Registers) -> Result<ExecutionState, InvalidProgram> {
        loop {
            regs = match self.step(regs)? {
                State::Running(regs) => regs,
                State::HaltedAt(regs) => return Ok(ExecutionState::HaltedAt(regs)),
                State::WaitingInput(io) => return Ok(ExecutionState::InputIO(io)),
                State::WaitingToOutput(io, val) => return Ok(ExecutionState::OutputIO(io, val)),
            };
        }
    }

    pub fn handle_input_completion(&mut self, input: Input, value: Word) -> Result<Registers, InvalidProgram> {
        let Input { registers: regs, parameters } = input;
        parameters.mode(0)
            .write(value, self.mem[regs.ip_rel(1)], regs.relbase, &mut self.mem)
            .map_err(|e| ProgramError::from(e).at(regs.clone()))?;
        Ok(regs.at_increment(2))
    }

    pub fn handle_output_completion(&mut self, Output(regs): Output) -> Registers {
        regs.at_increment(2)
    }

    pub fn with_memory_expansion(self) -> Self {
        Program { mem: self.mem.with_memory_expansion() }
    }

    pub fn wrap(mem: &'a mut [Word]) -> Program<'a> {
        Program { mem: Memory::from(mem) }
    }

    /// Returns Ok(instruction_pointer) for the halt instruction
    pub fn wrap_and_eval(data: &mut [Word]) -> Result<usize, InvalidProgram> {
        Self::wrap_and_eval_with_env(data, &mut Environment::default())
    }

    pub fn wrap_and_eval_with_env(
        data: &mut [Word],
        env: &mut Environment,
    ) -> Result<usize, InvalidProgram> {
        let mut p = Program {
            mem: Memory::from(data),
        };
        p.eval_with_env(env)
    }

    pub fn eval_with_env(&mut self, env: &mut Environment) -> Result<usize, InvalidProgram> {
        // I feel like this could be an instance property but it does not necessarily need to be?
        let mut regs = Registers::default();
        loop {
            regs = match self.eval_from_instruction(regs)? {
                ExecutionState::Paused(_regs) => unreachable!("Pausing not implemented yet?"),
                ExecutionState::HaltedAt(regs) => return Ok(regs.instruction_pointer()),
                ExecutionState::InputIO(io) => {
                    let input = env.input().map_err(|e| e.at(io.registers()))?;
                    self.handle_input_completion(io, input)?
                },
                ExecutionState::OutputIO(io, value) => {
                    env.output(value).map_err(|e| e.at(io.registers()))?;
                    self.handle_output_completion(io)
                },
            };
        }
    }

}


