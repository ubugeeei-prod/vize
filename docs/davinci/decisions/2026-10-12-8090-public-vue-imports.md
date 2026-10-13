# Public Vue import eligibility and safe fixes (#8090)

## Decision

`script/prefer-import-from-vue` must preserve named imports that `vue` does not
publicly export. Element Plus uses `isString` from `@vue/shared`; rewriting this
import to `vue` breaks its actual module contract. Keep public named diagnostics,
help, labels, byte ranges, quote and separator behavior unchanged. Offer the
existing replacement only when every imported binding is a public named export.
Namespace, default and mixed public/private imports keep their advisory without
an unsafe fix. Eligibility uses the imported identity, including aliases and
OXC-decoded string names, rather than the local binding.

The private compile-time table contains exactly the 361 unique sorted names
from eslint-plugin-vue 10.9.2 at
[`9aa463a4`](https://github.com/vuejs/eslint-plugin-vue/blob/9aa463a4aa4ad3c37d7facacda2e40ad7880ea92/lib/utils/vue3-export-names.json).
This fixed compatibility authority includes `ref` and `camelize`, and excludes
`isString`. It does not claim the newest Vue export coverage. A whole-set law
rejects omissions, additions, duplicates and order changes. All four internal
packages remain eligible. The existing shared AST and statement traversal remain;
there is no new parse, stage, public context API, dependency or serialization.

## Original source and historical failure

Retain the whole 452-byte
[`vp-api-enum.vue`](https://github.com/element-plus/element-plus/blob/20298a21ae54a5417955373b80130ae7c548a6bb/docs/.vitepress/vitepress/components/globals/vp-api-enum.vue),
SHA-256 `3e39e2d5e9c3657f7240c75e0f53acd60054a97ae3e4ff6aa6cee2185efe508a`.
Its old finding is the complete `from '@vue/shared'` clause at bytes 76–94,
line 3 columns 21–39; the intended corrected scoped vector is empty.

The frozen fd4 Element Plus Matrix packet remains whole: 9,123,302 raw bytes,
SHA-256 `e80b6ba6ce467f01da5bf9ca3d978f5a796936b51cb553379c677700a603efb9`.
It came from run 38159810046, artifact 11709971103, authenticated shard 2 archive
`7bb9ae76824189884d7a9c59aa66672c9e3b8cdf9dfcd33bf95b41697da766e7`.
Its breached verdict and both zero caps remain unchanged. No new whole Matrix
observation, full-project parity, source-budget adjustment or reduction of the
103 failed historical projects follows from this bounded correction.

## Authorship and reference custody

The [corpus](../../../tests/_fixtures/differential/lint/public-vue-imports-8090/cases.json)
was authored before reference or candidate execution: 29 complete carriers,
24 old scoped native findings and 17 intended corrected findings. It includes
the immutable original, private/public aliases, all four packages, type and
string imports, namespace/default/mixed imports, re-exports, side effects,
comments, quote styles, Unicode/CRLF, standalone script and unrelated controls.
Its SHA-256 remains
`d52480f474c2a6dba9681aa620a0c5e6dd2597accd91ad286000230a1b9d2389`.
Four minimal carriers have seven complete default/preset/config/order routes;
the independent route authority remains
`412dc8a553f58e3feda1d9ffc572cc89e5f08929d134c8943ee11a8689db888f`.
The original 51-rule n8n projection does not select this rule.

The existing authenticated v0.443.0 binary and original current-parent producer
have byte-identical whole rule source. Complete released-before packets match
29/29 with 24 diagnostics. Separately authored old `--fix` contracts match 6/6;
whole JSON, raw streams, status and final file bytes preserve proof that the old
binary physically rewrites the original/private/mixed/namespace/default imports.

Pinned ESLint 10.4.1, eslint-plugin-vue 10.9.2, vue-eslint-parser 10.4.1,
TypeScript parser 8.65.0 and TypeScript 6.0.3 retain complete package-file trees,
version/package identities, full lock bytes, primary source and MIT licenses.
The first official whole-result attempt matched 10/29: the 19 source-authored
positive messages included `nodeType`, which ESLint 10 omits. Preserve that
entire failed attempt, initial helper/primary, all 29 inputs and both original
vector files unchanged in `initial-reference.json.gz`.

Before replay, independently authenticate the whole pinned ESLint serializer:
`file-report.js` `createProblem` lines 350–387 and `addRuleMessage` lines 530–564
emit locations, message identity, severity and fix but no `nodeType`.
The separate version-specific [supplement](../../../tests/_fixtures/differential/lint/public-vue-imports-8090/provider-envelope-supplement.json)
removes only 19 `nodeType: Literal` envelope fields. No semantic or native vector
changes, version switches or disabled checks are permitted. Refusal controls
reject other field deletion/addition, range/severity/message/fix mutation and
case/source identity drift. Corrected whole reference packets match 29/29 with
19 diagnostics; the initial 10/29 failure remains visible and authenticated.

## Consumer gates and remaining work

Rust API consumers compare all fields, complete fix objects, whole applied
source bytes and fixed-source diagnostics. There are 57 input scenarios
(29 scoped and 28 minimal routes), plus 29 fixed-source whole-result queries.
The actual source-built CLI has 63 complete scenarios: those 57 plus six real
`--fix` carriers covering public fixes, unsafe alternatives and the original.
The CLI retains raw output/status/config/input/final bytes and the exact source
build receipt before assertions. The original 452-byte file must remain exact
under the corrected `--fix` consumer. Original unit tests and positive snapshots
remain unchanged. Three whole explain snapshots retain their original texts and
add only the source-authored valid private import; native whole-page tests gate
that expected change. Four local source-only reference/custody laws pass.

Candidate Rust/API/CLI execution, exact-head source Actions, mandatory native
checks, protected merge queue and actual merge are still pending at preparation.
Installed public acceptance requires a fresh fix-containing release tuple owned
by the release lane; old v0.443.0 is before evidence. No speedup claim is made.

Side-effect/re-export producer expansion, the original from-clause lexical
ownership, and newer Vue export coverage remain unfinished. Native public string
imports preserve their existing legitimate warning/fix; the pinned provider's
`.imported.name` contract omits that case. Provider/native range and severity
contracts remain separate and complete; no parity waiver is inferred. Broader
#8090/#8564 failures and the historical full Matrix remain open.

## First source Actions checkpoint

Initial exact PR head `9ae7ec8`, Check 38190285207, failed source planning before
Rust/API/CLI/tooling execution. The existing consumer ledger had two stale
earliest-import positions: both producer rows 809/810 move only 24→26. The
established Node generator refreshes those two positions; every other TSV byte,
row count, dependency kind and use count remains exact. Retain the original whole
ledger Git blob `7c5e249d81861c2257ca83cf04e8b18b6a26cb25` and the
[complete raw failed log](../../../tests/_fixtures/differential/lint/public-vue-imports-8090/initial-source-plan-failure.json),
raw SHA-256 `ef62533c7286622f10c719ba06ff49fcaefab69149aace9628e08800cf032263`.
No diagnostic fixture, budget, source guard, product source or generator changes
follow from this metadata repair. Fresh exact-head source/native qualification
remains required; the initial skipped product execution receives no credit.

The same initial Check also passed all 9,556 format files but failed the zero
warning budget on one typed `require-array-sort-compare` finding in the new
table-law helper. Retain its complete
[raw failure and whole original helper](../../../tests/_fixtures/differential/lint/public-vue-imports-8090/initial-check-js-failure.json).
Supply an explicit lexical string comparator with the same code-unit ordering;
only the active helper digest advances. Original execution/initial helper
authorities, export data and every semantic diagnostic vector remain exact.
This preparation correction requires fresh full source and native Actions.

## Second source Actions checkpoint

At PR head `4ecee412`, Check 38191191286 passes source planning, typed JS checks,
all four Rust shards and the whole CLI 63/63 scenarios, including six actual
fixes. Authenticate the real builder separately: `f253d4cbd78ff971e021204eec9c12824fdf8793`,
binary SHA-256 `94d64b92fced47fa7847100c93480c91639fef643f854d6b77c74e77ad64f4b0`.
The [complete checkpoint](../../../tests/_fixtures/differential/lint/public-vue-imports-8090/source4ece-checkpoint.json)
retains full raw63 output, original failing tooling log and exact builder custody.

Full Check remains RED: the original Japanese/English 317-page custody rejects
the one new valid private import example. Preserve that whole original authority
SHA-256 `165c6b367ca2928177f379ebbb6c8d71a7e454d585f45f826f5a5168dcc8dad3`.
The finite successor retains both prior whole pages/Git blobs and all four old
fences; independently derive the after fences by adding only `isString` in the
Valid example. Reject configuration/Bad/graph/metadata drift and keep all other
316 entries on their original complete fence gate. Original support-fact checks
still run for every entry. No original hash or baseline is overwritten.

Mandatory native run 38191191158 also fails: its driver invokes an absent
`native-lexical-dollar.sh` and exits 127. Retain the complete raw failure; the
existing CI owner must repair or incorporate the actual main helper. This slice
changes no native driver, budget or check. The 63-case source proof grants no
full-source, next-head, protected queue or installed public acceptance.
