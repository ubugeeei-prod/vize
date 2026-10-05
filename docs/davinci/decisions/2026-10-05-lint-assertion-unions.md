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

Preserve Vue 2/petite-vue gating, all original scanner laws, whole messages/help,
labels/fixes and authored diagnostic locations. Source-hashed reporter corpus and
complete public-result laws cover all original bindings, nested types, casts in
function bodies/interpolations, runtime filter positives, invalid/trailing syntax,
strings, Vue 2 and UTF-8/CRLF positions. This change grants no native-stage credit.

Source Actions, protected full fixtures/Rust suites and unchanged instruction
ceilings, actual signed merge, issue closure and published release remain required.

Corpus uses raw `.fixture` carriers with original public filenames and input
hashes unchanged, retaining repository zero-warning lint/format policy. A
preserved parenthesis wrapper forces complete expression consumption while
allowing trailing comments; union spans subtract only its one-byte prefix.
