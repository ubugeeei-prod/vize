# Template assertion unions (#7911)

The reporter's complete `TextField.vue` contains four unions inside `as` or
`satisfies` types. The existing arrow-parameter scanner correctly handles #7235
but misses these type positions, including an assertion within a function-ref
arrow body. Keep the existing byte scanner and arrow-type spans. Only when an
otherwise suspicious single pipe remains and an assertion keyword is present,
use the existing pinned OXC expression parser to collect exact TS union spans.
Only a complete successful parse can add spans. Malformed or unsupported syntax
retains the original finding; validity alone never exempts an expression.

A runtime pipe outside those spans still produces the original whole finding,
even when the expression also contains an assertion union. Ordinary no-pipe,
existing arrow-union and JavaScript filter paths add no expression parse. No
shared pipeline stage, dependency, serialization or instruction budget is added.
Protected instruction counts remain mandatory; no speed improvement is claimed.

Preserve existing dialect admission and scanner laws, whole messages/help,
labels/fixes and authored diagnostic locations. Source-hashed reporter corpus and
complete public-result laws cover all original bindings, nested types, casts in
function bodies/interpolations, runtime filter positives, invalid/trailing syntax,
strings, the legacy raw-template constructor and UTF-8/CRLF positions. This change
grants no native-stage credit.

Source Actions, protected full fixtures/Rust suites and unchanged instruction
ceilings, actual signed merge, issue closure and published release remain required.

Corpus uses raw `.fixture` carriers with original public filenames and input
hashes unchanged, retaining repository zero-warning lint/format policy. A
preserved parenthesis wrapper forces complete expression consumption while
allowing trailing comments; union spans subtract only its one-byte prefix.

Hosted Check `37258541630` executes 1,534 Rust tests and rejects one new control:
`lint_template` with `with_vue_version(V2)` still selects `VueDialect::Vue` in
the existing implementation, so it correctly retains the complete filter finding
under that constructor. Preserve that actual historical result instead of
changing unrelated dialect routing or waiving a runtime pipe. True Vue 2 routing
through this raw API remains a separate unfinished gap. The owned Patina consumer
inventory is regenerated without a waiver. Replay the canonical decision clause
with its parent's inventory repair intact; fresh exact-head Actions is required.

Replay the same reviewed source and full-result corpus onto the genuine parent
`46fd3fcc6f29125f4382fd80d1127b10b106188b`, rooted in literal actual main
`a2712e78968e9112c89cbc2111b108bd0e51959c`. Preserve all incoming canonical
decisions and require fresh source Actions, verified native Stack positions and
protected unchanged ceilings before admitting the highest ready contiguous prefix.
