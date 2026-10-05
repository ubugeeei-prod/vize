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
