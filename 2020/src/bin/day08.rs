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

    let part_two = find_termination(&visited, &code);

    let part_one = part_one.unwrap();

    println!("{}", part_one);
    println!("{}", part_two);

    assert_eq!(1766, part_one);
    assert_eq!(1639, part_two);
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

        execute(&code[*pc], pc, reg_acc);
    }
}

fn execute(code: &(Op, i64), pc: &mut usize, reg_acc: &mut i64) {
    let increment = match code {
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

fn find_termination(visited: &HashSet<usize>, code: &[(Op, i64)]) -> i64 {
    // need to find a single instruction to modify so that the pc will become code.len()
    // since the current corrupted version loops this has to be some negative jump following which
    // no instructions were executed...?

    let candidates = code
        .iter()
        .enumerate()
        .filter(|(idx, _)| visited.contains(idx))
        // not filtering on arg0 as we might need to find an earlier positive jump
        .filter(|(_, (op, _arg0))| *op == Op::Jump)
        .map(|(idx, _)| idx)
        .collect::<Vec<_>>();

    let mut code = code.to_vec();

    let mut visited = HashSet::new();

    for candidate in candidates {
        // was kind of hoping that these could had been kept outside the for loop
        // but failed somehow, or then it just can't be done.
        let mut pc = 0;
        let mut reg_acc = 0;

        visited.clear();

        while pc != candidate {
            assert!(visited.insert(pc));
            execute(&code[pc], &mut pc, &mut reg_acc);
        }

        assert_eq!(pc, candidate);

        // just for printing
        let replace_at = pc;

        // keep around to be restored
        let old = code[pc];

        // temporary replacement
        code[pc] = (Op::Nop, 0);

        let mut count = 0;

        {
            let mut pc = pc;
            let mut reg_acc = reg_acc;
            let mut visited = visited.clone();

            while visited.insert(pc) && pc != code.len() {
                execute(&code[pc], &mut pc, &mut reg_acc);
                count += 1;
            }

            if pc == code.len() {
                println!(
                    "found with replacement at {}, executed {} more",
                    replace_at, count
                );
                return reg_acc;
            }
        }

        // replace back
        code[pc] = old;

        println!(
            "did not find with replacement at {}, executed {} more",
            candidate, count
        );
    }

    unreachable!("could not find a replacement")
}
