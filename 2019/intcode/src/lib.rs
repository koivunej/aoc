mod instr;
mod error;
pub mod util;
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

/// Custom version of std::borrow::Cow which does not work on mutable borrows.
enum RawMemory<'a> {
    Borrowed(&'a mut [Word]),
    Owned(Vec<Word>),
}

impl<'a> Clone for RawMemory<'a> {
    fn clone(&self) -> RawMemory<'static> {
        match *self {
            RawMemory::Borrowed(ref uniq) => RawMemory::Owned(uniq.to_vec()),
            RawMemory::Owned(ref v) => RawMemory::Owned(v.clone()),
        }
    }
}

impl<'a> From<&'a mut [Word]> for RawMemory<'a> {
    fn from(m: &'a mut [Word]) -> Self {
        RawMemory::Borrowed(m)
    }
}

impl<'a> std::ops::Deref for RawMemory<'a> {
    type Target = [Word];

    fn deref(&self) -> &Self::Target {
        match self {
            &Self::Borrowed(ref data) => data,
            &Self::Owned(ref data) => data.as_slice(),
        }
    }
}

impl<'a> std::ops::DerefMut for RawMemory<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            &mut Self::Borrowed(ref mut data) => data,
            &mut Self::Owned(ref mut data) => data.as_mut_slice(),
        }
    }
}

impl<'a> RawMemory<'a> {
    fn into_owned(self) -> RawMemory<'static> {
        match self {
            Self::Borrowed(data) => RawMemory::Owned(data.to_vec()),

            // this is quite surprising that it needs to be repeated but the left side is
            // RawMemory<'a> but right side is RawMemory<'static>
            Self::Owned(data) => RawMemory::Owned(data),
        }
    }
}

/// State of a single program.
#[derive(Clone)]
pub struct Memory<'a> {
    mem: RawMemory<'a>,
    expansion: Option<Vec<Word>>, // None if expanded memory is not supported
}

impl<'a> From<&'a mut [Word]> for Memory<'a> {
    fn from(mem: &'a mut [Word]) -> Self {
        Memory {
            mem: RawMemory::from(mem),
            expansion: None,
        }
    }
}

impl<'a> From<&'a [Word]> for Memory<'static> {
    fn from(mem: &'a [Word]) -> Self {
        Memory {
            mem: RawMemory::Owned(mem.to_vec()),
            expansion: None,
        }
    }
}

impl From<Vec<Word>> for Memory<'static> {
    fn from(mem: Vec<Word>) -> Self {
        Memory {
            mem: RawMemory::Owned(mem),
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

    pub fn into_owned(self) -> Memory<'static> {
        Memory {
            mem: self.mem.into_owned(),
            expansion: self.expansion,
        }
    }

    pub fn with_memory_expansion(mut self) -> Self {
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
