# Component model modifier diagnostic anchors

Regression corpus for [#7950](https://github.com/ubugeeei-prod/vize/issues/7950).
Exact `.txt` carriers are materialized as Vue files by
`crates/vize/tests/check_canon_define_model_modifiers_cli.rs`.

The complete CLI diagnostic list must contain one TS2353 starting at `bogus`
for the default model and a multiline named model whose unknown modifier sits
between two valid modifiers. Both repaired inputs and the positive fixture must
be clean. The positive fixture includes native modifiers and an explicitly
passed modifier object so synthetic modifier mappings cannot steal their ranges.
The generator test checks exact start/end byte mappings for every generated
modifier key, including non-ASCII names, CRLF and a nonzero template offset.

This is a production Canon regression corpus. It provides no Davinci product
replacement or whole fix-history acceptance credit.
