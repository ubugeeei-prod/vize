//! P2-11 installment 5 witness: static native HTML, interpolations,
//! mixed text siblings, static-name binds, static-name events including
//! event/key/option modifiers, native v-if, native v-for,
//! object-spread v-bind, static-name components, object v-on, and
//! implicit text / native / component default slots, compared
//! **byte-for-byte** including helper usage.
//!
//! S2 owns the shipped DOM renderer. The comparison battery stays as a
//! byte-for-byte release ratchet, and its pinned count makes any accidental
//! reduction in coverage loud.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

#[test]
fn s2_native_html_and_interpolations_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(support::battery::dom::DOM_BATTERY);
}
