# L2 attribute token round-trip (#8628)

The retained input from [#8628](https://github.com/ubugeeei-prod/vize/issues/8628)
passes native authored-span, operation-count, canonical verifier and side-table
laws, then fails when its L2 dump is parsed. An attribute name containing `=`
prints bare; the dump reader treats that character as the value delimiter and
expects a quoted value. Filtering malformed inputs would discard the existing
total native lowering contract.

The L2 printer now quotes attribute names containing whitespace, `=` or `"`
using the existing string escaping. The reader accepts this complete quoted
name before its unchanged optional value and mandatory span. Ordinary bare
attribute bytes, values, nesting, source spans and production lowering remain
unchanged. This adds no pipeline stage or serialized level boundary.

The original artifact is preserved byte for byte as
`tests/fuzz/regressions/l1_lowering/issue-8628.input` and seeded into the existing
L1 lowering target. Its native regression retains every original span, count,
canonical verifier, scope/text/wrapper/provenance side-table and structural
round-trip assertion. The target itself and all existing verdicts are retained.

Custody: original run `38133534173`, source
`31323aaa027b48540b4cf5f80b68155b704a30fc`; the downloaded 429-byte ZIP has SHA256
`751c79e0d315ee1b436030de1f1871c5071a9e18111136aad4914aaba0c7de0b`.
The input is 1516 bytes with SHA256
`6f40100935c784e26c9f7c149c7a66055d0f86ddab3eadb2f215555b0ad2e371`,
checked by the native test. It lowers to 21 operations. No input bytes were
normalized or minimized in the recorded regression.

Local validation: the unmodified complete native replay fails at dump parse
line 24; the corrected replay passes every original assertion. Two of the
three independent owned-page tests fail before the fix and all pass afterward:
42 name/value round trips, fixed ordinary Full/Display bytes, and six malformed
quoted-token refusals. The complete `vize_l2` suite and the native retained-input
test pass on Rust 1.99.0. Source Actions, protected queue, original fuzz replay
and actual merge remain pending; this is not completion of the broader native
compiler roadmap.
