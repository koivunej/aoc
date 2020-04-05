// aoc2018 library or crate code

pub fn try_fold_stdin<F, St, E>(initial: St, mut inner: F) -> Result<St, E>
where
    F: for<'a> FnMut(&mut St, &'a str) -> Result<(), E>,
{
    use std::io::BufRead;

    let stdin = std::io::stdin();
    let mut locked = stdin.lock();
    let mut buffer = String::new();

    let mut state = initial;

    loop {
        buffer.clear();
        let bytes = locked
            .read_line(&mut buffer)
            .expect("Failed to read line from stdin");

        if bytes == 0 {
            break;
        }

        inner(&mut state, buffer.as_str())?;
    }

    return Ok(state);
}

pub fn process_stdin_lines<F>(mut inner: F)
where
    F: for<'a> FnMut(&'a str) -> (),
{
    use std::io::BufRead;

    let stdin = std::io::stdin();
    let mut locked = stdin.lock();
    let mut buffer = String::new();

    loop {
        buffer.clear();
        let bytes = locked
            .read_line(&mut buffer)
            .expect("Failed to read line from stdin");

        if bytes == 0 {
            break;
        }

        inner(buffer.as_str());
    }
}
