# JSON formatter layout (2026-10-08)

Issue: [#7928](https://github.com/ubugeeei-prod/vize/issues/7928).

The existing product JSON/JSONC formatter retains whether an object has a
newline after its opening brace, and retains one blank line between members
or elements. Empty collections stay compact. Nonempty source-expanded objects
stay expanded; other collections flatten when they fit at their actual output
column under the existing `printWidth` and `bracketSpacing` settings.

Fit checks include indentation, member prefixes, following commas and inline
block comments. Display width uses the existing workspace `unicode-width`
dependency, and tabs use the configured `tabWidth`. Broken numeric-only arrays
fill each line greedily, restarting after an authored blank line; comments
disable numeric fill. Comments retain adjacent blank lines rather than moving
them to the other side of a comment. Existing strict input validation, JSONC
trailing-comma removal, scalar token text, key order, BOM and line-ending policy
remain part of the public contract.

The original report and twenty-one complete expected outputs live in
`tests/_fixtures/differential/formatter-regressions/json-layout-7928/cases.json`.
References were independently captured with cached Oxfmt 0.63.0, with
`trailingComma: "none"` for JSONC. Public API tests check complete bytes, three
passes and strict JSON value preservation; CLI tests check read-only verdicts,
written bytes and the subsequent passing check. Existing BOM and JSONC corpus
inputs remain intact while compact-output expectations follow this correction.

Source Actions, protected merge-queue verification and actual merge remain
required before delivery. This change fixes the existing formatter; it does
not establish native Davinci JSON formatter support.
