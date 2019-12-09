use smallvec::SmallVec;
use std::convert::TryFrom;
use crate::{DecodingError, DecodedOperation, Params, Param, BadWrite, Word};
use crate::error::InvalidReadAddress;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum OpCode {
    BinOp(BinOp),
    Store,
    Print,
    Jump(UnaryCondition),
    StoreCompared(BinaryCondition),
    AdjustRelative,
    Halt,
}

impl OpCode {
    fn parameters(&self) -> usize {
        match *self {
            OpCode::BinOp(_) => 3,
            OpCode::Store => 1,
            OpCode::Print => 1,
            OpCode::Jump(_) => 2,
            OpCode::StoreCompared(_) => 2,
            OpCode::AdjustRelative => 1,
            OpCode::Halt => 0,
        }
    }

    fn only_default_parameters(&self) -> bool {
        if let OpCode::Store = *self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum UnaryCondition {
    OnTrue,
    OnFalse,
}

impl UnaryCondition {
    pub(crate) fn eval(&self, first: Word) -> bool {
        match *self {
            Self::OnTrue => first != 0,
            Self::OnFalse => first == 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum BinaryCondition {
    OnLessThan,
    OnEq,
}

impl BinaryCondition {
    pub(crate) fn eval(&self, first: Word, second: Word) -> bool {
        match *self {
            Self::OnLessThan => first < second,
            Self::OnEq => first == second,
        }
    }
}


impl TryFrom<Word> for OpCode {
    type Error = DecodingError;
    fn try_from(u: Word) -> Result<Self, Self::Error> {
        Ok(match u % 100 {
            1 => OpCode::BinOp(BinOp::Add),
            2 => OpCode::BinOp(BinOp::Mul),
            3 => OpCode::Store,
            4 => OpCode::Print,
            5 => OpCode::Jump(UnaryCondition::OnTrue),
            6 => OpCode::Jump(UnaryCondition::OnFalse),
            7 => OpCode::StoreCompared(BinaryCondition::OnLessThan),
            8 => OpCode::StoreCompared(BinaryCondition::OnEq),
            9 => OpCode::AdjustRelative,
            99 => OpCode::Halt,
            x => {
                return Err(DecodingError::UnknownOpCode(x));
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct Operation(OpCode, ParameterModes);

impl DecodedOperation for Operation {
    type Parameters = ParameterModes;

    fn unpack(self) -> (OpCode, Self::Parameters) {
        (self.0, self.1)
    }

    fn default_parameters(&self) -> bool { self.1.is_default() }
}

impl TryFrom<Word> for Operation {
    type Error = DecodingError;

    fn try_from(raw: Word) -> Result<Self, Self::Error> {
        if raw < 0 {
            return Err(DecodingError::UnknownOpCode(raw));
        }

        let op = OpCode::try_from(raw)?;
        let mut pvs = ParameterModes::try_from(raw)?
            .at_most(op.parameters())
            .map_err(|_| DecodingError::TooManyParameters(raw))?;

        if op.only_default_parameters() {
            pvs = pvs
                .all_must_equal_default()
                .map_err(|_| DecodingError::InvalidParameterMode(raw))?;
        }

        Ok(Operation(op, pvs))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ParameterModes {
    modes: SmallVec<[ParameterMode; 4]>,
}

impl TryFrom<Word> for ParameterModes {
    type Error = DecodingError;

    fn try_from(raw: Word) -> Result<ParameterModes, Self::Error> {
        if !Self::instruction_has_modes(raw) {
            Ok(ParameterModes {
                modes: SmallVec::new(),
            })
        } else if raw > 0 {
            let mut shifted = raw / 100;
            let mut pm = ParameterModes {
                modes: SmallVec::new(),
            };
            while shifted > 0 {
                pm.modes.push(match shifted % 10 {
                    1 => ParameterMode::Immediate,
                    0 => ParameterMode::Address,
                    2 => ParameterMode::Relative,
                    x => return Err(DecodingError::InvalidParameterMode(x)),
                });

                shifted /= 10;
            }
            Ok(pm)
        } else {
            unreachable!("Negative values must be handled before calling this method");
        }
    }
}

impl Params for ParameterModes {
    type Parameter = ParameterMode;

    fn mode(&self, index: usize) -> &Self::Parameter {
        self.mode(index)
    }
}

static DEFAULT_PARAMETER_MODE: ParameterMode = ParameterMode::Address;

impl ParameterModes {
    pub(crate) fn mode(&self, index: usize) -> &ParameterMode {
        self.modes.get(index).unwrap_or(&DEFAULT_PARAMETER_MODE)
    }

    pub(crate) fn is_default(&self) -> bool {
        self.modes.is_empty() || self.modes.iter().all(|pm| pm == &DEFAULT_PARAMETER_MODE)
    }

    fn instruction_has_modes(raw: Word) -> bool {
        raw > 100
    }

    fn all_must_equal_default(self) -> Result<Self, ()> {
        if self.modes.is_empty() || self.modes.iter().all(|pm| pm == &DEFAULT_PARAMETER_MODE) {
            Ok(self)
        } else {
            Err(())
        }
    }

    fn at_most(mut self, count: usize) -> Result<Self, ()> {
        use std::iter::repeat;
        if self.modes.len() <= count {
            self.modes
                .extend(repeat(DEFAULT_PARAMETER_MODE).take(count - self.modes.len()));
            Ok(self)
        } else {
            Err(())
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum ParameterMode {
    Address,
    Immediate,
    Relative,
}

impl ParameterMode {
    pub(crate) fn read(self, arg: Word, relbase: Word, program: &[Word]) -> Result<Word, InvalidReadAddress> {
        match self {
            ParameterMode::Address => Self::read_at(arg, program),
            ParameterMode::Immediate => Ok(arg),
            ParameterMode::Relative => Self::read_at(arg + relbase, program),
        }
    }

    fn read_at(addr: Word, program: &[Word]) -> Result<Word, InvalidReadAddress> {
        if addr < 0 {
            return Err(InvalidReadAddress(addr));
        }
        program.get(addr as usize)
            .cloned()
            .ok_or(InvalidReadAddress(addr))
    }

    pub(crate) fn write(self, value: Word, arg: Word, relbase: Word, program: &mut [Word]) -> Result<(), BadWrite> {
        use BadWrite::*;
        match self {
            ParameterMode::Address => Self::write_at(value, arg, program),
            ParameterMode::Immediate => Err(ImmediateParameter),
            ParameterMode::Relative => Self::write_at(value, arg + relbase, program),
        }
    }

    fn write_at(value: Word, addr: Word, program: &mut [Word]) -> Result<(), BadWrite> {
        use BadWrite::*;
        if addr < 0 {
            return Err(AddressOutOfBounds);
        }
        let cell = program.get_mut(addr as usize).ok_or(AddressOutOfBounds)?;
        *cell = value;
        Ok(())
    }
}

impl Param for ParameterMode {
    fn read(self, arg: Word, relbase: Word, memory: &[Word]) -> Result<Word, InvalidReadAddress> { self.read(arg, relbase, memory) }
    fn write(self, value: Word, arg: Word, relbase: Word, memory: &mut [Word]) -> Result<(), BadWrite> { self.write(value, arg, relbase, memory) }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinOp {
    Add,
    Mul,
}

impl BinOp {
    pub(crate) fn eval(&self, lhs: Word, rhs: Word) -> Word {
        match *self {
            BinOp::Add => lhs.checked_add(rhs).expect("Add overflow"),
            BinOp::Mul => lhs.checked_mul(rhs).expect("Mul overflow"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use super::{BinOp, OpCode, ParameterMode, ParameterModes};

    #[test]
    fn parse_opcode_with_modes() {
        let input = 1002;

        assert_eq!(OpCode::try_from(input).unwrap(), OpCode::BinOp(BinOp::Mul));

        let pm = ParameterModes::try_from(input).unwrap();
        assert_eq!(pm.mode(0), &ParameterMode::Address);
        assert_eq!(pm.mode(1), &ParameterMode::Immediate);
        assert_eq!(pm.mode(2), &ParameterMode::Address);
    }

    #[test]
    fn relative_mode() {
        let input = 204;

        assert_eq!(OpCode::try_from(input).unwrap(), OpCode::Print);
        let pm = ParameterModes::try_from(input).unwrap();
        assert_eq!(pm.mode(0), &ParameterMode::Relative);
    }

    #[test]
    fn relative_adjust_op() {
        let input = 9;

        assert_eq!(OpCode::try_from(input).unwrap(), OpCode::AdjustRelative);
    }
}
