//! Deductive verification of the same local edge rules consumed by the
//! independent checker. The proof crate is separate from the compiler.
//!
//! `accepted_certificates_are_exact` is Lean `Davinci.exact_answer`
//! (`tests/formal/ir-graph/Reach.lean`) over the shipped rules, for any
//! number of operations: a certificate that admits its rows by
//! `rule::justifies` (each row other than the root has an earlier parent row
//! and a selected edge from it) and that `rule::closed` accepts for every
//! selected edge holds exactly the operations reachable from the root.
mod rule;

use creusot_std::prelude::*;

/// The relation already incorporates a query's direction and its selected
/// edge kinds: `edges[from][to]` holds when a selected edge leads from `from`
/// to `to`. Several edges between one pair and loops are one entry.
#[logic(open)]
pub fn step(edges: Seq<Vec<bool>>, from: Int, to: Int) -> bool {
    pearlite! {
        0 <= from && from < edges.len() && 0 <= to && to < edges[from]@.len() && edges[from]@[to]
    }
}

/// Lean `Path`: `to` is reachable from `root` along at most `k` steps.
#[logic]
#[variant(k)]
#[requires(k >= 0)]
pub fn path_within(edges: Seq<Vec<bool>>, root: Int, to: Int, k: Int) -> bool {
    pearlite! {
        if k == 0 {
            to == root
        } else {
            path_within(edges, root, to, k - 1)
                || exists<from: Int> path_within(edges, root, from, k - 1) && step(edges, from, to)
        }
    }
}

/// One more step extends a path by one edge.
#[logic]
#[ensures(forall<k: Int, from: Int, to: Int> k >= 0 && path_within(edges, root, from, k)
    && step(edges, from, to) ==> path_within(edges, root, to, k + 1))]
pub fn paths_extend(edges: Seq<Vec<bool>>, root: Int) {}

/// Lean `Path edge root to`, which includes the empty path.
#[logic(open)]
pub fn reachable(edges: Seq<Vec<bool>>, root: Int, to: Int) -> bool {
    pearlite! { exists<k: Int> k >= 0 && path_within(edges, root, to, k) }
}

/// Lean `Supported`: every row other than the root has a parent row of a
/// lower rank (an earlier witness) with a step to it.
#[logic(open)]
pub fn supported(edges: Seq<Vec<bool>>, rows: Seq<bool>, ranks: Seq<u32>, root: Int) -> bool {
    pearlite! {
        forall<child: Int> 0 <= child && child < rows.len() && rows[child] && child != root ==>
            exists<parent: Int> 0 <= parent && parent < rows.len() && rows[parent]
                && ranks[parent]@ < ranks[child]@ && step(edges, parent, child)
    }
}

/// The rows are closed under every step.
#[logic(open)]
pub fn closed(edges: Seq<Vec<bool>>, rows: Seq<bool>) -> bool {
    pearlite! {
        forall<parent: Int, child: Int> 0 <= parent && parent < rows.len()
            && 0 <= child && child < rows.len() && rows[parent] && step(edges, parent, child)
            ==> rows[child]
    }
}

/// Lean `supported_sound`, by induction on the rank: a supported row is
/// reachable.
#[logic]
#[variant(rank)]
#[requires(rank >= 0)]
#[requires(rows.len() == edges.len() && ranks.len() == edges.len())]
#[requires(supported(edges, rows, ranks, root))]
#[ensures(forall<to: Int> 0 <= to && to < rows.len() && rows[to] && ranks[to]@ <= rank
    ==> reachable(edges, root, to))]
pub fn supported_rows_are_reachable(
    edges: Seq<Vec<bool>>,
    rows: Seq<bool>,
    ranks: Seq<u32>,
    root: Int,
    rank: Int,
) {
    pearlite! {
        if rank > 0 {
            paths_extend(edges, root);
            supported_rows_are_reachable(edges, rows, ranks, root, rank - 1)
        }
    }
}

