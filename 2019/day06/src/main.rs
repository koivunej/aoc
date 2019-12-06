use std::collections::HashMap;
use std::io::BufRead;
use std::marker::PhantomData;
use petgraph::graph::{DiGraph, DefaultIx, NodeIndex};
use petgraph::algo::transitive_closure;

fn main() {
    let stdin = std::io::stdin();
    let mut locked = stdin.lock();

    // almost lost my mind juggling the &str here so I ended up buffering the whole file

    let mut buf = String::new();
    loop {
        match locked.read_line(&mut buf).unwrap() {
            0 => break,
            _ => continue,
        }
    }

    let r = Graphthing::builder()
        .parse_and_push_all(&buf)
        .build();

    println!("{}", r.reachable());

    let r = r.undirected();

    println!("{}", r.distance("YOU", "SAN"));
}

/// Unit type designating that edges are backwards from orbiting towards orbited.
struct EdgesTowardsOrbited;

/// Unit type designating that edges are bidirectional as I couldn't figure out how to make
/// undirected graph out of the directed...
struct BidirectionalEdges;

struct Graphthing<'a, S> {
    graph: DiGraph<&'a str, usize>,
    nodes: HashMap<&'a str, NodeIndex<DefaultIx>>,
    state: PhantomData<S>,
}

impl<'a> Graphthing<'a, EdgesTowardsOrbited> {
    fn builder() -> GraphthingBuilder<'a> {
        GraphthingBuilder::new()
    }

    // direct and indirect paths?
    fn reachable(&self) -> usize {
        // not sure why there are ones on the diagonal there as many values on the diagonal as
        // there are nodes
        transitive_closure(&self.graph).count_ones(..) - self.graph.node_count()
    }

    fn undirected(self) -> Graphthing<'a, BidirectionalEdges> {
        // hard to think that there is no better way to do this?
        let Graphthing { nodes, mut graph, .. } = self;
        let edges = graph.raw_edges().to_vec();

        for edge in edges {
            graph.add_edge(edge.target(), edge.source(), 1);
        }

        Graphthing {
            nodes,
            graph,
            state: PhantomData,
        }
    }

    #[cfg(test)]
    fn reachable_from(relations: &'a str) -> usize {
        Self::builder()
            .parse_and_push_all(relations)
            .build()
            .reachable()
    }
}

impl<'a> Graphthing<'a, BidirectionalEdges> {
    fn distance(&self, from: &str, to: &str) -> usize {
        let from = self.nodes.get(from).unwrap();
        let to = self.nodes.get(to).unwrap();

        let paths = petgraph::algo::dijkstra(&self.graph, *from, Some(*to), |e| *e.weight());

        // we wanted to find path between orbited; YOU and SAN both orbit a planet (hopefully a
        // different one) so subtract 2.
        (paths[to] - 2) as usize
    }
}

struct GraphthingBuilder<'a> {
    graph: DiGraph<&'a str, usize>,
    nodes: HashMap<&'a str, NodeIndex<DefaultIx>>,
}

impl<'a> GraphthingBuilder<'a> {
    fn new() -> Self {
        Self {
            graph: petgraph::Graph::new(),
            nodes: HashMap::new(),
        }
    }

    fn get_or_insert(&mut self, name: &'a str) -> NodeIndex<DefaultIx> {
        use std::collections::hash_map::Entry;
        match self.nodes.entry(name) {
            Entry::Occupied(o) => *o.get(),
            Entry::Vacant(v) => {
                let num = self.graph.add_node(name);
                *v.insert(num)
            }
        }
    }

    fn push(&mut self, orbited: &'a str, orbits: &'a str) {
        assert_ne!(orbits, "COM");

        let lhs = self.get_or_insert(orbited);
        let rhs = self.get_or_insert(orbits);

        // rhs cannot be COM so the COM)B is here rhs = B, lhs = COM
        self.graph.add_edge(rhs, lhs, 1);
    }

    fn parse_and_push_all(mut self, relations: &'a str) -> Self {
        for line in relations.lines() {
            self.parse_and_push(line);
        }
        self
    }

    fn parse<'b>(line: &'b str) -> (&'b str, &'b str) {
        let mut split = line.trim().split(')');
        let lhs = split.next().unwrap();
        let rhs = split.next().unwrap();
        assert!(split.next().is_none(), "There should only be two parts, not: \"{}\"", line.escape_debug());
        (lhs, rhs)
    }

    fn parse_and_push(&mut self, line: &'a str) {
        let (lhs, rhs) = Self::parse(line);
        self.push(lhs, rhs);
    }

    fn build(self) -> Graphthing<'a, EdgesTowardsOrbited> {
        let GraphthingBuilder { graph, nodes } = self;
        // there is some FronzenGraph in petgraph which might work here as well?
        Graphthing {
            graph,
            nodes,
            state: PhantomData,
        }
    }
}

#[test]
fn stage1_example() {
    let input =
        "COM)B\n\
        B)C\n\
        C)D\n\
        D)E\n\
        E)F\n\
        B)G\n\
        G)H\n\
        D)I\n\
        E)J\n\
        J)K\n\
        K)L\n";

    assert_eq!(Graphthing::reachable_from(input), 42);
}

#[test]
fn simplified_stage1_examples() {
    assert_eq!(Graphthing::reachable_from("COM)B\n"), 1);
    assert_eq!(Graphthing::reachable_from("COM)B\nB)C\n"), 3);
}

#[test]
fn stage2_example() {
    let input =
        "COM)B\n\
        B)C\n\
        C)D\n\
        D)E\n\
        E)F\n\
        B)G\n\
        G)H\n\
        D)I\n\
        E)J\n\
        J)K\n\
        K)L\n\
        K)YOU\n\
        I)SAN\n";

    let r = Graphthing::builder()
        .parse_and_push_all(input)
        .build()
        .undirected();

    assert_eq!(r.distance("YOU", "SAN"), 4);
}
