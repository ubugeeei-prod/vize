//! Creusot proves the span, range and mapping laws of the shipped L0 source
//! coordinates for every `u32` and `i32` value. This file is the library root
//! of `tests/formal/creusot/coordinates` and is not part of `vize_l0`'s module
//! tree: the modules below are the ones the product compiles, found by
//! ordinary module discovery.
//!
//! Each shipped function carries its contract in the shipped source, restated
//! from its documentation, never from its body: a span or range is the
//! half-open set of offsets `[start, end)` (`covers`), `union` is the least
//! range enclosing both, `intersection` holds exactly the offsets in both,
//! the shifts move each endpoint in the integers and clamp it into `u32`, and
//! a mapping carries an offset by its distance from the range start, clamped
//! to the last offset of the other range. The functions below call the
//! shipped functions and assert each law over their contracts, for every
//! input. They replace the Kani harnesses of the same names, which checked
//! the same statements one SAT query at a time. See
//! docs/davinci/decisions/2026-11-17-creusot-inventory.md.

mod source_range;
mod span;

pub use source_range::{SourceMapping, SourceRange, carried, shifted};
pub use span::Span;

/// `source_range.rs` spells its mapping metadata with `crate::CompactString`.
/// No law reads that metadata, and the proof crate has no `compact_str`, so an
/// opaque type stands in for the field type.
pub struct CompactString;

use creusot_std::prelude::*;

/// Two nonempty ranges that cover the same offsets are the same range.
#[logic]
#[requires(x.start@ < x.end@ && y.start@ < y.end@)]
#[requires(forall<offset: Int> x.covers(offset) == y.covers(offset))]
#[ensures(x == y)]
pub fn a_nonempty_range_is_its_offsets(x: SourceRange, y: SourceRange) {}

/// `Span::len` is the number of offsets in `[start, end)`: the span covers
/// exactly the `len` offsets from its start, and is empty exactly when that
/// number is zero.
pub fn span_length_counts_the_offsets_it_covers(span: Span) {
    let len = span.len();
    let empty = span.is_empty();
    proof_assert!(empty == (len@ == 0));
    proof_assert!(empty || span.start@ + len@ == span.end@);
    proof_assert!(forall<offset: Int>
        span.covers(offset) == (span.start@ <= offset && offset < span.start@ + len@));
}

/// The block-relative span never grows, inverts the shift for a span inside
/// the block, and starts at zero for a span starting before the block.
pub fn span_block_relative_form_never_grows_and_inverts_the_shift(span: Span, block_start: u32) {
    let relative = span.to_block_relative(block_start);
    let before = span.len();
    let after = relative.len();
    proof_assert!(after@ <= before@);
    proof_assert!(block_start@ <= span.start@ && block_start@ <= span.end@ ==>
        after@ == before@
            && relative.start@ + block_start@ == span.start@
            && relative.end@ + block_start@ == span.end@);
    proof_assert!(block_start@ > span.start@ ==> relative.start@ == 0);
}

/// Moving a span and its block together leaves the block-relative form.
#[requires(span.start@ + shift@ <= u32::MAX@)]
#[requires(span.end@ + shift@ <= u32::MAX@)]
#[requires(block_start@ + shift@ <= u32::MAX@)]
pub fn span_block_relative_form_ignores_where_the_block_starts(
    span: Span,
    block_start: u32,
    shift: u32,
) {
    let relative = span.to_block_relative(block_start);
    let moved = Span::new(span.start + shift, span.end + shift);
    let moved_relative = moved.to_block_relative(block_start + shift);
    proof_assert!(moved_relative == relative);
}

/// `union` is commutative, associative, idempotent and the least range whose
/// endpoints enclose both.
pub fn source_range_union_is_the_least_enclosing_range(
    a: SourceRange,
    b: SourceRange,
    c: SourceRange,
) {
    let union = a.union(&b);
    let flipped = b.union(&a);
    proof_assert!(union.encloses(flipped) && flipped.encloses(union));
    proof_assert!(union == flipped);
    let same = a.union(&a);
    proof_assert!(same.encloses(a) && a.encloses(same));
    proof_assert!(same == a);
    let left = union.union(&c);
    let tail = b.union(&c);
    let right = a.union(&tail);
    proof_assert!(left.encloses(right) && right.encloses(left));
    proof_assert!(left == right);
    proof_assert!(union.encloses(a) && union.encloses(b));
    proof_assert!(c.encloses(a) && c.encloses(b) ==> c.encloses(union));
}

