# Setup-scope await context (#7972)

`Probe.vue.txt` is the complete original 303-byte report, unchanged. The fourteen
authored controls add if/else (including unbraced statements), try/catch/finally,
bare blocks, loops/switch, value and nested awaits, nested-function exclusions,
and a CRLF/Unicode source. `expected.json` was authored from those sources and
the Vue context contract before executing the changed compiler.

The Rust integration compiles all fifteen complete SFCs for DOM and SSR with
maps enabled and disabled, comparing every public result field except the
additive map. One Node child retains the full current results and independently
compiled Vue 3.5.38 code/maps. Both actual default components are rendered by the
real pinned SSR renderer for absent/true/false props (ninety render pairs), with
whole HTML and complete warning vectors compared to the authored reference.
Instance identity, injection and a hook registered after suspension are visible
in the full HTML. Actual parsed modules also prove the expected wrapper count
and that four authored nested async-function awaits stay untransformed. Every
map segment is retained and checked against complete UTF16 source bounds.

The lock adds the exact Vue 3.5.38 oracle as a test-only alias, authenticating
Vue/compiler/renderer identities. No local install or compiler build is used.
Raw input, both complete compiler objects, maps, runtime rows, child streams and
process status are saved beneath the existing Rust-worker artifact directory.
This tests the retained whole-SFC entrypoint and real Vue SSR runtime; it gives
no native compiler, Vapor SSR, hydration, Router/Pinia, or performance credit.
Hosted source/protected qualification and public release replay remain required.

The added `object-expression` control retains a bare object operand for Vize and
a separately authored, explicitly parenthesized complete SFC for stock Vue.
Those JavaScript operands are semantically identical. Vue 3.5.38 has the same
raw-arrow ambiguity for the bare input: retain its complete compiled object/maps
and actual parsed block callbacks, granting no stock-bare runtime acceptance.
The fixed bare and stock grouped components must match the same authored whole
HTML/context/warnings. The original fifteen sources/references remain exact;
this separate semantic pair adds six render pairs (ninety-six total).

The complete seven-field current result is compared between source-map modes and retained whole. Stock binding metadata is captured whole, but the existing DOM/SSR Vue-import classifications differ; no current-versus-stock binding byte parity or full metadata acceptance is claimed.
