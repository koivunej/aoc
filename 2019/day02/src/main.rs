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

    // stage1
    {
        let mut data = Vec::clone(&data);

        // restore
        data[1] = 12;
        data[2] = 2;

        Program::wrap_and_eval(&mut data);

        println!("Value at position 0: {}", data[0]);
    }

    {
        let magic = 19690720;

        if let Some((i, j)) = find_coords(&data[..], magic) {
            println!("Found it at {:?}: 100 * noun + verb == {}", (i, j), 100 * i + j);
        } else {
            println!("Did not find...");
        }
    }

}

fn find_coords(input: &[isize], magic: isize) -> Option<(isize, isize)> {

    let mut copy = input.to_vec();

    for i in 0..100 {
        for j in 0..100 {
            copy[1] = i;
            copy[2] = j;

            Program::wrap_and_eval(&mut copy);

            if copy[0] == magic {
                return Some((i, j));
            }

            copy.copy_from_slice(&input[..]);
        }
    }

    None
}

enum OpCode {
    BinOp(BinOp),
    Halt,
}

impl OpCode {
    fn len(&self) -> usize {
        match *self {
            OpCode::BinOp(_) => 4,
            OpCode::Halt => 1,
        }
    }
}

#[derive(Debug)]
enum BinOp {
    Add,
    Mul,
}

impl BinOp {
    fn eval(&self, lhs: isize, rhs: isize) -> isize {
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
        val
    }

    fn indirect_store(&mut self, index: usize, value: isize) {
        let index = index % self.prog.len();
        let addr = self.prog[index];
        assert!(addr >= 0);
        // probably shouldn't panic?
        self.prog[addr as usize] = value;
    }

    fn eval(&mut self) {
        let mut ip = 0;
        loop {
            let skipped = match OpCode::try_from(self.prog[ip]) {
                Ok(OpCode::Halt) => return,
                Ok(OpCode::BinOp(b)) => {

                    let res = b.eval(
                        self.indirect_value(ip + 1),
                        self.indirect_value(ip + 2));

                    self.indirect_store(ip + 3, res);

                    OpCode::BinOp(b).len()
                },
                Err(_) => {
                    panic!("Invalid opcode at {}: {}", ip, self.prog[ip]);
                }
            };

            ip = (ip + skipped) % self.prog.len();
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
