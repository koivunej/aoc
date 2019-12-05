use crate::error::ProgramError;
use crate::IO;

#[derive(Debug)]
pub enum Environment {
    NoIO,
    Once(Option<isize>, Option<isize>),
    Collector(Option<isize>, Vec<isize>),
}

impl std::default::Default for Environment {
    fn default() -> Self {
        Self::NoIO
    }
}

impl Environment {
    fn input(&mut self) -> Result<isize, ProgramError> {
        match *self {
            Environment::NoIO => Err(ProgramError::NoMoreInput),
            Environment::Once(ref mut input, _) | Environment::Collector(ref mut input, _) => input
                .take()
                .ok_or(ProgramError::NoMoreInput),
        }
    }

    fn output(&mut self, value: isize) -> Result<(), ProgramError> {
        match *self {
            Environment::NoIO => Err(ProgramError::CannotOutput),
            Environment::Once(_, ref mut output) => {
                if output.is_some() {
                    Err(ProgramError::CannotOutput)
                } else {
                    *output = Some(value);
                    Ok(())
                }
            }
            Environment::Collector(_, ref mut collected) => {
                collected.push(value);
                Ok(())
            }
        }
    }

    pub fn once(input: Option<isize>) -> Self {
        Self::Once(input, None)
    }

    pub fn collector(input: Option<isize>) -> Self {
        Self::Collector(input, Vec::new())
    }

    pub fn unwrap_input_consumed_once(self) -> Option<isize> {
        match self.unwrap_once() {
            (None, x) => x,
            (Some(unconsumed), _) => unreachable!("Input {} was not consumed", unconsumed),
        }
    }

    pub fn unwrap_once(self) -> (Option<isize>, Option<isize>) {
        match self {
            Environment::Once(input, output) => (input, output),
            x => unreachable!("Was not once: {:?}", x),
        }
    }

    pub fn unwrap_collected(self) -> Vec<isize> {
        match self {
            Environment::Collector(_, collected) => collected,
            x => unreachable!("Was not collector: {:?}", x),
        }
    }
}

impl IO for Environment {
    fn input(&mut self) -> Result<isize, ProgramError> { self.input() }
    fn output(&mut self, value: isize) -> Result<(), ProgramError> { self.output(value) }
}

