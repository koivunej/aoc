fn main() {
    let input = [0, 13, 1, 16, 6, 17];

    let part_one = process(&input[..], 2020);
    println!("{}", part_one);

    let part_two = process(&input[..], 30_000_000);
    println!("{}", part_two);

    assert_ne!(part_one, 7);
    assert_eq!(part_one, 234);
}
fn process(starting_numbers: &[u32], nth: u32) -> u32 {
    //use std::collections::{btree_map::Entry, BTreeMap as Collection};
    use std::collections::{hash_map::Entry, HashMap as Collection};
    let mut last_turn = Collection::new();

    starting_numbers
        .iter()
        .enumerate()
        .for_each(|(i, s)| assert_eq!(last_turn.insert(*s, i as u32 + 1), None));

    let mut last = starting_numbers[starting_numbers.len() - 1];
    for turn in ((starting_numbers.len() + 1) as u32).. {
        // print!("{} following {} => ", turn, last);
        match last_turn.entry(last) {
            Entry::Vacant(ve) => {
                // last starting number was the first time it was spoken
                ve.insert(turn - 1);
                last = 0;
            }
            Entry::Occupied(mut oe) => {
                let slot = oe.get_mut();
                let when = *slot;
                *slot = turn - 1;

                last = turn - 1 - when;
            }
        }
        //println!("{}", last);
        if turn as u32 == nth {
            return last;
        }
    }

    todo!();
}

#[test]
fn part_one_examples() {
    let examples = [
        ([0, 3, 6], 2020, 436),
        ([1, 3, 2], 2020, 1),
        ([2, 1, 3], 2020, 10),
        ([1, 2, 3], 2020, 27),
        ([2, 3, 1], 2020, 78),
        ([3, 2, 1], 2020, 438),
        ([3, 1, 2], 2020, 1836),
    ];

    for (starting_numbers, nth, expected) in &examples {
        assert_eq!(process(&starting_numbers[..], *nth), *expected);
    }
}
