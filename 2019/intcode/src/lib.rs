mod instr;
mod error;
mod util;
mod env;
mod exec;

pub use error::*;
pub use util::{ParsingError, parse_program};
pub use env::Environment;
pub use exec::{Program, ExecutionState};

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
