use intcode::{Word, util::parse_stdin_program_n_lines};

fn main() {
    let data = parse_stdin_program_n_lines(Some(1));

    println!("stage1: {}", stage1(&data[..]));
    println!("stage2: {}", stage2(&data[..]));
}

fn stage1(data: &[Word]) -> Word {
    unimplemented!()
}

fn stage2(data: &[Word]) -> Word {
    unimplemented!()
}
