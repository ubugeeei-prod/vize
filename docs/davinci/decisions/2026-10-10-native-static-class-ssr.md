# Static class spelling in the native SSR backend

Tracking: [#6839](https://github.com/ubugeeei-prod/vize/issues/6839),
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840), and
[#6100](https://github.com/ubugeeei-prod/vize/issues/6100).

The existing sealed, complete lower File SSR entry now accepts static `class`.
L3 records eligibility in its existing canonical attribute visit. L4 consumes
the original already-decoded attribute value and applies pinned Vue 3.5.35's
five-character HTML whitespace condensation followed by ECMAScript trimming.
Interior non-HTML whitespace remains authored class content. U+0085 is not
trimmed; U+FEFF is trimmed at an edge. Rust's Unicode whitespace predicate
cannot express this contract. Already-normalized values and ordinary attributes
keep borrowed storage; condensation alone prepares an owned CompactString buffer.

Single roots reuse actual `mergeProps` and server-renderer fallthrough helpers.
Nested and fragment elements write normalized escaped literals. Bare and empty
class spellings remain distinct on nested elements. The writer keeps each whole
original attribute span on its generated name/value links and does not mutate
the File, parse source, decode an entity twice, add a stage, or traverse ops again.

Fourteen appended original native template inputs cover root props and order,
nested ASCII whitespace, fragments, bare/empty classes, one-time entities,
unquoted values, Unicode edges and interiors, voids, and comment roots. Their
complete pinned official code, maps, options and hashes join the existing eleven
references, whose whole parsed fixture packet has an independent immutable hash.
Fresh Rust captures require exact whole-function bytes in both writer sinks,
complete source maps and original links. The existing mandatory hosted SSR
action executes all twenty-five prepared components through actual Vue SSR
under three fallthrough contexts in both development and production runtimes
and compares complete official HTML. A separate
map judge requires every segment-bearing original anchor and both UTF-8 ends;
removing class links must fail. Runtime packets retain source, module, map,
link and primary reference hashes. Missing native capture fails closed.

This is a real backend prerequisite, not original selected-SFC class completion.
The unchanged selected L2 header still refuses static class; its original
#7502 and whole-SFC refusal captures remain byte-exact. Completing that lower
provider and its product consumers is follow-up work. Dynamic class/style,
scoped-style class integration, default compiler replacement and #6880 remain
unfinished. No legacy-backed output, upstream writes, budget change or
unmeasured performance claim is introduced. Exact-source Actions, protected
full suites and actual merge are required before delivery.

Initial source `7880ca760` / Check `38025854674` passed the complete new
source-built native SSR capture/runtime/map step. Its Rust Clippy gate rejected
our standard String buffer. The successor uses L0 CompactString with a borrowed
value variant, preserving every authored input, official module, map and output
expectation. Fresh successor source/protected qualification remains required.
