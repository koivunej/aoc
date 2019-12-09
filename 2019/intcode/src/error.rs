use crate::{Registers, Word};

#[derive(Debug)]
pub struct InvalidProgram {
    registers: Registers,
    pub error: ProgramError,
}

#[derive(Debug)]
pub enum ProgramError {
    Decoding(DecodingError),
    NoMoreInput,
    CannotOutput,
    NegativeJump(Word),
    InvalidReadAddress(Word),
    BadWrite(BadWrite),
}

#[derive(Debug)]
pub enum DecodingError {
    UnknownOpCode(Word),
    InvalidParameterMode(Word),
    TooManyParameters(Word),
}

pub(crate) struct InvalidReadAddress(pub(crate) Word);

impl From<InvalidReadAddress> for ProgramError {
    fn from(InvalidReadAddress(addr): InvalidReadAddress) -> Self {
        ProgramError::InvalidReadAddress(addr)
    }
}

impl From<BadWrite> for ProgramError {
    fn from(b: BadWrite) -> Self {
        ProgramError::BadWrite(b)
    }
}

impl From<DecodingError> for ProgramError {
    fn from(d: DecodingError) -> Self {
        ProgramError::Decoding(d)
    }
}

#[derive(Debug)]
pub enum BadWrite {
    NegativeAddress(Word),
    AddressOutOfBounds(usize),
    ImmediateParameter,
}

impl ProgramError {
    pub(crate) fn at(self, registers: Registers) -> InvalidProgram {
        InvalidProgram {
            registers,
            error: self,
        }
    }
}
