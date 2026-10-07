# Cross-file SSR callback execution contexts

This pairs the decision on [P0 #7908](https://github.com/ubugeeei-prod/vize/issues/7908#issuecomment-6047507178).
It changes legacy `croquis/cf/browser-api-ssr` classification, without replacing
its raw browser-use producer, source-origin mapping, severity or code.

## Execution proof

Keep the original HomeButton, GuardedWatch and GuardedEffect heredocs and their
original opinionated CLI invocation byte for byte. An event-only handler and
its directly called helpers execute on the client. A helper also used by setup,
a bound template expression, an unknown callback, alias or value export retains
its SSR warning. Resolve references to authored declaration identities; equal
identifier spellings, hoisted declarations and shadowed imports cannot confer
client ownership. A recursive call graph requires a real client root and no
server root.

Use the existing script AST walks and binding-occurrence packet. Record private
function, call, callback and guard facts, then resolve them once at the existing
cross-file boundary. Recognize the actual Vue `watch` import, including aliases
and imports in the other script block. Its callback is client-only when it is
non-immediate and not `flush: 'sync'`. Its source getter still runs in setup.
Unknown options, spreads and ambiguous declarations fail closed. Named helpers
passed to an authored Vue client lifecycle hook receive the same ownership
proof. The older inline lifecycle policy remains in place.

An explicit `if` guard can prove the client branch through
`!import.meta.env.SSR`, `import.meta.client`, or an unshadowed
`typeof window !== 'undefined'` test. An unconditional return or throw from the
server branch can prove the following statements in that same body. Do not
extend that proof across a nested function or into a separate finally block.
Missing or refused script ownership packets prohibit callback and function
exemptions; an import-only or invalid sibling block cannot silently disappear.
Complete packet capture is demanded only by full analysis that collects template
expressions, or by an existing explicit packet caller. Preserve the original
ordinary lint empty-capture law and compiler no-capture path unchanged.
Skip the new reachability query when the unchanged raw producer has no script
browser use; template events keep their separate source-kind check.
These facts are private and do not change Croquis debug snapshots or add a
public serialized field, parse pass, compiler stage or level serialization.

The help now names all three documented guard forms:

```text
Wrap in onMounted() or guard with !import.meta.env.SSR, import.meta.client, or typeof window !== 'undefined'
```

## Corpus and history custody

The whole issue body and all three original input hashes are pinned in
`crates/vize/tests/fixtures/issue-7908/`. The 38 complete SFC inputs include
positive SSR warnings, both branches and return directions, watch scheduling,
source getters, shadowed and forward declarations, helper escapes, split script
blocks, exported helpers, and guarded try plus unguarded finally reads.
Source-built tooling checks all 76 enabled/disabled whole JSON results, process
status and stderr, immutable input bytes, the complete default-help response,
and the actual original plain command. The plain command retains its entire
raw output but only claims the issue's authored absence of cross-file warnings;
unrelated opinionated diagnostics are not an invented output oracle.

Native producer checks compare every diagnostic kind, API, context, severity,
file, source, offset, related location, message and suggestion. Doctor checks
all 38 physical source coordinates. The original #7907 corpus remains exactly
450 lines and retains all eleven complete input files and historical results.
Only its PageTitle non-immediate watch result receives a separate, checksum
pinned current reference, with a required inequality from its old warning.
All other ten enabled results and eleven disabled results remain unchanged.
The entire original producer-law source is archived; its live laws change only
the intended suggestion literal. Forged current and historical references are
rejected explicitly.

[#7907's source-coordinate contract](https://github.com/ubugeeei-prod/vize/issues/7907#issuecomment-6046928795)
was already delivered by the signed #8070 merge and is retained on signed
`3e0745b6277c00176178de6e9c15de2ee500a258`. It is closed on that original
contract; this does not claim that an unprovided uncaught-error reproducer,
the new callback classification, or public release delivery is complete.

## Qualification and remaining work

The same complete public native test fails on signed pre-fix main `3e0745b6`
at the original HomeButton window read, preserving the entire old warning.
The final 38-input current producer test and both preserved origin laws pass
locally. Narrow strict Clippy, source formatting, the consumer inventory,
350-line limits, authored-corpus custody and independent source review pass.
These local results do not qualify a hosted CLI binary or a future merge group. Full CLI/Doctor runtime, exact-head Actions,
all unchanged protected suites and 104 instruction budgets remain required.
The first hosted source run rejected the separately generated Croquis consumer
ledger: the new public boundary test adds one real Croquis type-consumer site.
Regenerate its whole owned shard with the existing generator; preserve all
other nineteen ledger files, every corpus byte and all gate assertions.
The next hosted run exposed the ordinary lint empty-capture law and strict
test-target lint errors. Narrow full-analysis demand rather than altering that
old law; make new test helpers use checked ranges and explicit Results, retain
every original assertion, and await all Node test registrations. All original
462 Croquis laws, complete38 producer replay/two origin laws and strict
test-target Clippy pass locally after these corrections.
[The following hosted correction](https://github.com/ubugeeei-prod/vize/issues/7908#issuecomment-6048132969)
also exercises the original SFC disabled-script law:
automatic ownership capture must require `analyze_script` as well as full
analysis demand. Keep that entire law unchanged and add its existing disabled
gate to both constructors. Replace the new producer test's partial browser-name
assertion with exact authored identifier-token validation; retain all 38 whole
diagnostic vectors and the committed assertion allowlist unchanged. Hosted
requalification, rather than a new heavy local build, verifies this correction.
No performance improvement is claimed from local results. Actual queue merge,
signed main, release and public distribution remain root-owned acceptance.

## Protected instruction regression and explicit demand

[Paired issue decision](https://github.com/ubugeeei-prod/vize/issues/7908#issuecomment-6048944211) records this correction.

The first protected candidate `24e5aea5` failed five genuine Croquis full-analysis
ceilings in job `113062557255`: small 193881/184254, large 3002042/2786473,
stress-deep 269206/263737, stress-wide 653599/626201 and stress-interp
1475400/1320076. The immediately preceding `7ceb8747` job `113061732264`
passed all 100 level and four formatter ceilings. Retain both complete raw logs
and measured artifacts; the red candidate was removed from the queue.

Automatic full-mode occurrence capture made ordinary compiler/type-checker
analysis allocate and resolve facts that only a cross-file diagnostic requested.
Restore demand-only Drawer construction and retain inherited complete packets.
The actual cross-file SFC, Doctor, raw-module and native producer adapters use the
existing occurrence-capture APIs explicitly; new SSR AST note helpers also
require that demand. This adds no parse, compiler stage, public option or
serialization. Keep all 38 whole vectors, the complete 450-line original corpus,
all original laws, benchmark inputs and 104 ceilings unchanged. The new full-mode
law verifies absent ordinary capture and equal complete public outputs under
explicit capture. Fresh hosted source/runtime and standalone instruction
measurements, followed by a newly composed protected candidate, remain required;
the repaired instruction counts are unmeasured until those Actions complete.
