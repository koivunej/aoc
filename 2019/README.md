# AOC2019

In my 2019 solutions I'll try to look for Rust things I haven't used before, or
just simple code if the day is challenging.

Day1: I had never used the stabilized `std::convert::TryFrom` before

Day2: Nothing special, tried out `go`... Got my source code file overwritten by
the compiler.

Day3: Iterator training, deep error propagation, generic `Point<T>`... Suprised
I passed in the end.

Day4: Failed to try out anything, there were no complications. Possible
optimizations yes but I am not interested in those.

Day5: Crate extraction practice. Using types and whatnot paying off with
`intcode`, succeeded at first submissions. Possibly need to extract another
crate as currently the "read 'input' file and test with it" code is copypasted
for `day02` and `day05`.

Went way too far with this and prototyped a `tracing` feature which would allow
another type to wrap the traits at top-level of `intcode` ... Turned out not so
pretty but still a lot of learning.

Day6: Used `petgraph` which turned out quite difficult. Hard to understand why
you can't take a directed graph and view it as undirected? Also needed to
rebase [an older PR](https://github.com/petgraph/petgraph/pull/151) to get the
transitive closure algorithm.

Day11: Further `intcode` memory abstraction with custom `Cow` type. Can't remember why.

Day12: `num` crate, threading, SoA, maybe SIMD enabled computation?
