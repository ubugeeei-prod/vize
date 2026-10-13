# Bare Options API keys are distinct from Vue public instance properties

Issue: [#8090](https://github.com/ubugeeei-prod/vize/issues/8090).

The official release preflight for candidate 0.446.0 failed on Real Project
Matrix [38147644073](https://github.com/ubugeeei-prod/vize/actions/runs/38147644073).
Its WaveUI report contains 83 `script/no-reserved-keys` false positives.
The existing rule removes `$` from public instance names and rejects fourteen
ordinary member spellings such as `props`, `slots`, and `emit`. Vue retains the
`$` prefix; its [pinned public instance implementation](https://github.com/vuejs/core/blob/8be32d654f5c99de2801d6815e0dc798fb1f7d00/packages/runtime-core/src/componentPublicInstance.ts)
resolves ordinary setup/data/props/context members separately. The pinned
[eslint-plugin-vue 10.9.2 rule](https://github.com/vuejs/eslint-plugin-vue/blob/9aa463a4aa4ad3c37d7facacda2e40ad7880ea92/lib/rules/no-reserved-keys.js)
checks exact [reserved spellings](https://github.com/vuejs/eslint-plugin-vue/blob/9aa463a4aa4ad3c37d7facacda2e40ad7880ea92/lib/utils/vue-reserved.json),
without stripping that prefix.

Remove only the incorrect fourteen-name bare-member deny list. Preserve the
existing `$`/`_` prefix policy, all tracked groups, severities, labels, help,
fix absence, source ranges, ordering and existing producer/dispatch path.
No new parse, semantic query, preset membership or pipeline stage is added.

## Frozen before and independent authority

`tests/_fixtures/differential/lint/bare-reserved-8090/` retains sixteen authored
source bodies and complete before/after API diagnostics, including all fourteen
names across tracked object/array groups, near names, untracked option groups,
setup locals, quoted Unicode, CRLF and mixed prefixed members. The complete
before vectors contain 137 findings; the intended after vectors retain the
same seventeen prefix findings and remove only 120 bare-name false positives.
Seven routes per case cover default, recommended, essential, ecosystem,
incremental, explicit disabling and the unchanged 51-rule n8n projection.
The API and source-qualified CLI each execute 112 corpus routes plus one
original-file route.

Pinned ESLint 9.39.1, eslint-plugin-vue 10.9.2 and vue-eslint-parser 10.4.1 pass
all sixteen complete, independently authored provider packets (nine findings).
`primary-sources.json` retains source/package hashes and complete dependency
lockfile custody. `observe-before.mjs` reproduces both baseline observers
without updating expected vectors. Authenticated public 0.443.0 passes all
sixteen whole CLI before packets (137 findings); the archive, binary, official
release/run and source hashes are recorded beside the raw packets.
The first baseline attempt incorrectly removed serialized help under
`--help-level none`; its complete failed receipt remains retained. The pinned
rule unconditionally supplies help and the JSON serializer retains it. Only
that source-derived observer projection was corrected; authored diagnostic
vectors and provider expectations were preserved.

The original WaveUI accordion API file is retained byte-for-byte from revision
`a95a3bb1d65369de9befed6341bdc56bb5de075a` (11,190 bytes;
SHA-256 `3312b54575de99e7026851bcbac68c7edef605374718ecd8a727f656e7d903d6`).
Its two original native findings cover data `slots` at 10625..10630 and computed
`props` at 10870..10875. Authenticated public-before reports exactly those
whole packets; the pinned official rule reports none. Current API/CLI gates
require no findings for this unchanged original file. The complete original
Matrix divergence packet remains compressed in the corpus, including all 83
findings; this does not claim a current rerun of every affected project/file.

The old `method_bare_builtin_name` unit snapshot encoded the false positive.
Its original source, Git blob, snapshot bytes and hashes remain retained as
historical evidence in the corpus. The active snapshot assertion becomes a
negative expectation on the independently justified bare `emit` spelling;
all positive prefix snapshots remain unchanged. Diagnostic expectations are
source-authored and never regenerated from the candidate.

## Remaining work and delivery

Patina's existing arbitrary `$`/`_` prefix policy and tracked-group policy still
differ from the pinned provider. The retained seventeen native versus nine
provider findings explicitly preserve those separate divergences. Full
24-project Matrix parity, all 83 WaveUI current findings, broader #8090 coverage
and the failed 0.446.0 release remain unfinished. No Matrix budget, corpus
requirement, snapshot gate or release qualification is reduced.

Current-head Actions must pass the new complete source API/CLI observations
and all configured checks, followed by the protected full/performance queue,
actual merge and a successful official successor release with installed
acceptance. Public-before proofs and source review grant no candidate or
release completion credit. The root delivery operator owns release triggers.

The module's invalid example now uses the actual prefixed `$emit` spelling.
Native explain loads these examples from the module source, so its three
locale snapshots change only that authored example line. The incorrect bare
`emit` snapshot remains historical evidence; no diagnostic packet is recaptured.

The exact `4dcf0e71` source CLI check exposed an observer omission: the
unchanged default `script/no-reserved-props` also reports `$props` and `$custom`
in the prefix props carrier. The original 137-before/17-after target vectors and
all pinned public/ESLint receipts remain untouched. A separate source-authored
companion freezes these two whole diagnostics, 19 combined default findings and
139 combined before findings; target-off preserves the two companion findings.
The authenticated v0.443.0 binary then matched all 112 complete default-route
packets, including incremental, target-off and n8n controls. The initial failed
64-run Actions artifact and raw job log are retained in full. This fixes the
observer's missing existing diagnostics; it changes no production rule or
preset. Current-head API/CLI and protected qualification are still required.

The subsequent full Rust API assertion exposed a second observer assumption:
template-free SFCs return the existing registry append order before the engine's
merge sort. A separate API successor preserves all target diagnostics followed
by all companion diagnostics, while CLI identity-sorted expectations and every
initial vector remain unchanged. The whole failed Rust job log is retained.
This follows the existing producer and makes no production ordering change.