/// The other half of Lean `exact_answer`, by induction on the path length:
/// rows that contain the root and are closed hold every reachable operation.
#[logic]
#[variant(k)]
#[requires(k >= 0)]
#[requires(rows.len() == edges.len())]
#[requires(forall<i: Int> 0 <= i && i < edges.len() ==> edges[i]@.len() == edges.len())]
#[requires(0 <= root && root < rows.len() && rows[root])]
#[requires(closed(edges, rows))]
#[ensures(forall<to: Int> path_within(edges, root, to, k) ==> 0 <= to && to < rows.len() && rows[to])]
pub fn closed_rows_hold_every_path(edges: Seq<Vec<bool>>, rows: Seq<bool>, root: Int, k: Int) {
    pearlite! {
        if k > 0 {
            closed_rows_hold_every_path(edges, rows, root, k - 1)
        }
    }
}

/// The certificate check of every vertex against every parent, through the
/// shipped `rule::justifies` and `rule::closed`, for any number of
/// operations. When it accepts, the rows are exactly the reachable set.
#[requires(rows@.len() == edges@.len() && ranks@.len() == edges@.len())]
#[requires(forall<i: Int> 0 <= i && i < edges@.len() ==> edges[i]@.len() == edges@.len())]
#[requires(root@ < edges@.len())]
#[ensures(result ==> forall<to: Int> 0 <= to && to < rows@.len()
    ==> rows[to] == reachable(edges@, root@, to))]
pub fn accepted_certificates_are_exact(
    edges: &[Vec<bool>],
    rows: &[bool],
    ranks: &[u32],
    root: usize,
) -> bool {
    let count = edges.len();
    let mut accepts = rows[root];
    let mut child = 0;
    #[invariant(child@ <= count@)]
    #[invariant(accepts ==> rows[root@])]
    #[invariant(accepts ==> forall<c: Int> 0 <= c && c < child@ && rows[c] && c != root@ ==>
        exists<p: Int> 0 <= p && p < count@ && rows[p] && ranks[p]@ < ranks[c]@
            && step(edges@, p, c))]
    #[invariant(accepts ==> forall<p: Int, c: Int> 0 <= p && p < count@ && 0 <= c && c < child@
        && rows[p] && step(edges@, p, c) ==> rows[c])]
    #[variant(count@ - child@)]
    while child < count {
        let mut path = child == root;
        let mut parent = 0;
        #[invariant(parent@ <= count@)]
        #[invariant(path ==> child == root || exists<p: Int> 0 <= p && p < parent@ && rows[p]
            && ranks[p]@ < ranks[child@]@ && step(edges@, p, child@))]
        #[invariant(accepts ==> rows[root@])]
        #[invariant(accepts ==> forall<c: Int> 0 <= c && c < child@ && rows[c] && c != root@ ==>
            exists<p: Int> 0 <= p && p < count@ && rows[p] && ranks[p]@ < ranks[c]@
                && step(edges@, p, c))]
        #[invariant(accepts ==> forall<p: Int, c: Int> 0 <= p && p < count@ && 0 <= c && c < child@
            && rows[p] && step(edges@, p, c) ==> rows[c])]
        #[invariant(accepts ==> forall<p: Int> 0 <= p && p < parent@
            && rows[p] && step(edges@, p, child@) ==> rows[child@])]
        #[variant(count@ - parent@)]
        while parent < count {
            let edge = edges[parent][child];
            path |= rule::justifies(
                rows[parent] && ranks[parent] < ranks[child],
                edge,
                child as u32,
                child as u32,
            );
            accepts &= rule::closed(rows[parent], edge, rows[child]);
            parent += 1;
        }
        accepts &= !rows[child] || path;
        child += 1;
    }
    proof_assert!(accepts ==> supported(edges@, rows@, ranks@, root@));
    proof_assert!(accepts ==> closed(edges@, rows@));
    proof_assert!(accepts ==> {
        supported_rows_are_reachable(edges@, rows@, ranks@, root@, u32::MAX@);
        forall<to: Int> 0 <= to && to < rows@.len() && rows[to] ==> reachable(edges@, root@, to)
    });
    proof_assert!(accepts ==> forall<to: Int, k: Int> k >= 0 && path_within(edges@, root@, to, k)
        ==> {
            closed_rows_hold_every_path(edges@, rows@, root@, k);
            0 <= to && to < rows@.len() && rows[to]
        });
    accepts
}
