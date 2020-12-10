//use std::collections::BTreeMap;
use std::io::BufRead;

/*struct Adapter {
    index: usize,
    rating: u8,
}*/

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    // let mut adapters: Vec<Adapter> = Vec::new();
    // let mut all: BTreeMap<u8, Vec<usize>> = BTreeMap::new();
    let mut ratings = Vec::new();
    {
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();

        let mut buffer = String::new();

        loop {
            buffer.clear();

            let read = stdin.read_line(&mut buffer)?;
            if read == 0 {
                break;
            }

            let rating = buffer.trim().parse::<u8>()?;
            /*let index = adapters.len();
            adapters.push(Adapter { index, rating });
            all.entry(rating).or_default().push(index);*/
            ratings.push(rating);
        }
    }

    /*
    let mut chain = Vec::new();

    let mut rating = 0;

    while !all.is_empty() {
        let lowest = *all.iter().next().expect("not empty yet").0;
        println!("fitting {} on top of chain with rating {}", lowest, rating);

        let adapter_ids = all.get_mut(&lowest).unwrap();


    }
    */

    ratings.sort_unstable();

    let diffs = ratings
        .windows(2)
        .map(|slice| slice[1] - slice[0])
        .fold((0, 0), |acc, next| match next {
            1 => (acc.0 + 1, acc.1),
            3 => (acc.0, acc.1 + 1),
            x => unreachable!("unexpected difference between ratings: {}", x),
        });

    /*
    let mut last = 0;

    let mut diffs = (0, 0);

    for r in ratings {
        let diff = r - last;
        match diff {
            0 => {}
            1 => diffs.0 += 1,
            3 => diffs.1 += 1,
            x => unreachable!("{}", x),
        }
        println!("{}\t{}", r, diff);
        last = r;
    }*/

    let mut diffs = diffs;
    diffs.0 += 1; // zero from the initial outlet
    diffs.1 += 1; // our devices internal charger

    let part_one = diffs.0 * diffs.1;

    println!("{}", part_one);
    assert_ne!(part_one, 1952);
    assert_ne!(part_one, 1984);
    assert_ne!(part_one, 2170);
    assert_eq!(part_one, 2046);

    Ok(())
}

#[test]
fn short_example_part_one() {
    let input = "16
10
15
5
1
11
7
19
6
12
4";

    let mut input = input
        .lines()
        .map(|s| s.parse::<u8>().unwrap())
        .collect::<Vec<_>>();

    input.sort_unstable();

    let diffs = input
        .windows(2)
        .inspect(|slice| println!("{:?}", slice))
        .map(|slice| slice[1] - slice[0])
        .fold((0, 0), |acc, next| match next {
            1 => (acc.0 + 1, acc.1),
            3 => (acc.0, acc.1 + 1),
            x => unreachable!("unexpected difference between ratings: {}", x),
        });

    assert_eq!(diffs.0 + 1, 7);
    assert_eq!(diffs.1 + 1, 5);
}

#[test]
fn long_example_part_one() {
    let input = "28
33
18
42
31
14
46
20
48
47
24
23
49
45
19
38
39
11
1
32
25
35
8
17
7
9
4
2
34
10
3";
    let mut input = input
        .lines()
        .map(|s| s.parse::<u8>().unwrap())
        .collect::<Vec<_>>();

    input.sort_unstable();

    let diffs = input
        .windows(2)
        .inspect(|slice| println!("{:?}", slice))
        .map(|slice| slice[1] - slice[0])
        .fold((0, 0), |acc, next| match next {
            1 => (acc.0 + 1, acc.1),
            3 => (acc.0, acc.1 + 1),
            x => unreachable!("unexpected difference between ratings: {}", x),
        });

    assert_eq!(diffs.0 + 1, 22);
    assert_eq!(diffs.1 + 1, 10);
}
