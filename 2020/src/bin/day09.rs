use std::collections::{btree_map::Entry, BTreeMap, VecDeque};
use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut buffer = String::new();

    let mut part_one = None;
    let mut all = Vec::new();

    {
        let mut preample = VecDeque::with_capacity(25);
        let mut sorted_preample = BTreeMap::new();

        loop {
            buffer.clear();
            let read = stdin.read_line(&mut buffer)?;
            if read == 0 {
                break;
            }

            let num = buffer.trim().parse::<u64>().unwrap();

            if preample.len() != preample.capacity() {
                *sorted_preample.entry(num).or_insert(0) += 1;
                preample.push_back(num);
                continue;
            }

            // num needs to be a sum of two in the preample

            // seems natural that we would have less of the smaller than the larger
            // so maybe test these out then try to find the smallest pair?
            let valid = sorted_preample
                .keys()
                .enumerate()
                .rev()
                .filter(|&(_, &p)| p < num)
                // if preample was a bitvec or Vec<bool> this would be O(1)?
                .any(|(i, l)| sorted_preample.keys().take(i).any(|&s| s + l == num));

            if !valid {
                part_one = part_one.or(Some((all.len(), num)));
            }

            let oldest = preample.pop_front().unwrap();

            match sorted_preample.entry(oldest) {
                Entry::Vacant(_) => unreachable!("it is there"),
                Entry::Occupied(oe) if *oe.get() == 1 => {
                    oe.remove();
                }
                Entry::Occupied(mut oe) => {
                    *oe.get_mut() -= 1;
                }
            }

            *sorted_preample.entry(num).or_insert(0) += 1;
            preample.push_back(num);

            // for part_two
            all.push(num);
        }
    }

    let (part_one_index, part_one) = part_one.unwrap();
    println!("{}", part_one);

    // need to find a window of `all` which sums up to part_one
    let part_two = find_weakness((part_one_index, part_one), &all);
    println!("{}", part_two);

    assert_eq!(105950735, part_one);
    assert_eq!(13826915, part_two);

    Ok(())
}

fn find_weakness((part_one_index, part_one): (usize, u64), all: &[u64]) -> u64 {
    assert_eq!(all[part_one_index], part_one);
    let (before, after) = all.split_at(part_one_index);
    let (_, after) = after.split_at(1); // remove the part_one
    assert_ne!(after[0], part_one);

    // can only think of a bruteforce way of maintaining sums for all subsets
    // [i]
    // [i + j, j]
    // [i + j + k, j + k, k]
    // ...
    let mut sums = VecDeque::with_capacity(all.len() / 2);
    let mut sums_hwm = 0;
    for slice in &[before, after] {
        sums.clear();

        // eeech might need to dedup the all? or not? luckily did not.
        for (end, &i) in slice.iter().enumerate() {
            // avoid summing the i with itself
            sums.push_back((end, 0));

            let mut any_overflown = false;

            for (start, sum) in sums.iter_mut() {
                *sum += i;

                if *sum == part_one {
                    let mut target = all
                        .iter()
                        .skip(*start)
                        .take(end - *start)
                        .copied()
                        .collect::<Vec<_>>();
                    target.sort_unstable();
                    let (first, last) = (target[0], target[target.len() - 1]);
                    println!(
                        "found {} + {} = {} with {} sums (at most {})",
                        first,
                        last,
                        first + last,
                        sums.len(),
                        sums_hwm,
                    );
                    return first + last;
                }

                if !any_overflown && *sum > part_one {
                    any_overflown = true;
                }
            }

            while any_overflown {
                // remove such sums from the front which we no longer need to calculate
                sums_hwm = sums_hwm.max(sums.len());

                match sums.front() {
                    Some((_, over)) if *over > part_one => {
                        sums.pop_front();
                    }
                    _ => break,
                }
            }
        }
    }

    unreachable!("no weakness found");
}
