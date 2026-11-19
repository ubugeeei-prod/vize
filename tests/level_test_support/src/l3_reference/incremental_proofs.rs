//! Bounded laws of the incremental reference machine in `incremental.rs`.
//!
//! These harnesses replace `L3.IncrementalLaws`. The Lean statements are
//! unbounded. Kani covers fixed structural shapes with symbolic values: empty
//! and text-only siblings, two element siblings, a depth-two element tree, a
//! keyed reorder, and one insertion. Addresses and tags range over three
//! symbols, payloads over both booleans, and identities below eight.
//!
//! Keeping the shape concrete is deliberate. CBMC could not bound recursive
//! vector-backed symbolic trees even at width two; fixed shapes still quantify
//! over the values that decide retention, tag compatibility, payload
//! replacement, allocation, and ordering.

use super::incremental::{Node, View, reconcile, retain, siblings};

type V = View<u8, bool>;
type N = Node<u8, bool>;

fn symbol() -> u8 {
    let value: u8 = kani::any();
    kani::assume(value < 3);
    value
}

fn identity() -> u32 {
    let value: u32 = kani::any();
    kani::assume(value < 8);
    value
}

fn node(address: u8, identity: u32, tag: u8, children: Vec<N>) -> N {
    Node::Element {
        address,
        identity,
        tag,
        payload: kani::any(),
        children,
    }
}

/// Lean `retain_ok`, retained branch: a matching sibling keeps its identity,
/// children, and allocator.
#[kani::proof]
#[kani::unwind(3)]
#[kani::solver(cadical)]
fn retain_keeps_the_addressed_element() {
    let (address, kept_identity, old_tag) = (symbol(), identity(), symbol());
    let previous = [node(address, kept_identity, old_tag, Vec::new())];
    let next = identity();
    match retain(&previous, &address, &old_tag, next) {
        Ok((kept, children, after)) => {
            assert!(kept == kept_identity && after == next);
            assert!(children.is_empty());
        }
        Err(_) => {
            assert!(false);
        }
    }
}

/// The same retained branch over the non-empty concrete shape: `retain`
/// returns the stored child slice rather than reconstructing it.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn retain_preserves_the_previous_child_slice() {
    let previous = [Node::Element {
        address: 0,
        identity: 1,
        tag: 2,
        payload: false,
        children: vec![N::Text(1)],
    }];
    match retain(&previous, &0, &2, 3) {
        Ok((kept, children, after)) => {
            assert!(kept == 1 && after == 3);
            assert!(matches!(children, [N::Text(1)]));
        }
        Err(_) => {
            assert!(false);
        }
    }
}

/// Lean `retain_ok`, refusal branch: changing the element kind at a retained
/// address is rejected before the allocator or children can change.
#[kani::proof]
#[kani::unwind(3)]
#[kani::solver(cadical)]
fn retain_rejects_a_changed_tag() {
    let address = symbol();
    let previous = [node(address, identity(), 0, Vec::new())];
    assert!(retain(&previous, &address, &1, identity()).is_err());
}

/// Lean `retain_ok`, allocation branch: a missing address consumes exactly
/// the next identity and starts with no retained children.
#[kani::proof]
#[kani::unwind(3)]
#[kani::solver(cadical)]
fn retain_allocates_one_missing_address() {
    let existing = symbol();
    let missing = symbol();
    kani::assume(existing != missing);
    let previous = [node(existing, identity(), symbol(), Vec::new())];
    let next = identity();
    match retain(&previous, &missing, &symbol(), next) {
        Ok((kept, children, after)) => {
            assert!(kept == next);
            assert!(after == next + 1);
            assert!(children.is_empty());
        }
        Err(_) => {
            assert!(false);
        }
    }
}

/// Lean `siblings_erase_of`, `siblings_allocations_of`, and
/// `siblings_advance`, for the text arm.
#[kani::proof]
#[kani::unwind(3)]
#[kani::solver(cadical)]
fn a_text_sibling_preserves_display_and_allocator() {
    let value = symbol();
    let next = identity();
    match siblings(0, &[], &[], &[V::Text(value)], next) {
        Ok((patched, after)) => {
            assert!(after == next);
            assert!(matches!(patched.as_slice(), [N::Text(actual)] if *actual == value));
        }
        Err(_) => {
            assert!(false);
        }
    }
}

