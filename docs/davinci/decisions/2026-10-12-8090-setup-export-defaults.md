# Setup-export correctness in default script presets

Decision: enable the existing `script/no-export-in-script-setup` through
`ESSENTIAL_SCRIPT_PRESETS` instead of `OPT_IN_SCRIPT_PRESETS`.
The paired decision is [#8090](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6116620684).
The source preparation starts from actual main
`47b204f26839afb27af4a2f789eafc86295f0f81`.

## Product boundary

The production change is one registry field. It enables the existing Error rule
in Essential, HappyPath (the CLI default/recommended preset), Ecosystem, Nuxt and
Opinionated. Incremental stays empty; explicit `off`, CLI preset precedence and
the separately pinned 51-rule n8n projection remain unchanged. Explicit opt-in
membership already contains this rule. No parser, AST producer, compiler-macro
ownership, source dispatcher, Canon API, query or pipeline stage changes.

The existing rule rejects runtime exports in inline `<script setup>` and keeps
ordinary module `<script>`, type aliases, interfaces and ambient declarations.
It retains its original whole-statement native spans and stored help. The
pinned ESLint rule uses the initial `export` token for exported declarations;
that independently recorded provider range difference is unchanged.

Two known false negatives remain: type-only star exports and export declarations
containing only type specifiers. Vue compiler-sfc 3.5.43 rejects these statements,
while the existing native and pinned ESLint rule allow them. This preset change
does not claim compiler parity for those cases, source-external ownership,
malformed programs, or complete default lint/project parity.

## Authored authorities and preserved evidence

The legacy differential fixture is
`tests/_fixtures/differential/lint/setup-export-default-8090`.
Its 18 independently authored inputs retain complete native API/CLI packets and
separate whole official provider/compiler packets: eight runtime findings and
ten native negative controls, including the two named compiler differences.
Pinned provider versions are eslint-plugin-vue 10.9.2, ESLint/vue-eslint-parser
10.4.1, TypeScript parser 8.65.0 and TypeScript 6.0.3. Their complete before/after
package file tables, source bodies, lock authority and licenses are inert,
authenticated reference archives. The exact official compiler-sfc 3.5.43 npm
registry response, tarball with SRI/SHA1, loaded CJS bytes, Git source and license
are retained; no archived source is imported into the product.

Four complete script-only carriers cover runtime/type/normal exports and UTF-8
with CRLF and two findings. Thirteen routes each preserve full diagnostics,
messages, severities, help, ranges, JSON/status, source bytes, config and argv:
default/recommended/Essential/Ecosystem/Opinionated/Nuxt/all/Incremental,
explicit off, both CLI-preset precedence directions, off plus CLI Essential,
and the unchanged n8n projection. API diagnostics preserve registry order;
CLI packets preserve span sorting. Opinionated/all retain the pre-existing
`script/no-options-api` Error diagnostic and its original range/order.

The initial authored CLI packets omitted stored help and the two pre-existing
Opinionated/all Error diagnostic packets. The complete initial 70-run public observation
and both initial authored bodies remain in lossless archives. The bounded
`authorship-correction.json` permits only those source-demonstrated omissions,
with five complete original producer/renderer/ordering bodies and independent
review. The succeeding public-before observation retains all 70 whole packets,
raw stdout/stderr/status and physical binary identity. It passes against genuine
v0.443.0 binary SHA256
`9ad7cf956351265a152a9bbfa08b8c50c85dadacef66529474bbb6170267e254`;
this is unchanged historical baseline evidence, not successor release credit.
The finite source-only test explicitly converts a copy of distinct-span API
findings to the existing CLI comparator order; fixtures keep both original
orders and are not rewritten in response to successor execution.

## Exact source-family integration

The original runtime defaults complete twelve-body source family, manifest SHA
`ea908f5de1c09c19ad59bf9f64bbbabccab676940f584fe754bb8e10f79828cb`,
its original whole archives and all previous 79/392/357/443 inputs are immutable.
The new complete twelve-body family is authenticated by
`source-custody/successor.json`, SHA
`7a612fc13dfe59fd50cfa31300980eacfa645e84a410cf6e04ea5777bc8d2e4b`.
All eleven other incoming bodies remain exact. Only the registry body receives
the single literal preset-field substitution; unknown bodies, missing archives,
source order, manifest identity and mixed families refuse admission. The old
API-order authority remains literal and admits this registry only after full
family authentication. Existing runtime iterator/default dispatch and both
previous source-carrier laws remain unchanged.

The whole original membership snapshot is archived. Its finite law adds the
single rule in registry order in exactly five presets and preserves all other
identities/order plus empty Incremental and opt-in membership. Generated docs
change only the preset membership of this rule; the rule examples remain exact.

The first exact-source Check `38196958922` at `46ee5aa` refused a second
historical registry caller in tooling job `114642149488`. Its entire original
bare-default reference test and complete raw failure log are retained in
`source-custody/`. The finite `bare-caller-integration.json` authenticates the
whole original/after caller and permits only an import plus an exact registry
selector through the existing complete-family guard. Original registry length
21919 and hash stay literal; every other source check and all 112 whole bare
before packets remain unchanged. This is source-custody integration, with no
new diagnostic expectation or runtime policy change. The initial ten source laws pass;
the initial failed source job and separate successful native outcome remain
distinct from a fresh successor.

The other initial source failures are preserved as complete job logs with whole
original callers and metadata in `policy-caller-integration.json`. Suppression
routing admits the new snapshot only after the complete twelve-body family and
whole five-insertion membership law authenticate; all other producer pins and
all original whole suppression packets stay literal. Two Rust negative assertions
for Opinionated/Nuxt become positive, and the rule map changes only this rule's
five sorted preset names. No other severity, membership or rule-map entry changes.

The warm host's original ten-body manifest, nine preserved bodies, helper and
real400 driver retain their exact bytes. The current-source test explicitly
requires the old host qualification to return `null` for the new snapshot;
it then independently authenticates the complete twelve-body family, the whole
membership successor, all ten original before bodies and all nine before/after
corpus bodies. This is current-source custody, not measurement qualification.
All original cohort positive, missing/unknown/mutated-body refusal and literal-cut
laws remain. No digest substitution or new performance authority is introduced.
Initial exact source `46ee5aa` completed Check `38196958922` with FAILURE;
mandatory native `38196958808` independently completed SUCCESS. Neither outcome
qualifies a successor source head. The original source CLI 70 packets remain
eligible only as their own archived source observation pending full custody.

## Qualification and remaining work

Nineteen source-only authority/mutation laws and typed tooling checks pass locally.
Rust/API 18+52 and source CLI 18+52 are required on the inherited Actions shared
builder; complete process/build packets persist before assertions. No separate
native builder, full Matrix recapture, snapshot recapture, benchmark ceiling,
provider suppression or budget relaxation is introduced.

TODO: exact published source Actions, mandatory native/performance qualification,
all protected suites, actual merge, then an authenticated fresh official release
containing that merge with installed default/config verification. Root remains
the sole official release/public dispatch operator. Historical v0.443.0 and old
source greens cannot qualify this default change. The earlier 42 false-negative
rule groups were already enabled in HappyPath; this slice does not fix the
103 failed projects or 28 unusable baselines in the retained fd4 Matrix.
#8090, broader rule coverage and real-project/default parity remain unfinished.
