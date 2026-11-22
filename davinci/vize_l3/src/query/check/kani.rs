//! Bounded equivalence of the shipped witness and closure rules with a
//! separate transitive-closure computation. One mask per ordered pair can
//! encode several edges of different kinds, including loops.
mod rule;

#[kani::proof]
fn machine_word_witness() {
    let parent = kani::any();
    let selected = kani::any();
    let target: u32 = kani::any();
    let claim: u32 = kani::any();
    let accepted = rule::justifies(parent, selected, target, claim);
    assert_eq!(accepted, parent && selected && claim == target);
}

#[kani::proof]
#[kani::unwind(4)]
fn three_node_certificates_are_exact() {
    let edges: [u8; 9] = kani::any();
    let kinds: u8 = kani::any();
    let reverse: bool = kani::any();
    let root: usize = kani::any();
    kani::assume(root < 3);
    let rows: [bool; 3] = kani::any();
    let ranks: [u8; 3] = kani::any();
    let selected = |from: usize, to: usize| {
        edges[if reverse { to * 3 + from } else { from * 3 + to }] & kinds & 7 != 0
    };
    let mut accepts = rows[root];
    for child in 0..3 {
        let mut path = child == root;
        for parent in 0..3 {
            let edge = selected(parent, child);
            path |= rule::justifies(rows[parent] && ranks[parent] < ranks[child], edge,
                                    child as u32, child as u32);
            accepts &= rule::closed(rows[parent], edge, rows[child]);
        }
        accepts &= !rows[child] || path;
    }
    // With three vertices, a shortest simple path contains at most two edges.
    let mut reached = [false; 3];
    reached[root] = true;
    for _ in 0..2 {
        let old = reached;
        for to in 0..3 {
            for from in 0..3 {
                reached[to] |= old[from] && selected(from, to);
            }
        }
    }
    if accepts { assert_eq!(rows, reached); }
    kani::cover!(accepts && rows == [true; 3], "three reachable operations admitted");
}
