mod instr;
mod error;
mod util;
mod env;
mod exec;

pub use error::*;
pub use util::{ParsingError, parse_stdin_program, with_parsed_program};
pub use env::Environment;
pub use exec::{Program, ExecutionState};

pub type Word = i64;

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

pub struct Memory<'a> {
    mem: &'a mut [Word],
    expansion: Option<Vec<Word>>, // None if expanded memory is not supported
}

impl<'a> From<&'a mut [Word]> for Memory<'a> {
    fn from(mem: &'a mut [Word]) -> Self {
        Memory {
            mem,
            expansion: None,
        }
    }
}

impl<'a> Memory<'a> {

    fn read(&self, addr: usize) -> Result<Word, InvalidReadAddress> {
        if addr < self.mem.len() {
            self.mem.get(addr as usize)
                .cloned()
                .ok_or(InvalidReadAddress(addr as Word))
        } else if let Some(expanded) = self.expansion.as_ref() {
            Ok(*expanded.get(addr - self.mem.len()).unwrap_or(&0))
        } else {
            Err(InvalidReadAddress(addr as Word))
        }
    }

    fn write(&mut self, addr: usize, value: Word) -> Result<(), BadWrite> {
        if addr < self.mem.len() {
            let cell = self.mem.get_mut(addr).ok_or(BadWrite::AddressOutOfBounds(addr))?;
            *cell = value;
            Ok(())
        } else if let Some(expanded) = self.expansion.as_mut() {
            let addr = addr - self.mem.len();
            if expanded.len() <= addr + 1 {
                expanded.resize(addr + 1, 0);
            }
            expanded[addr] = value;
            Ok(())
        } else {
            Err(BadWrite::AddressOutOfBounds(addr))
        }
    }

    fn get(&self, addr: usize) -> Option<&Word> {
        if addr < self.mem.len() {
            self.mem.get(addr)
        } else if let Some(expanded) = self.expansion.as_ref() {
            Some(expanded.get(addr - self.mem.len()).unwrap_or(&0))
        } else {
            None
        }
    }

    fn with_memory_expansion(mut self) -> Self {
        assert!(self.expansion.is_none());
        self.expansion = Some(Vec::new());
        self
    }
}

impl<'a> std::ops::Index<usize> for Memory<'a> {
    type Output = Word;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.mem.len() {
            &self.mem[index]
        } else if let Some(expanded) = self.expansion.as_ref() {
            &expanded[index - self.mem.len()]
        } else {
            panic!("Index out of bounds: {}", index);
        }
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
    fn read(self, arg: Word, relbase: Word, memory: &Memory) -> Result<Word, InvalidReadAddress>;
    fn write(self, value: Word, arg: Word, relbase: Word, memory: &mut Memory) -> Result<(), BadWrite>;
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
