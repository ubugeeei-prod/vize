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

## Independent object-argument correction

Independent source review rejected the first private a38a75c6 proposal: a newly collected value-context await of a bare object literal would put that object after an arrow without grouping, making it a block and losing the value. The correction carries the actual OXC ObjectExpression argument role and adds grouping only for that role; the existing parser preserves ParenthesizedExpression, so already grouped operands and ordinary direct-await output remain exact. Previously incorrect bare-object direct callbacks receive the same semantic repair; no claim of retaining those broken bytes is made.

All original fifteen complete source/control objects and their authored references are unchanged. One extra full control keeps the valid original bare-object expressions and independently pins a complete semantic reference SFC with explicitly grouped operands. JavaScript grouping preserves the operand/result. Pinned Vue3.5.38 uses the same ungrouped raw-arrow emission for the bare source, so its entire bare compile object/code/map and parsed block-callback evidence are retained separately without stock-bare runtime acceptance. Fixed Vize bare and actual Vue grouped components must produce the same independently authored whole HTML/context/warning vector. This adds six semantic render pairs to the original ninety same-source pairs; both complete sources and their map probes/identities are explicit. No original oracle is changed, filtered, or recaptured.

Current source verification remains UNEXECUTED: no install/build/native runtime, no PR/Ready/queue claim. Fresh exact successor source review, all96 real SSR pairs, complete maps/results, protected104/fullRust, signed delivery and public release replay remain mandatory.

The complete seven-field current result is compared between source-map modes and retained whole. Stock binding metadata is captured whole, but the existing DOM/SSR Vue-import classifications differ; no current-versus-stock binding byte parity or full metadata acceptance is claimed.

## Complete failing-result custody

Final harness source review found a recording-order gap: AST/map validators ran while constructing the observation, before the complete official compile result was saved. Persist the complete current, official and stock-bare objects first, then populate and validate the same AST/map facts. This preserves failing official output evidence even when a map assertion rejects it. The map validator body and every assertion are unchanged; it is extracted into a small owned helper receiving the same authenticated SourceMapConsumer constructor.

Production, all sixteen full sources, all independently authored references, runtime states, lock resolutions and prior source records are unchanged. Both helpers remain below 350 lines. No compilation/runtime/Actions/readiness credit is claimed; fresh immutable source review and the existing required/protected/delivery gates remain mandatory.
