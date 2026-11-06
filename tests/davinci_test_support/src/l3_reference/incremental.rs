//! The reference update machine that TS-29 executes. It owns retained
//! identities and the previous materialized tree. It receives a fresh view,
//! matches within each parent by L3 address, patches payloads, inserts missing
//! nodes and retires absent ones. No runtime output is input. `kani.rs`
//! proves every accepted update equals recompute-from-scratch and allocates
//! exactly the inserted delta. Strings and payloads are type parameters so
//! the proofs run on small domains.

/// A freshly rendered view. Addresses encode L3 operation ownership and loop
/// scopes; they are reference identities, never inferred from DOM strings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum View<S, P> {
    Text(S),
    Element {
        address: S,
        tag: S,
        payload: P,
        children: Vec<View<S, P>>,
    },
}

/// A materialized node with its retained identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node<S, P> {
    Text(S),
    Element {
        address: S,
        identity: u32,
        tag: S,
        payload: P,
        children: Vec<Node<S, P>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State<S, P> {
    pub roots: Vec<Node<S, P>>,
    pub next_identity: u32,
}

impl<S, P> Default for State<S, P> {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            next_identity: 0,
        }
    }
}

impl<S, P> Node<S, P> {
    #[must_use]
    pub fn address(&self) -> Option<&S> {
        match self {
            Self::Text(_) => None,
            Self::Element { address, .. } => Some(address),
        }
    }
}

/// Forget retained identities: the view a materialized tree displays.
#[must_use]
pub fn erase<S: Clone, P: Clone>(node: &Node<S, P>) -> View<S, P> {
    match node {
        Node::Text(value) => View::Text(value.clone()),
        Node::Element {
            address,
            tag,
            payload,
            children,
            ..
        } => View::Element {
            address: address.clone(),
            tag: tag.clone(),
            payload: payload.clone(),
            children: erase_all(children),
        },
    }
}

#[must_use]
pub fn erase_all<S: Clone, P: Clone>(nodes: &[Node<S, P>]) -> Vec<View<S, P>> {
    nodes.iter().map(erase).collect()
}

/// The identity and the previous children kept for `owner`, and the
/// allocator after it. The children are borrowed, never copied.
pub type Retained<'p, S, P> = (u32, &'p [Node<S, P>], u32);

/// Reuse the first previous sibling with the same L3 address, or allocate.
pub fn retain<'p, S: PartialEq, P>(
    previous: &'p [Node<S, P>],
    owner: &S,
    tag: &S,
    next: u32,
) -> Result<Retained<'p, S, P>, &'static str> {
    match previous.iter().find(|node| node.address() == Some(owner)) {
        Some(Node::Element {
            identity,
            tag: old,
            children,
            ..
        }) => {
            if tag == old {
                Ok((*identity, children, next))
            } else {
                Err("stable L3 identity changed element kind")
            }
        }
        _ => {
            let after = next.checked_add(1).ok_or("identity allocator overflow")?;
            Ok((next, &[], after))
        }
    }
}

pub type Reconciled<S, P> = (Vec<Node<S, P>>, u32);

/// Reconcile one sibling list, spending one unit of structural fuel per level.
pub fn reconcile<S: Clone + PartialEq, P: Clone>(
    fuel: usize,
    previous: &[Node<S, P>],
    desired: &[View<S, P>],
    next: u32,
) -> Result<Reconciled<S, P>, &'static str> {
    let Some(fuel) = fuel.checked_sub(1) else {
        return Err("incremental tree exceeds structural fuel");
    };
    siblings(fuel, previous, &[], desired, next)
}

/// Siblings left to right; `seen` holds the addresses placed before them.
pub fn siblings<S: Clone + PartialEq, P: Clone>(
    fuel: usize,
    previous: &[Node<S, P>],
    seen: &[S],
    desired: &[View<S, P>],
    next: u32,
) -> Result<Reconciled<S, P>, &'static str> {
    let mut nodes = Vec::with_capacity(desired.len());
    let mut next = next;
    for (index, view) in desired.iter().enumerate() {
        match view {
            View::Text(value) => nodes.push(Node::Text(value.clone())),
            View::Element {
                address,
                tag,
                payload,
                children,
            } => {
                // Placed so far: `seen`, then every earlier element here.
                let earlier = desired.get(..index).unwrap_or_default();
                let placed = |other: &View<S, P>| matches!(other, View::Element { address: placed, .. } if placed == address);
                if seen.iter().any(|placed| placed == address) || earlier.iter().any(placed) {
                    return Err("duplicate materialized address");
                }
                let (identity, old, after) = retain(previous, address, tag, next)?;
                let (updated, after) = reconcile(fuel, old, children, after)?;
                next = after;
                nodes.push(Node::Element {
                    address: address.clone(),
                    identity,
                    tag: tag.clone(),
                    payload: payload.clone(),
                    children: updated,
                });
            }
        }
    }
    Ok((nodes, next))
}

/// One update. The drift check is the runtime face of `update_tree`.
pub fn update<S: Clone + PartialEq, P: Clone + PartialEq>(
    fuel: usize,
    state: &State<S, P>,
    view: &[View<S, P>],
) -> Result<State<S, P>, &'static str> {
    let (roots, next_identity) = reconcile(fuel, &state.roots, view, state.next_identity)?;
    if erase_all(&roots) != view {
        return Err("incremental versus from-scratch tree drift");
    }
    Ok(State {
        roots,
        next_identity,
    })
}
