use crate::error::ProgramError;
use crate::{IO, Word};
use std::collections::VecDeque;

#[derive(Debug)]
pub enum Environment {
    NoIO,
    Once(Option<Word>, Option<Word>),
    Collector(Option<Word>, Vec<Word>),
    ManyMany(VecDeque<Word>, Vec<Word>),
}

impl std::default::Default for Environment {
    fn default() -> Self {
        Self::NoIO
    }
}

impl Environment {
    fn input(&mut self) -> Result<Word, ProgramError> {
        let ret = match *self {
            Environment::NoIO => Err(ProgramError::NoMoreInput),
            Environment::Once(ref mut input, _)
                | Environment::Collector(ref mut input, _) => input
                .take()
                .ok_or(ProgramError::NoMoreInput),
            Environment::ManyMany(ref mut input, _) =>
                input.pop_front()
                    .ok_or(ProgramError::NoMoreInput),
        };
        ret
    }

    fn output(&mut self, value: Word) -> Result<(), ProgramError> {
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
            Environment::Collector(_, ref mut collected)
            | Environment::ManyMany(_, ref mut collected) => {
                collected.push(value);
                Ok(())
            }
        }
    }

    pub fn once(input: Option<Word>) -> Self {
        Self::Once(input, None)
    }

    pub fn collector(input: Option<Word>) -> Self {
        Self::Collector(input, Vec::new())
    }

    /// Inputs will be consumed with inputs.pop_front()
    pub fn collected_with_many_inputs(inputs: VecDeque<Word>) -> Self {
        Self::ManyMany(inputs, Vec::new())
    }

    pub fn unwrap_input_consumed_once(self) -> Option<Word> {
        match self.unwrap_once() {
            (None, x) => x,
            (Some(unconsumed), _) => unreachable!("Input {} was not consumed", unconsumed),
        }
    }

    pub fn unwrap_once(self) -> (Option<Word>, Option<Word>) {
        match self {
            Environment::Once(input, output) => (input, output),
            x => unreachable!("Was not once: {:?}", x),
        }
    }

    pub fn unwrap_collected(self) -> Vec<Word> {
        match self {
            Environment::Collector(input, collected) => {
                assert!(input.is_none());
                collected
            }
            Environment::ManyMany(inputs, collected) => {
                assert!(inputs.is_empty());
                collected
            }
            x => unreachable!("Was not collector: {:?}", x),
        }
    }
}

impl IO for Environment {
    fn input(&mut self) -> Result<Word, ProgramError> { self.input() }
    fn output(&mut self, value: Word) -> Result<(), ProgramError> { self.output(value) }
}
