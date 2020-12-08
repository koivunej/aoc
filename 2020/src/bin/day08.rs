use std::collections::HashSet;
use std::convert::TryInto;
use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut buffer = String::new();
    let mut part_one = None;

    let mut code = vec![];
    let mut pc = 0;
    let mut reg_acc = 0;
    let mut visited = HashSet::new();

    loop {
        buffer.clear();
        let read = stdin.read_line(&mut buffer)?;

        if read == 0 {
            break;
        }

        let buffer = buffer.trim();

        let mut split = buffer.split_whitespace();
        let op = split
            .next()
            .map(|op| {
                Ok(match op {
                    "nop" => Op::Nop,
                    "acc" => Op::Acc,
                    "jmp" => Op::Jump,
                    x => return Err(x.to_string()),
                })
            })
            .unwrap();
        let arg0 = split.next().map(|s| s.parse::<i64>()).unwrap();
        assert_eq!(split.next(), None);

        code.push((op?, arg0?));

        interpret_ahead(
            &mut part_one,
            &mut visited,
            code.as_slice(),
            &mut pc,
            &mut reg_acc,
        );
    }

    interpret_ahead(
        &mut part_one,
        &mut visited,
        code.as_slice(),
        &mut pc,
        &mut reg_acc,
    );

    if part_one.is_none() {
        assert!(pc < code.len(), "pc out of range: {:?}", 0..code.len());

        panic!("not sure why no result");
    }

    println!("{}", part_one.unwrap());
    Ok(())
}

enum Op {
    Nop,
    Acc,
    Jump,
}

fn interpret_ahead(
    part_one: &mut Option<i64>,
    visited: &mut HashSet<usize>,
    code: &[(Op, i64)],
    pc: &mut usize,
    reg_acc: &mut i64,
) {
    while part_one.is_none() && code.len() > *pc {
        if !visited.insert(*pc) {
            *part_one = Some(*reg_acc);
            break;
        }

        let increment = match code[*pc] {
            (Op::Nop, _) => 1,
            (Op::Acc, x) => {
                *reg_acc += x;
                1
            }
            (Op::Jump, x) => {
                *pc = (*pc as i64 + x).try_into().expect("overflow");
                0
            }
        };

        *pc += increment;
    }
}
