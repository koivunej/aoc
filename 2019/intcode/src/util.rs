use crate::Word;

#[derive(Debug)]
pub enum ParsingError {
    Io(std::io::Error, usize),
    Int(std::num::ParseIntError, usize, String),
}

pub fn parse_program<R: std::io::BufRead>(mut r: R) -> Result<Vec<Word>, ParsingError> {
    use std::str::FromStr;

    let mut data = vec![];
    let mut buffer = String::new();
    let mut line = 0;

    loop {
        buffer.clear();
        let bytes = r
            .read_line(&mut buffer)
            .map_err(|e| ParsingError::Io(e, line))?;

        if bytes == 0 {
            return Ok(data);
        }

        let parts = buffer.trim().split(',').map(Word::from_str);

        for part in parts {
            let part = match part {
                Ok(part) => part,
                Err(e) => return Err(ParsingError::Int(e, line, buffer)),
            };

            data.push(part);
        }

        line += 1;
    }
}

pub fn parse_stdin_program() -> Vec<Word> {
    let stdin = std::io::stdin();
    let locked = stdin.lock();

    match parse_program(locked) {
        Ok(data) => data,
        Err(ParsingError::Io(e, line)) => {
            eprintln!("Failed to read stdin near line {}: {}", line, e);
            std::process::exit(1);
        }
        Err(ParsingError::Int(e, line, raw)) => {
            eprintln!("Bad input at line {}: \"{}\" ({})", line, raw, e);
            std::process::exit(1);
        }
    }
}

/// Testing utility: parses "input" from current working directory as a program with
/// `parse_program`, unwrapping on error.
pub fn with_parsed_program<V, F>(f: F) -> V
    where F: FnOnce(&[Word]) -> V
{
    use std::io::BufReader;

    let file = std::fs::File::open("input").expect("Could not open day02 input?");

    let data = parse_program(BufReader::new(file)).unwrap();

    f(&data)
}
