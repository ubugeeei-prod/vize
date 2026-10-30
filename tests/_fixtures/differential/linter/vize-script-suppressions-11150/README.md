# Vize script suppression aliases (#11150)

The three source files retain the issue-authored plain-module, ESLint control,
and SFC setup examples. `lint_prefixed_suppressions_cli.rs` consumes them with
an incremental preset enabling the reported script rule, then compares the
complete CLI JSON vector and unchanged source bytes. The unit matrix separately
pins line and region scope, enable ordering, named rules, lexical negatives,
Unicode and CRLF. CSS and template pragma grammar keep their existing tests.
