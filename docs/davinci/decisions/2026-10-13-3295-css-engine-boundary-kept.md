# The CSS engine panic boundary stays

Issue: [#3295](https://github.com/ubugeeei-prod/vize/issues/3295).

## Decision

The `catch_unwind` boundary around the CSS engine in Atelier, Glyph and
Patina is permanent. #3295 is closed with the boundary in place.

## State of the retirement criteria

- Both crash classes are repaired in the vendored forks: `Percentage::parse`
  returns an ordinary `InvalidValue` error instead of reaching
  `unreachable!()`, and HSL conversion normalizes non-finite and out-of-range
  hues. The repairs are described in the
  [fork record](./2026-10-12-css-engine-vendor-exit.md). They are not upstream
  releases.
- The `css_parse` fuzz target no longer skips numeric or math inputs.
- The panic-matrix regression tests keep passing: shapes that used to panic
  now parse or return an error.

## Reason for keeping the boundary

Shipped tools must not abort on user input. The engine is a large third-party
parser, and the two classes above were found by fuzzing, not by review. The
boundary turns any remaining engine panic into a diagnostic instead of ending
the CLI or the language server. Its cost is one `catch_unwind` per stylesheet
and an unwinding release profile, which the editor boundaries need anyway.

## Tradeoff

A new engine panic is reported as a diagnostic and does not crash a user's
session, so it is less visible. Fuzzing and the boundary tests are the means
of finding such panics.
