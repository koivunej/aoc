use itertools::Itertools;
use std::collections::BTreeSet;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let input = parse_stdin()?;

    // original best idea at the time was a bit different, but then I thought it might be a good
    // idea to focus on learning more about itertools this time aorund.
    //
    // part_one originally was a clone vec of input, sort it, binary search for the needed.
    // thought that maybe it's easier to just use BTreeSet but perhaps sorting it would be faster,
    // but I was expecting part_two to require sorted input as well.

    let part_one = {
        let part_one = input
            .iter()
            .filter_map(|&small| {
                if let Some(gt) = 2020u16.checked_sub(small) {
                    if input.contains(&gt) {
                        Some((small, gt))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .map(|(a, b)| a as u32 * b as u32)
            .next();

        // assert_eq!(Some((896, 1124)), part_one);
        part_one.unwrap()
    };

    println!("{}", part_one);

    // originally just nested for-loops

    let part_two = {
        let part_two = input
            .iter()
            .cartesian_product(input.iter())
            .cartesian_product(input.iter())
            .filter_map(|((a, b), c)| {
                if a + b + c == 2020 {
                    Some((*a, *b, *c))
                } else {
                    None
                }
            })
            .map(|(a, b, c)| a as u32 * b as u32 * c as u32)
            .next();

        // assert_eq!(Some((24, 539, 1457)), part_two);
        part_two.unwrap()
    };

    println!("{}", part_two);

    assert_eq!(part_one, 1_007_104);
    assert_eq!(part_two, 18_847_752);

    Ok(())
}

fn parse_stdin() -> Result<BTreeSet<u16>, Box<dyn std::error::Error + 'static>> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut buf = String::new();
    let mut ret = BTreeSet::new();
    loop {
        buf.clear();
        let read = stdin.read_line(&mut buf)?;
        if read == 0 {
            break;
        }
        let num = buf.trim().parse::<u16>()?;
        ret.insert(num);
    }

    Ok(ret)
}
