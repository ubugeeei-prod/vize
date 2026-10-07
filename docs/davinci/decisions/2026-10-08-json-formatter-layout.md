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

The original report and twenty-two complete expected outputs live in
`tests/_fixtures/differential/formatter-regressions/json-layout-7928/cases.json`.
References were independently captured with cached Oxfmt 0.63.0, with
`trailingComma: "none"` for JSONC. Public API tests check complete bytes, three
passes and strict JSON value preservation; CLI tests check read-only verdicts,
written bytes and the subsequent passing check. Existing BOM and JSONC corpus
inputs remain intact while compact-output expectations follow this correction.
The existing CLI package JSON case also retains its original input and command,
with the complete compact output following the same generic JSON layout.

The complete original JSON unit-test witness remains an immutable source asset.
All twenty-two live law inputs remain original, including the minified custom
indent control; a separate expanded-space-indent case observes four-space
indentation. Six explicit current references qualify only the corrected whole
JSON outputs while preserving all original three hundred formatter carriers,
historical expected bytes, hashes and capture receipts. Historical mismatches
remain separately observable, and source/output mutation controls reject scope
or ownership drift.
The original literal pack therefore reports seventy-eight historical byte
matches and six current-reference matches, retaining all eighty-four cases.

Source Actions, protected merge-queue verification and actual merge remain
required before delivery. This change fixes the existing formatter; it does
not establish native Davinci JSON formatter support.
