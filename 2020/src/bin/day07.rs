use indexmap::IndexSet;
use regex::Regex;
use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let g = parse(std::io::stdin().lock())?;

    let part_one = query_reachability("shiny gold", &g);
    let part_two = query_nesting("shiny gold", &g);

    println!("{}", part_one);
    println!("{}", part_two);

    assert_eq!(326, part_one);
    assert_ne!(2457, part_two);
    assert_eq!(5635, part_two);

    Ok(())
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct Bag {
    count: usize,
    material: usize,
}

struct Graph {
    intern: IndexSet<String>,
    forward: Vec<Vec<Bag>>,
    backward: Vec<Vec<usize>>,
}

fn parse<I: BufRead>(mut input: I) -> Result<Graph, Box<dyn std::error::Error + 'static>> {
    let top = Regex::new(r"^(\S+ \S+) bags contain (.+)$").unwrap();
    let contained = Regex::new(r"(?:, )?([0-9]+) (\S+ \S+) bags?").unwrap();

    let mut intern = IndexSet::new();
    let mut forward = Vec::new();
    let mut backward = Vec::new();

    let mut line = String::new();
    let mut bags = Vec::new();

    loop {
        line.clear();
        let read = input.read_line(&mut line)?;
        if read == 0 {
            break;
        }

        for top_cap in top.captures_iter(line.trim()) {
            let material = intern.insert_full(top_cap[1].to_string()).0;

            bags.extend(contained.captures_iter(&top_cap[2]).map(|cap| {
                let count = cap[1].parse::<usize>().expect("matched already");
                let material = intern.insert_full(cap[2].to_string()).0;
                Bag { count, material }
            }));

            while backward.len() < intern.len() {
                backward.push(Vec::new());
            }

            for bag in &bags {
                backward[bag.material].push(material);
            }

            while forward.len() < intern.len() {
                forward.push(Vec::new());
            }

            forward[material].extend(bags.drain(..));
        }
    }

    Ok(Graph {
        intern,
        forward,
        backward,
    })
}

fn query_reachability(s: &str, bags: &Graph) -> usize {
    let key = if let Some(idx) = bags.intern.get_index_of(s) {
        idx
    } else {
        return 0;
    };

    let mut work = std::collections::VecDeque::new();
    let mut visited = bitvec::bitvec![0; bags.intern.len()];

    work.extend(
        bags.backward[key]
            .as_slice()
            .iter()
            .copied()
            .map(|key| (key, 1)),
    );

    let mut count = 0;

    while let Some((material_idx, depth)) = work.pop_front() {
        if visited[material_idx] {
            continue;
        }
        visited.set(material_idx, true);

        work.extend(
            bags.backward[material_idx]
                .iter()
                .copied()
                .filter(|key| !visited[*key])
                .map(|key| (key, depth + 1)),
        );
        count += 1;
    }

    count
}

fn query_nesting(s: &str, bags: &Graph) -> usize {
    let key = if let Some(idx) = bags.intern.get_index_of(s) {
        idx
    } else {
        return 0;
    };

    let mut work = std::collections::VecDeque::new();

    // visited cannot be tracked as each of the bags needs to be constructed
    // let mut visited = HashSet::new();

    work.extend(
        bags.forward[key]
            .iter()
            .map(|bag| (bag.material, bag.count)),
    );

    // increment this when we have completed a bag
    let mut count = 0;

    while let Some((material_idx, this_bag_count)) = work.pop_front() {
        work.extend(
            bags.forward[material_idx]
                .iter()
                .map(|bag| (bag.material, this_bag_count * bag.count)),
        );
        count += this_bag_count;
    }

    count
}

#[test]
fn first_example() {
    use std::io::{BufReader, Cursor};
    let input = b"light red bags contain 1 bright white bag, 2 muted yellow bags.
dark orange bags contain 3 bright white bags, 4 muted yellow bags.
bright white bags contain 1 shiny gold bag.
muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
dark olive bags contain 3 faded blue bags, 4 dotted black bags.
vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
faded blue bags contain no other bags.
dotted black bags contain no other bags.
";

    let bags = parse(BufReader::new(Cursor::new(input))).unwrap();
    assert_eq!(query_reachability("shiny gold", &bags), 4);
}

#[test]
fn second_example() {
    use std::io::{BufReader, Cursor};
    let input = b"shiny gold bags contain 2 dark red bags.
dark red bags contain 2 dark orange bags.
dark orange bags contain 2 dark yellow bags.
dark yellow bags contain 2 dark green bags.
dark green bags contain 2 dark blue bags.
dark blue bags contain 2 dark violet bags.
dark violet bags contain no other bags.
";

    let bags = parse(BufReader::new(Cursor::new(input))).unwrap();
    assert_eq!(query_nesting("shiny gold", &bags), 126);
}
