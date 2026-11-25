# Patina Rust API compatibility case

This case registers the four deprecated IR getters restored after #13674.
It does not claim SFC diagnostic equivalence or whole-language/native parity.

The admission-time source/support pairs retain exact SHA-256 and Git blob
identities. Node controls authenticate those immutable historical objects;
they do not pin later implementation bytes. A fresh registration still checks
the actual candidate base/head files, and a historical case cannot excuse a
later legacy-source change.

Source-pair digests are historical record-only custody entries. The immutable
Rust/data inputs remain current compiled/runtime compatibility oracles.

The required Patina Rust cohort compiles the current Rust input at
`crates/vize_patina/tests/native_ir_accessor_compat.rs` through ordinary sibling
module discovery in its explicit integration target. It checks all four shared-receiver signatures and an owner borrow that
outlives a moved/dropped canonical wrapper. The existing moved-owner unit and
real public template-rule callback consume `cases.json` and the fixture consumer
to compare the old/new getters' pointers on actual owners. Execution evidence
comes from those current Rust test results, never from Node source inspection.
