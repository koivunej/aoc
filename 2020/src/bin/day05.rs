fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut buffer = String::new();
    let mut part_one = None;
    let mut all = vec![];

    loop {
        buffer.clear();
        let read = stdin.read_line(&mut buffer)?;
        if read == 0 {
            break;
        }

        let id = find_seat_id(buffer.trim().as_bytes());

        part_one = part_one.max(Some(id));

        // collect in order to sort, and find the missing one in between
        all.push(id);
    }

    all.sort();

    let missing_one_in_between = all
        .windows(2)
        .filter_map(|w| {
            let earlier = w[0];
            let later = w[1];
            assert!(earlier < later);
            if later - earlier == 2 {
                Some(later - 1)
            } else {
                None
            }
        })
        .fold(None, |acc, next| {
            if acc.is_none() {
                Some(next)
            } else {
                panic!("multiple results: {} and {}", acc.unwrap(), next);
            }
        });

    let part_one = part_one.unwrap();
    let part_two = missing_one_in_between.unwrap();
    println!("{}", part_one);
    println!("{}", part_two);

    assert_eq!(892, part_one);
    assert_eq!(625, part_two);

    Ok(())
}

#[cfg(feature = "nightly")]
#[no_mangle]
fn find_seat_id(ins: &[u8]) -> u16 {
    use packed_simd::u8x16;

    // 0b01000010
    //            => 1
    // 0b01010010
    //
    // 0b01000110
    //            => 0
    // 0b01001100
    //
    // mask idea?
    // 0b00000100 => 1st => 0, 2 => 0, 3 => 0100, 4 => 0100
    assert_eq!(ins.len(), 10);

    let wz = 4; // we want these to be zeroes after negation

    let all = u8x16::new(
        ins[9], ins[8], ins[7], ins[6], ins[5], ins[4], ins[3], ins[2], ins[1], ins[0], wz, wz, wz,
        wz, wz, wz,
    );
    let next = all & wz;

    // now all B and R should be zeroes, F and B (and wz) will have 0x04

    // now shift all 0x04 to become the highest bit; lowest might be set as well
    let next = next << 5;

    // bitmask of highest should be the negation
    let bitmask = next.bitmask();
    let id = !bitmask;

    // this at least produces the right results but probably isn't much faster because de-inlining
    // view asm with:
    //
    // RUSTFLAGS="-C target-cpu=native" cargo +nightly rustc --release --bin day05 --features nightly -- --emit asm
    //
    // find the .s file with: find target/release -name '*.s'
    //
    //find_seat_id:
    //        .cfi_startproc
    //        subq    $104, %rsp
    //        .cfi_def_cfa_offset 112
    //        movq    %rsi, (%rsp)
    //        cmpq    $10, %rsi
    //        jne     .LBB19_1
    //        movzbl  9(%rdi), %eax
    //        vmovd   %eax, %xmm0
    //        vpinsrb $1, 8(%rdi), %xmm0, %xmm0
    //        movl    $4, %eax
    //        vpinsrb $2, 7(%rdi), %xmm0, %xmm0
    //        vpinsrb $3, 6(%rdi), %xmm0, %xmm0
    //        vpinsrb $4, 5(%rdi), %xmm0, %xmm0
    //        vpinsrb $5, 4(%rdi), %xmm0, %xmm0
    //        vpinsrb $6, 3(%rdi), %xmm0, %xmm0
    //        vpinsrb $7, 2(%rdi), %xmm0, %xmm0
    //        vpinsrb $8, 1(%rdi), %xmm0, %xmm0
    //        vpinsrb $9, (%rdi), %xmm0, %xmm0
    //        vpinsrb $10, %eax, %xmm0, %xmm0
    //        vpinsrb $11, %eax, %xmm0, %xmm0
    //        vpinsrb $12, %eax, %xmm0, %xmm0
    //        vpinsrb $13, %eax, %xmm0, %xmm0
    //        vpinsrb $14, %eax, %xmm0, %xmm0
    //        vpinsrb $15, %eax, %xmm0, %xmm0
    //        vpsllw  $5, %xmm0, %xmm0
    //        vpmovmskb       %xmm0, %eax
    //        notl    %eax
    //        addq    $104, %rsp
    //        .cfi_def_cfa_offset 8
    //        retq

    id
}

#[cfg(feature = "nightly")]
#[cfg(test)]
fn find_seat(ins: &[u8]) -> (u8, u8) {
    let id = find_seat_id(ins);
    ((id >> 3) as u8, (id & (1 + 2 + 4)) as u8)
}

#[cfg(not(feature = "nightly"))]
fn find_seat_id(ins: &[u8]) -> u16 {
    let pos = find_seat(ins);
    (pos.0 as u16) << 3 | pos.1 as u16
}

#[cfg(not(feature = "nightly"))]
fn find_seat(ins: &[u8]) -> (u8, u8) {
    let mut main = ins.iter();

    // just a binary number encoded with FB and LR for 01
    // rewriting to use u8::from_str_radix would probably take too long
    //
    // >>> bin(ord('B'))
    // '0b0100 0010'
    // >>> bin(ord('R'))
    // '0b0101 0010'

    let row = main
        .by_ref()
        .copied()
        .take(7)
        .enumerate()
        .inspect(|&(_, ch)| assert!(ch == b'F' || ch == b'B'))
        .map(|(idx, ch)| if ch == b'B' { 1 << (7 - idx - 1) } else { 0 })
        .fold(0u8, |acc, next| acc | next);

    let seat = main
        .by_ref()
        .copied()
        .take(3)
        .enumerate()
        .inspect(|&(_, ch)| assert!(ch == b'L' || ch == b'R'))
        .map(|(idx, ch)| if ch == b'R' { 1 << (3 - idx - 1) } else { 0 })
        .fold(0u8, |acc, next| acc | next);

    assert_eq!(main.next(), None);

    (row, seat)
}

#[test]
fn seat_finding_example() {
    let ins = b"FBFBBFFRLR";
    assert_eq!(find_seat(ins), (44, 5));
}

#[test]
fn seat_id_examples() {
    let examples = [
        (b"BFFFBBFRRR", 70, 7, 567),
        (b"FFFBBBFRRR", 14, 7, 119),
        (b"BBFFBBFRLL", 102, 4, 820),
    ];

    for (example, row, column, id) in &examples {
        let pos = find_seat(&example[..]);
        assert_eq!((*row, *column), pos);
        assert_eq!(*id, (pos.0 as u16) << 3 | pos.1 as u16);
    }
}
