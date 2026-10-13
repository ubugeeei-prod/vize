# Retire the lint source join and documentation custody

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

The NoMutating component source join (`tests/tooling/support/no-mutating-component-join.mjs`)
and the attribute inheritance documentation custody
(`tests/tooling/support/attribute-inheritance-documentation-custody.mjs`) are
removed, together with the tooling tests that only ran them:

- the nineteen twelve-line callers that each re-ran the join for one historical
  source law (`no-mutating-props-*-source-custody`, `props-deep-*`,
  `reserved-*`, `setup-export-default-reference`, `component-value-*` and
  `warm-type-backed-*-host` callers);
- the join's own source and maintenance tests, and the two stack source tests
  that pinned the join's files;
- `attribute-inheritance-documentation-custody`, `public-vue-imports-explain`
  and `docs-ja-rule-editorial`, which pinned the complete bytes of the
  `vize explain` snapshots and of 317 Japanese and English rule pages;
- `runtime-prop-defaults-reference`, which pinned the bytes of other tooling
  tests (including `lint-reserved-prefix`) and ran no linter. The runtime
  defaults CLI cases stay in `lint-runtime-prop-defaults-cli`.

`lint-props-deep-destructure` keeps all 33 CLI cases and stops asking the join
for a source receipt.

## Reason

A test verifies product behaviour or an immutable past fact; it does not pin
the bytes of files that ordinary pull requests edit
([star listing record](./2026-10-13-6830-star-listing-recorded-commit.md)).

These tests compared current files against recorded SHA-256 digests. Adding a
lint rule changed the `vize explain` snapshots and failed three of them; moving
an import in a linter source failed the join and all nineteen callers; editing
a Japanese rule page failed the editorial test. Appending one newline to the
production files that the ledgers named failed 13 of the callers on its own.
None of them ran the linter: each failure meant "a file changed", which the
pull request already says.

The behaviour these laws were written for is checked where it lives: the
Patina rule tests and their fixtures, the lint CLI tests, and the
`vize explain` snapshot tests in `crates/vize`.

## Scope

The `no-mutating-props-options-uploader/source-custody` fixtures and the custody
support modules that still reference each other are left for a follow-up,
because the star listing and source counterpart tooling still read them. The
release surface custody tests stay with the release tooling.
