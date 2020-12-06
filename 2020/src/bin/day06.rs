use std::collections::HashSet;
use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut buffer = String::new();

    let mut questions_anyone_answered_yes: HashSet<u8> = HashSet::new();
    let mut questions_everyone_answered_yes: HashSet<u8> = HashSet::new();
    let mut current_person_answers = HashSet::new();

    let mut part_one = 0;
    let mut part_two = 0;

    let mut group_persons = 0;

    loop {
        buffer.clear();

        let read = stdin.read_line(&mut buffer)?;

        if buffer.trim().is_empty() {
            // process latest group
            part_one += questions_anyone_answered_yes.len();
            questions_anyone_answered_yes.clear();

            part_two += questions_everyone_answered_yes.len();
            questions_everyone_answered_yes.clear();

            group_persons = 0;

            if read == 0 {
                break;
            } else {
                continue;
            }
        }

        group_persons += 1;

        // process one persons answers
        current_person_answers.extend(buffer.trim().as_bytes().iter().copied());

        if group_persons == 1 {
            questions_everyone_answered_yes.extend(current_person_answers.iter().copied());
        } else {
            questions_everyone_answered_yes.retain(|b| current_person_answers.contains(b));
        }

        questions_anyone_answered_yes.extend(current_person_answers.drain());
    }

    println!("{}", part_one);
    println!("{}", part_two);
    assert_ne!(26, part_one);
    assert_eq!(6726, part_one);
    assert_eq!(3316, part_two);

    Ok(())
}
