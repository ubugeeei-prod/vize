//! Whole source/mapping/link controls; native acceptance remains separate.
#![expect(clippy::disallowed_types, reason = "test-only immutable fact records")]
#![expect(clippy::disallowed_macros, reason = "whole test fact formatting")]

mod imported_props;
mod invalidation;
mod lifecycle;
mod overlay_transfer;
mod packages;
mod support;
