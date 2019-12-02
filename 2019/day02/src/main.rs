use std::convert::TryFrom;
use std::io::BufRead;
use std::str::FromStr;

fn main() {
    let stdin = std::io::stdin();
    let locked = stdin.lock();

    let mut data = vec![];

    for (line_num, line) in locked.lines().enumerate() {
        match line {
            Ok(line) => {
                for part in line.split(',') {
                    match isize::from_str(part) {
                        Ok(val) => data.push(val),
                        Err(e) => {
                            eprintln!("Bad input at line {}: \"{}\" ({})", line_num, line, e);
                            std::process::exit(1);
                        }
                    }

                }
            },
            Err(e) => {
                eprintln!("Failed to read stdin near line {}: {}", line_num, e);
                std::process::exit(1);
            },
        }
    }

    // restore
    data[1] = 12;
    data[2] = 2;

    Program::wrap_and_eval(&mut data);

    println!("Value at position 0: {}", data[0]);
}

enum OpCode {
    BinOp(BinOp),
    Halt,
}

#[derive(Debug)]
enum BinOp {
    Add,
    Mul,
}

impl BinOp {
    fn eval(&self, lhs: isize, rhs: isize) -> isize {
        eprintln!("BinOp::{:?}({}, {})", self, lhs, rhs);
        match *self {
            BinOp::Add => lhs.checked_add(rhs).expect("Add overflow"),
            BinOp::Mul => lhs.checked_mul(rhs).expect("Mul overflow"),
        }
    }
}

struct UnknownOpCode(isize);

impl TryFrom<isize> for OpCode {
    type Error = UnknownOpCode;
    fn try_from(u: isize) -> Result<Self, Self::Error> {
        Ok(match u {
            1 => OpCode::BinOp(BinOp::Add),
            2 => OpCode::BinOp(BinOp::Mul),
            99 => OpCode::Halt,
            x => { return Err(UnknownOpCode(x)); },
        })
    }
}

struct Program<'a> {
    prog: &'a mut [isize],
}

impl<'a> Program<'a> {
    fn indirect_value(&self, index: usize) -> isize {
        let index = index % self.prog.len();
        let addr = self.prog[index];
        assert!(addr >= 0);
        let val = self.prog[addr as usize];
        eprintln!("indirect_value({}) -> prog[{}] == {}", index, addr, val);
        val
    }

    fn indirect_store(&mut self, index: usize, value: isize) {
        let index = index % self.prog.len();
        let addr = self.prog[index];
        assert!(addr >= 0);
        eprintln!("indirect_store({}, {}) -> prog[{}] = {}", index, value, addr, value);
        // probably shouldn't panic?
        self.prog[addr as usize] = value;
    }

    fn eval(&mut self) {
        let mut index = 0;
        loop {
            match OpCode::try_from(self.prog[index]) {
                Ok(OpCode::Halt) => return,
                Ok(OpCode::BinOp(b)) => {

                    let res = b.eval(
                        self.indirect_value(index + 1),
                        self.indirect_value(index + 2));

                    self.indirect_store(index + 3, res);
                },
                Err(_) => {
                    panic!("Invalid opcode at {}: {}", index, self.prog[index]);
                }
            }

            index = (index + 4) % self.prog.len();
        }
    }

    fn wrap_and_eval(data: &mut [isize]) {
        let mut p = Program { prog: data };
        p.eval();
    }
}


#[test]
fn stage1_example() {
    let mut prog = vec![
        1, 9, 10, 3,
        2, 3, 11, 0,
        99,
        30, 40, 50];

    let expected = &[
        3500isize, 9, 10, 70,
        2, 3, 11, 0,
        99,
        30, 40, 50];

    Program::wrap_and_eval(&mut prog);

    assert_eq!(&prog[..], expected);
}
