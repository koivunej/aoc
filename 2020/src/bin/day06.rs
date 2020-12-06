use std::collections::{HashMap, HashSet};
use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut buffer = String::new();

    let mut questions_anyone_answered_yes = HashSet::new();
    let mut questions_everyone_answered_yes = HashMap::new();

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

            let yes_answers_in_group = questions_everyone_answered_yes
                .drain()
                .filter(|&(_, v)| v == group_persons)
                .count();

            part_two += yes_answers_in_group;

            group_persons = 0;

            if read == 0 {
                break;
            } else {
                continue;
            }
        }

        group_persons += 1;

        // process one persons answers
        buffer.trim().chars().for_each(|ch| {
            questions_anyone_answered_yes.insert(ch);

            *questions_everyone_answered_yes.entry(ch).or_insert(0) += 1;
        });
    }

    println!("{}", part_one);
    println!("{}", part_two);
    assert_ne!(26, part_one);
    assert_eq!(6726, part_one);
    assert_eq!(3316, part_two);

    Ok(())
}
