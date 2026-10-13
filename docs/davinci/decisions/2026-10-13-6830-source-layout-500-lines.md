# Keep source modules readable without unnecessary splits

Tracking: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The maintainer raised the authored-file ceiling from 350 to 500 lines and
asked that a Rust module not need a directory solely to hold `test.rs` or
`tests.rs`. This supersedes the former 350-line source growth limit and the
blanket prohibition of path attributes for this narrow test layout.
Instruction-count, runtime, and memory budgets retain their existing rules.

The source-length command defaults to 500 lines. New files and growth up to
500 lines are allowed; new files above 500 and growth of already oversized
files still fail against the event's comparison base. Rename-aware accounting,
generated-file exclusions, and explicit caller-selected limits remain intact.
Workflow source-length assertions and current contributor guidance use the
same ceiling. Historical receipts continue to describe their original files.

Keep small tests inline. When a separate unit-test file helps readability,
place it next to its owner and gate its module with `#[cfg(test)]`:

```rust
#[cfg(test)]
#[path = "parser_tests.rs"]
mod tests;
```

The layout check permits only direct test gating and a sibling `test.rs`,
`tests.rs`, `<module>_test.rs`, or `<module>_tests.rs` path. Production modules
still use ordinary module discovery. Paths with directory traversal,
subdirectories, duplicate redirects, or conditional production redirects stay
rejected. Existing test trees are not moved solely to enforce this preference.

Regression tests cover the 500/501 boundary, custom limits, new and growing
oversized files, sibling test modules in both attribute orders, and rejected
production or non-sibling redirects. Existing comment/string detection and
comparison-base regressions remain enabled.
