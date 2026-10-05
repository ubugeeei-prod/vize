# Nuxt bridge benchmark entry point

Issue: [#7846](https://github.com/ubugeeei-prod/vize/issues/7846).

The documented benchmark imported the bridge provider below
`tools/benchmarks/npm/`, where it does not exist. Resolve its three unchanged
exports from the actual `npm/framework/nuxt/src/bridge-fast-path.ts` at the
repository root. Leave the production bridge, generated module workload,
iteration defaults, timing code and published results unchanged.

A bounded Actions control runs the actual entry with ten modules and one pass
from an empty external working directory. It checks the complete output shape,
all three provider gates and their work counts, and the explicit unmeasured
composable stage when Nuxt's optional `unimport` cannot resolve. This is an
entry-point correctness check; its transient timing values are not a new
performance baseline or an end-to-end Nuxt build result.

The original source at signed main `ef506012169d228982ded40d893ac2115a3e58a7`
reproduces `ERR_MODULE_NOT_FOUND` with Node 24.14.0 before any benchmark stage.
The issue reporter's public identity is `ubugeeei` (GitHub ID 71201308).
Actual merge, release inclusion and the separate Nuxt build campaign remain
pending until their external results are verified.
