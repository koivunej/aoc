mod instr;
mod error;
mod util;
mod env;
mod exec;

pub use error::*;
pub use util::{ParsingError, parse_program};
pub use env::Environment;
pub use exec::{Program, ExecutionState};

type Word = isize;

#[derive(Default, Debug, Clone)]
pub struct Registers {
    ip: usize,
    relbase: Word,
}

impl Registers {
    fn at(self, ip: usize) -> Self {
        Registers { ip, relbase: self.relbase }
    }

    fn at_increment(self, amount: usize) -> Self {
        let ip = self.ip;
        self.at(ip + amount)
    }

    pub fn instruction_pointer(&self) -> usize {
        self.ip
    }

    fn with_relbase(self, new_relbase: Word) -> Self {
        Registers { ip: self.ip, relbase: new_relbase }
    }

    fn with_relbase_increment(self, added: Word) -> Self {
        let relbase = self.relbase;
        self.with_relbase(relbase + added)
    }

    fn ip_rel(&self, offset: usize) -> usize {
        self.ip + offset
    }
}


trait IO {
    fn input(&mut self) -> Result<Word, ProgramError>;
    fn output(&mut self, value: Word) -> Result<(), ProgramError>;
}

trait Params {
    type Parameter: Param;

    fn mode(&self, index: usize) -> &Self::Parameter;
}

trait Param {
    fn read(self, arg: Word, relbase: Word, memory: &[Word]) -> Result<Word, InvalidReadAddress>;
    fn write(self, value: Word, arg: Word, relbase: Word, memory: &mut [Word]) -> Result<(), BadWrite>;
}

trait DecodedOperation {
    type Parameters: Params;

    fn unpack(self) -> (instr::OpCode, Self::Parameters);
    fn default_parameters(&self) -> bool;
}

trait Decoder {
    type Operation: DecodedOperation;

    fn decode(&self, ip: usize, value: Word) -> Result<Self::Operation, InvalidProgram>;
}
