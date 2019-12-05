#[derive(Debug)]
pub struct InvalidProgram {
    pub instruction_pointer: usize,
    pub error: ProgramError,
}

#[derive(Debug)]
pub enum ProgramError {
    Decoding(DecodingError),
    NoMoreInput,
    CannotOutput,
    NegativeJump(isize),
    InvalidReadAddress(isize),
    BadWrite(BadWrite),
}

#[derive(Debug)]
pub enum DecodingError {
    UnknownOpCode(isize),
    InvalidParameterMode(isize),
    TooManyParameters(isize),
}

pub(crate) struct InvalidReadAddress(pub(crate) isize);

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
    AddressOutOfBounds,
    ImmediateParameter,
}

impl ProgramError {
    pub(crate) fn at(self, instruction_pointer: usize) -> InvalidProgram {
        InvalidProgram {
            instruction_pointer,
            error: self,
        }
    }
}
