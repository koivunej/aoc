fn main() {
    let numbers: Vec<i64> = parse_from_stdin().unwrap();

    let part1 = numbers.iter().sum::<i64>();
    println!("part1: {}", part1);

    assert_eq!(part1, 406);
}

fn parse_from_stdin() -> Result<Vec<i64>, std::io::Error> {
    use std::io::BufRead;

    let stdin = std::io::stdin();
    let mut lock = stdin.lock();
    let mut buffer = String::new();

    let mut vec = Vec::new();

    loop {
        buffer.clear();
        let bytes = lock.read_line(&mut buffer)?;

        if bytes == 0 {
            break;
        }

        let num = buffer
            .trim()
            .parse::<i64>()
            .unwrap_or_else(|e| panic!("failed to parse ({}): {:?}", e, buffer));

        vec.push(num);
    }

    Ok(vec)
}