/// Lean `siblings_erase_of`, `siblings_allocations_of`, `siblings_advance`,
/// and `siblings_identity`, for one retained-or-inserted element.
#[kani::proof]
#[kani::unwind(3)]
#[kani::solver(cadical)]
fn top_level_identities_are_retained_or_fresh() {
    let should_retain: bool = kani::any();
    let previous_address = if should_retain { 1 } else { 0 };
    let (tag, previous_identity, next) = (symbol(), identity(), identity());
    let previous = [node(previous_address, previous_identity, tag, Vec::new())];
    let payload = kani::any();
    let desired = [View::Element {
        address: 1,
        tag,
        payload,
        children: Vec::new(),
    }];
    match siblings(1, &previous, &[], &desired, next) {
        Ok((patched, after)) => {
            assert!(matches!(
                patched.as_slice(),
                [Node::Element {
                    address: 1,
                    identity,
                    tag: actual_tag,
                    payload: actual_payload,
                    children,
                }] if *actual_tag == tag
                    && *actual_payload == payload
                    && children.is_empty()
                    && if should_retain {
                        *identity == previous_identity && after == next
                    } else {
                        *identity == next && after == next + 1
                            && next <= *identity && *identity < after
                    }
            ));
        }
        Err(_) => {
            assert!(false);
        }
    }
}

/// Lean `reconcile_erase` and `reconcile_allocations` at depth two. The root
/// is retained and its previous child is replaced by one inserted child.
/// Identities and payloads remain symbolic while the structural relationship
/// stays concrete enough for CBMC to bound.
#[kani::proof]
#[kani::unwind(3)]
#[kani::solver(cadical)]
fn the_patched_tree_displays_the_view() {
    let root_identity = identity();
    let previous = [node(
        0,
        root_identity,
        0,
        vec![node(1, identity(), 1, Vec::new())],
    )];
    let root_payload = kani::any();
    let child_payload = kani::any();
    let desired = [View::Element {
        address: 0,
        tag: 0,
        payload: root_payload,
        children: vec![View::Element {
            address: 2,
            tag: 2,
            payload: child_payload,
            children: Vec::new(),
        }],
    }];
    let next = identity();
    match reconcile(3, &previous, &desired, next) {
        Ok((patched, after)) => {
            assert!(after == next + 1);
            assert!(matches!(
                patched.as_slice(),
                [Node::Element {
                    address: 0,
                    identity: actual_root_identity,
                    tag: 0,
                    payload: actual_root_payload,
                    children,
                }] if *actual_root_identity == root_identity
                    && *actual_root_payload == root_payload
                    && matches!(
                        children.as_slice(),
                        [Node::Element {
                            address: 2,
                            identity: actual_child_identity,
                            tag: 2,
                            payload: actual_child_payload,
                            children: grand_children,
                        }] if *actual_child_identity == next
                            && *actual_child_payload == child_payload
                            && grand_children.is_empty()
                    )
            ));
        }
        Err(_) => {
            assert!(false);
        }
    }
}

/// The two-root keyed-reorder representative of the same laws. The actual
/// `retain` primitive selects both roots in desired order without advancing
/// the allocator; exact reconstruction is covered by the depth-two
/// `reconcile` harness above.
#[kani::proof]
#[kani::unwind(3)]
#[kani::solver(cadical)]
fn an_update_is_recompute_and_reorders_allocate_nothing() {
    let (first_identity, second_identity, next) = (identity(), identity(), identity());
    let previous = [
        node(0, first_identity, 0, Vec::new()),
        node(1, second_identity, 1, Vec::new()),
    ];
    match retain(&previous, &1, &1, next) {
        Ok((actual_second, second_children, after_second)) => {
            assert!(actual_second == second_identity);
            assert!(second_children.is_empty());
            assert!(after_second == next);
            match retain(&previous, &0, &0, after_second) {
                Ok((actual_first, first_children, after_first)) => {
                    assert!(actual_first == first_identity);
                    assert!(first_children.is_empty());
                    assert!(after_first == next);
                }
                Err(_) => {
                    assert!(false);
                }
            }
        }
        Err(_) => {
            assert!(false);
        }
    }
}

/// Lean `siblings_allocations_of`, `reconcile_allocations`, and
/// `siblings_advance`: one missing address produces one exact fresh delta.
#[kani::proof]
#[kani::unwind(3)]
#[kani::solver(cadical)]
fn allocation_is_exactly_the_inserted_delta() {
    let next = identity();
    let previous = [node(0, identity(), symbol(), Vec::new())];
    let (tag, payload) = (symbol(), kani::any());
    let desired = [View::Element {
        address: 1,
        tag,
        payload,
        children: Vec::new(),
    }];
    match siblings(1, &previous, &[], &desired, next) {
        Ok((patched, after)) => {
            assert!(after == next + 1);
            assert!(matches!(
                patched.as_slice(),
                [Node::Element {
                    address: 1,
                    identity,
                    tag: actual_tag,
                    payload: actual_payload,
                    children,
                }] if *identity == next
                    && *actual_tag == tag
                    && *actual_payload == payload
                    && children.is_empty()
            ));
        }
        Err(_) => {
            assert!(false);
        }
    }
}
