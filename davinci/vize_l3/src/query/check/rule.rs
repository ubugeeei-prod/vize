//! Local obligations of a path witness and of a frontier closed under an edge.

#[cfg(creusot)]
use creusot_std::prelude::*;

#[inline]
#[cfg_attr(creusot, ensures(result == (parent && selected && destination@ == claimed@)))]
pub(super) fn justifies(parent: bool, selected: bool, destination: u32, claimed: u32) -> bool {
    parent && selected && destination == claimed
}

#[inline]
#[cfg_attr(creusot, ensures(result == ((parent && selected) ==> child)))]
pub(super) fn closed(parent: bool, selected: bool, child: bool) -> bool {
    !parent || !selected || child
}
