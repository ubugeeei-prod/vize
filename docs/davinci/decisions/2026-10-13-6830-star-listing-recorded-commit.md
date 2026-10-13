# The star listing ledger authenticates its recorded commit

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

`tools/support/compat/github/empty-vue-star-listing-source.mjs` reads the files
its ledger names from the main commit that merged the ledger
(`53db87849ea0d4d5d60d95f87acf4088566337b2`, #8809) instead of from `HEAD`. It
still checks that the working tree matches `HEAD`, and that its own executing
files are committed.

A test verifies product behaviour or an immutable past fact. It does not pin
the bytes, line numbers or changed-file count of files that ordinary PRs edit.

## Reason

The ledger named 479 files and required each to have its recorded hash at
`HEAD`. They include 45 files under `crates/vize_canon/src`, the Croquis and
SFC analysis sources, `crates/vize_patina/src/linter/engine/rule_sets.rs`,
`crates/vize_canon/Cargo.toml`, two workflows and the decision foundation
record. Any PR that changed one byte of any of them failed the tooling tier in
the merge queue unless it also regenerated the ledger, which made every two
such PRs conflict.

Appending one newline to the 1,949 editable files that hash ledgers name made
72 of the 323 locally runnable tooling tests fail. With this change the star
listing tests pass under the same edit.

## Scope

This removes the `HEAD` pin of one ledger without deleting it: the lexical
dollar evidence step of the native phase workflow still reads through it.
Retiring the ledger files themselves, and the other byte-pinning tests the
probe found, is tracked on #6830.