/// The offsets an optional range covers; `None` covers none.
#[logic(open)]
pub fn covered(range: Option<SourceRange>, offset: Int) -> bool {
    match range {
        Some(range) => range.covers(offset),
        None => false,
    }
}

/// `intersection` holds exactly the offsets in both ranges, is `Some` exactly
/// when they `intersects`, never exceeds either range, and is a semilattice
/// meet: commutative, associative and idempotent on nonempty ranges.
pub fn source_range_intersection_is_the_offsets_in_both(
    a: SourceRange,
    b: SourceRange,
    c: SourceRange,
) {
    let both = a.intersection(&b);
    proof_assert!(forall<offset: Int>
        covered(both, offset) == (a.covers(offset) && b.covers(offset)));
    let meets = a.intersects(&b);
    proof_assert!(meets == (both != None));
    if let Some(range) = both {
        let len = range.len();
        let outer = a.len();
        let empty = range.is_empty();
        proof_assert!(!empty && len@ <= outer@);
    }

    let flipped = b.intersection(&a);
    proof_assert!(match (both, flipped) {
        (Some(x), Some(y)) => {
            a_nonempty_range_is_its_offsets(x, y);
            x == y
        }
        (None, None) => true,
        _ => false,
    });
    proof_assert!(both == flipped);

    let left = match both {
        Some(range) => range.intersection(&c),
        None => None,
    };
    let tail = b.intersection(&c);
    let right = match tail {
        Some(range) => a.intersection(&range),
        None => None,
    };
    proof_assert!(forall<offset: Int>
        covered(left, offset) == (a.covers(offset) && b.covers(offset) && c.covers(offset)));
    proof_assert!(forall<offset: Int> covered(left, offset) == covered(right, offset));
    proof_assert!(match (left, right) {
        (Some(x), Some(y)) => {
            a_nonempty_range_is_its_offsets(x, y);
            x == y
        }
        (None, None) => true,
        _ => false,
    });
    proof_assert!(left == right);

    let same = a.intersection(&a);
    let empty = a.is_empty();
    proof_assert!(match same {
        Some(x) => {
            a_nonempty_range_is_its_offsets(x, a);
            x == a && !empty
        }
        None => empty,
    });
    let len = a.len();
    proof_assert!(empty == (len@ == 0));
}

/// `offset` moves each endpoint by the shift in the integers, clamped into
/// `u32`, and `extend` encloses the range it grows.
pub fn source_range_shifts_saturate(range: SourceRange, amount: i32, grow: u32) {
    let moved = range.offset(amount);
    proof_assert!(moved.start@ == shifted(range.start@, amount@));
    proof_assert!(moved.end@ == shifted(range.end@, amount@));
    let grown = range.extend(grow);
    proof_assert!(grown.encloses(range));
}

/// Without saturation the shift keeps the length, and the opposite shift
/// restores the range.
#[requires(0 <= range.start@ + amount@ && range.start@ + amount@ <= u32::MAX@)]
#[requires(0 <= range.end@ + amount@ && range.end@ + amount@ <= u32::MAX@)]
#[requires(amount@ > i32::MIN@)]
pub fn source_range_shifts_invert(range: SourceRange, amount: i32) {
    let moved = range.offset(amount);
    let before = range.len();
    let after = moved.len();
    proof_assert!(after == before);
    let back = moved.offset(-amount);
    proof_assert!(back == range);
}

/// A mapping carries an offset in one range to the offset at the same
/// distance in the other, clamped into it, and returns `None` outside; an
/// equal-length mapping is a bijection between its two ranges.
pub fn source_mapping_carries_offsets_between_its_ranges(
    source: SourceRange,
    generated: SourceRange,
    offset: u32,
) {
    let mapping = SourceMapping::new(source, generated);
    let forward = mapping.source_to_generated(offset);
    let backward = mapping.generated_to_source(offset);
    proof_assert!((forward != None) == source.covers(offset@));
    proof_assert!((backward != None) == generated.covers(offset@));
    let gap = generated.is_empty();
    if let Some(target) = forward
        && !gap
    {
        let distance = offset - source.start;
        let last = generated.len() - 1;
        proof_assert!(generated.covers(target@));
        proof_assert!(target@ - generated.start@ == if distance@ <= last@ {
            distance@
        } else {
            last@
        });
    }
    if source.len() == generated.len() {
        if let Some(target) = forward {
            let back = mapping.generated_to_source(target);
            proof_assert!(back == Some(offset));
        }
        if let Some(origin) = backward {
            let again = mapping.source_to_generated(origin);
            proof_assert!(again == Some(offset));
        }
    }
}
