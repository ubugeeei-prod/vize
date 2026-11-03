//! The IVM laws of the update machine, `incremental.rs`, for keyed and
//! positional `v-for` alike (they differ only in the address). Replaces
//! `Impeto.IncrementalLaws`. Bound: sibling lists of at most 2 nodes, trees
//! of depth 2 (elements, their element children, then text), addresses, tags
//! and text from a 3-letter alphabet, a 1-bit payload, identities below 8,
//! up to 1 already-seen address and fuel up to 3.

use super::incremental::{Node, State, View, erase_all, reconcile, retain, siblings, update};

type V = View<u8, bool>;
type N = Node<u8, bool>;

const WIDTH: usize = 2;
const DEPTH: usize = 2;

fn letter() -> u8 {
    let letter: u8 = kani::any();
    kani::assume(letter < 3);
    letter
}

fn identity() -> u32 {
    let identity: u32 = kani::any();
    kani::assume(identity < 8);
    identity
}

fn fuel() -> usize {
    let fuel: usize = kani::any();
    kani::assume(fuel <= DEPTH + 1);
    fuel
}

/// Up to two siblings, each list built at its final size: cutting a longer
/// list would run drop glue over a symbolic tail, which costs CBMC dearly.
fn siblings_of<T>(item: impl Fn() -> T) -> Vec<T> {
    match kani::any::<u8>() % 3 {
        0 => Vec::new(),
        1 => vec![item()],
        _ => vec![item(), item()],
    }
}

fn view(depth: usize) -> V {
    if depth == 0 || kani::any() {
        return View::Text(letter());
    }
    View::Element {
        address: letter(),
        tag: letter(),
        payload: kani::any(),
        children: views(depth - 1),
    }
}

fn views(depth: usize) -> Vec<V> {
    siblings_of(|| view(depth))
}

fn node(depth: usize) -> N {
    if depth == 0 || kani::any() {
        return Node::Text(letter());
    }
    Node::Element {
        address: letter(),
        identity: identity(),
        tag: letter(),
        payload: kani::any(),
        children: nodes(depth - 1),
    }
}

fn nodes(depth: usize) -> Vec<N> {
    siblings_of(|| node(depth))
}

/// Identity of the first previous sibling carrying `owner`.
fn retained_identity(previous: &[N], owner: u8) -> Option<u32> {
    match previous.iter().find(|node| node.address() == Some(&owner)) {
        Some(Node::Element { identity, .. }) => Some(*identity),
        _ => None,
    }
}

/// Elements the update must create: views whose address has no retained node.
fn allocations_all(previous: &[N], desired: &[V]) -> u64 {
    desired
        .iter()
        .map(|view| allocations_one(previous, view))
        .sum()
}

fn allocations_one(previous: &[N], view: &V) -> u64 {
    match view {
        View::Text(_) => 0,
        View::Element {
            address, children, ..
        } => match previous.iter().find(|node| node.address() == Some(address)) {
            Some(Node::Element { children: old, .. }) => allocations_all(old, children),
            _ => 1 + allocations_all(&[], children),
        },
    }
}

/// `retain_ok`: retention reuses the first previous element with the address,
/// keeping its children and the allocator, or allocates the next identity.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn retain_keeps_the_addressed_element_or_allocates() {
    let previous = nodes(1);
    let (owner, tag) = (letter(), letter());
    let next = identity();
    if let Ok((kept, children, after)) = retain(&previous, &owner, &tag, next) {
        match previous.iter().find(|node| node.address() == Some(&owner)) {
            Some(Node::Element {
                identity,
                children: old,
                ..
            }) => {
                assert!(retained_identity(&previous, owner) == Some(*identity));
                assert!(kept == *identity && after == next && children == old.as_slice());
            }
            _ => {
                assert!(retained_identity(&previous, owner).is_none());
                assert!(kept == next && after == next + 1 && children.is_empty());
            }
        }
    }
}

/// `siblings_erase_of`, `reconcile_erase`: the patched tree, identities
/// erased, is exactly the freshly rendered view.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn the_patched_tree_displays_the_view() {
    let previous = nodes(DEPTH);
    let desired = views(DEPTH);
    let seen: Vec<u8> = if kani::any() {
        vec![letter()]
    } else {
        Vec::new()
    };
    let (next, fuel) = (identity(), fuel());
    if let Ok((patched, _)) = siblings(fuel, &previous, &seen, &desired, next) {
        assert!(erase_all(&patched) == desired);
    }
    if let Ok((patched, _)) = reconcile(fuel, &previous, &desired, next) {
        assert!(erase_all(&patched) == desired);
    }
}

/// `siblings_allocations_of`, `reconcile_allocations`, `siblings_advance`:
/// the allocator advances by exactly the elements with no retained address.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn allocation_is_exactly_the_inserted_delta() {
    let previous = nodes(DEPTH);
    let desired = views(DEPTH);
    let seen: Vec<u8> = if kani::any() {
        vec![letter()]
    } else {
        Vec::new()
    };
    let (next, fuel) = (identity(), fuel());
    let delta = allocations_all(&previous, &desired);
    if let Ok((_, after)) = siblings(fuel, &previous, &seen, &desired, next) {
        assert!(u64::from(after) == u64::from(next) + delta);
        assert!(next <= after);
    }
    if let Ok((_, after)) = reconcile(fuel, &previous, &desired, next) {
        assert!(u64::from(after) == u64::from(next) + delta);
    }
}

/// `siblings_identity`: every top-level element keeps the identity of the
/// previous sibling with its address, or takes a fresh one from `[next, after)`.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn top_level_identities_are_retained_or_fresh() {
    let previous = nodes(DEPTH);
    let desired = views(DEPTH);
    let seen: Vec<u8> = if kani::any() {
        vec![letter()]
    } else {
        Vec::new()
    };
    let (next, fuel) = (identity(), fuel());
    if let Ok((patched, after)) = siblings(fuel, &previous, &seen, &desired, next) {
        for node in &patched {
            if let Node::Element {
                address, identity, ..
            } = node
            {
                let retained = retained_identity(&previous, *address);
                assert!(
                    retained == Some(*identity)
                        || (retained.is_none() && next <= *identity && *identity < after)
                );
            }
        }
    }
}

/// `update_tree`, `update_without_insertions`: an accepted update displays
/// exactly the from-scratch view, and a keyed reorder or payload patch whose
/// every address survives allocates nothing.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn an_update_is_recompute_and_reorders_allocate_nothing() {
    let state = State {
        roots: nodes(DEPTH),
        next_identity: identity(),
    };
    let view = views(DEPTH);
    if let Ok(result) = update(fuel(), &state, &view) {
        assert!(erase_all(&result.roots) == view);
        if allocations_all(&state.roots, &view) == 0 {
            assert!(result.next_identity == state.next_identity);
        }
    }
}
