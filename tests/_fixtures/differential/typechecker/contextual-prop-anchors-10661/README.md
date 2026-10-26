# Contextual prop anchor identity (#10661)

The original-from source is byte-identical to public repro commit
8195df603650e88e61381f4878bd8ac89ed1953a. Public vize 0.569.0 and the
published TypeScript 7.1 beta report unmapped TS2552 while the native CLI
incorrectly appears clean. The raw materialized native program independently
reports the same TS2552 with both native providers.

The keyword corpus covers all 84 keyword spellings from the pinned TypeScript
6.0.3 lexical token table. Local/import/prop shadowing, reactive destructures,
const-enum import type queries and Unicode identifiers retain lexical identity.
The genuine authored localNmae error must remain TS2552 at authored line3/column6.

All ten batch cases compare every returned production diagnostic. Because the
original native mapping drops the synthetic error, batch-clean alone is not
regression evidence: the separate required real-provider control checks complete
raw native programs and actual 7.1 beta mapper output over LF/CRLF/Unicode cases.
No diagnostic filter, source map, identifier keyword table or existing diagnostic oracle changes.

The complete anchor-list expectations are frozen from public 0.569.0 producer
declarations and unchanged authored anchors: prop references use the actually
declared identifier, and the keyword corpus covers the complete binding list.
Existing generated-code snapshots change only the reviewed anchor spellings.

The original all-84 source is retained unchanged as a known unresolved global-
undefined collision control. Its full raw TS2448/TS2454 outputs are pinned from
source 6f67317c; it is not classified as a clean keyword result. Independent
non-colliding-keyword-props covers 83 synthetic prop names, and
authored-undefined-shadow covers a hoisted var binding, whose native and mapper\nprograms are independently clean with public 0.569.0. A const binding exposes an\nadditional existing mapper preamble capture and is retained as a separate failing\npacket for #10667; this control does not claim const-shadow support.
