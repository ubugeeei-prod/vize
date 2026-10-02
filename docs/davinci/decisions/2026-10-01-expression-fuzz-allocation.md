# Malformed slash escapes in the production expression guard (#7350)

Scheduled fuzz run 36852661197 found a 1310-byte expression that allocated over
4 GiB in OXC. The artifact is preserved byte-for-byte in the owning legacy
expression corpus at `crates/vize_atelier_core/tests/fixtures/js_ts_expression_oom_7350.txt`.

The expression safety scanner treated the slash of a malformed code-position
`\/` escape as a comment opener. In the pinned OXC lexer,
`identifier_unicode_escape_sequence` consumes the character following a backslash
when it is not `u`. Consequently, the slash cannot open a comment or regex, and
the brackets/type angles following it remain visible to the recursive parser.
The existing quote/backtick neutralization now also consumes this slash before
scanning subsequent source. Literal strings, templates and real comments retain
their current handling. The existing depth and speculation budgets are unchanged.

The owning integration target rejects the exact reproducer before parsing, verifies
the unchanged full output of both production expression rewrites, and verifies no
slot bindings are extracted. Small malformed slash families and safe literal/comment
controls cover the boundary. Required source-built Actions remain the merge gate;
this product parser correction does not promote a Davinci pipeline.
