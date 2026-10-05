# Setup-scope await context (#7972)

The setup async detector already finds setup-owned awaits inside blocks. The await rewrite currently visits only direct function-body statements, so the complete original Probe loses the active instance after the conditional suspension.

The private correction walks the same existing OXC setup parse once, collecting setup-owned AwaitExpression spans while stopping at functions and arrows. Emission reuses the established withAsyncContext/restore protocol, preserves old direct-await bytes and provenance, handles nested argument awaits, and avoids inserting a semicolon into an unbraced if branch. No new parse or pipeline stage is added.

The complete original Probe.vue is preserved (303 bytes; SHA-256 8bdddae43d782e3ef86cf7b8da7926b415dd8f1e479052e0e7d8dbbb4944c3d8). Fourteen authored controls cover branches, try/catch/finally, blocks, loops/switch, value/nested awaits, nested-function negatives and CRLF/Unicode. Full DOM/SSR SFC results with maps on/off are retained; an independently locked Vue/compiler/SSR renderer 3.5.38 compiles the whole sources and renders both actual default components for absent/true/false props. Whole HTML, warning vectors, instance identity, injection, server prefetch, parsed wrapper counts and complete map segments/exact post-await UTF16 positions are required. The independent compilers' differently formatted code/maps are retained whole, not assumed byte-identical. Existing direct-output/map and compatibility corpora stay unchanged.

Authority: [Vue 3.5.38 setup traversal](https://github.com/vuejs/core/blob/v3.5.38/packages/compiler-sfc/src/compileScript.ts) and [await emission](https://github.com/vuejs/core/blob/v3.5.38/packages/compiler-sfc/src/script/topLevelAwait.ts). The exact oracle alias is test-only and locked from official package metadata; no local install or native build is performed. Reporter identity is the maintainer ubugeeei (GitHub ID 71201308), retained for the Co-Author trailer.

Status: private source preparation only; compilation, all ninety real SSR render pairs, fresh exact-head Actions/protected104/fullRust/signed merge/public release replay are still UNKNOWN. This does not establish native compiler, Vapor SSR, hydration, useRoute/Pinia-specific integration or performance acceptance. Independent source review is next; hosted failures will retain their complete raw receipts and unchanged references.

The source built Rust-worker artifact directory contains input.json, runtime.json,
stdout.json, stderr.txt and process.json. The existing workflow includes this
directory; no workflow, gate, budget, expected output or default option is waived.
The parent build archive authenticates actual source execution separately.

TODO: independent immutable source review, whole hosted original/control receipts,
current required source checks, actual protected candidate including all 104
instruction laws and full Rust suites, signed merge/footer/issue closure, and
root-owned next-release public-payload original replay.
