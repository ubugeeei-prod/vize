# Complete licensed n8n lint observations

Tracking: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).

The read-only upstream contract is still
`5c2a2cf3d837b6538a3330a4d7cc817e4b6f04e2`. Its shared CLI configuration was
rechecked on 2026-10-16: SHA256
`d7b32f5a7bbf42ba301220b2ffee9a1dc372c252ab062e2f71ae3790149e46b5`
matches the existing owned projection. Upstream remains strictly read-only.
Its non-master source is requirement evidence, never executed or redistributed.

## What the existing campaigns establish

The final source campaign
[38895810920](https://github.com/ubugeeei-prod/vize/actions/runs/38895810920)
succeeded at `bfc2b3d41ab4ded617e4bb80784d1e67b9e3e944`. Its native CLI SHA256
is `ec7ec0ff2fa9f7bf97704bfbfbf3a0713f97b45b81bb2cf48365b3d4210b20ab`.
CLI artifact `12043313891`, digest
`sha256:c2f97e296e83b192679a6cd09e7c0a2b4f89d33345e3e8598f4d9aaa72520e7f`,
retains the licensed 1,369 SFCs, 19 scriptless inputs, nine package roots,
configuration/scope transformations and original 28 plus additional 15 and 18
independent authored controls. Its whole baseline CLI replay reports 52 errors;
those observations are not replaced by the narrower historical report below.

The older `n8n-lint-parity.ts` report from that same run says shared=18,
native-only=0 and ESLint-only=2. It compares file/line/rule projections, uses
different casing/registration options, substitutes a router manifest and does
not retain full foreign packets. Its broad documented recursive-component
exception also omits ranges and multiplicity. Preserve this as historical,
limited research; it is not full-51 or whole-packet parity authority.

## Observed result on 2026-10-17 JST

The observer was merged in [#9248](https://github.com/ubugeeei-prod/vize/pull/9248).
The single ordinary-main campaign
[38928976908](https://github.com/ubugeeei-prod/vize/actions/runs/38928976908)
ran both producer and observer at
`f21a8e0d783be6325a84640d844a7c99f1586e77`, a verified signed merge commit.
The source CLI reported `vize 0.457.0`; its binary SHA256 was
`0e788a281d0d9b3865562213c678f1dffd5973fff0b5549e9bbdba9880604ec1`.
This is a source-built result, not acceptance of a released npm package.

Installation, hydration, build, config packaging and observation succeeded.
The result has `captureComplete: true`, 1,369 original SFCs, 19 scriptless
inputs, nine package roots, 36 native and 36 independent observations,
5,476 complete file packets per provider and 18 comparisons. Both repetitions
were stable and the capture failure list was empty. The retained copies of
all 1,369 sources, two licenses and nine real package manifests also passed
an independent SHA256/length check after download; source/license Git blob
identities matched their original inventory.

The strict comparison failed with **178 difference entries**, not 178 distinct
product bugs. Baseline and scoped modes each have 18 matched findings, 34
native-only entries and 55 foreign-only entries:

| Boundary                                                             | Native entries per mode | Foreign entries per mode | Difference entries across both modes |
| -------------------------------------------------------------------- | ----------------------: | -----------------------: | -----------------------------------: |
| Same registration diagnostic identity, different physical range      |                      34 |                       34 |                                  136 |
| Self-recursive `defineOptions` component reports                     |                       0 |                        2 |                                    4 |
| Foreign unknown-rule definition errors from TypeScript rule comments |                       0 |                       19 |                                   38 |

The 34 paired registration findings agree on file, rule identity, severity and
multiplicity. Native spans cover the component name; the independent spans
cover its opening tag, including attributes. They cover 23 `RouterLink` and
11 `RouterView` occurrences. This is an unresolved diagnostic range contract,
not evidence of 34 missing findings. The full physical-range gate remains red.

The two other Vue findings are in the licensed design-system
`N8nDropdownMenu/DropdownMenuItem.vue` at lines 401 and 446. The independent
`vue/no-undef-components` rule reports recursive `N8nDropdownMenuItem` uses
despite `defineOptions({ name: 'N8nDropdownMenuItem' })`; native emits neither
report. Preserve this as a separate self-resolution disagreement. It is not
automatically a native false negative or a waived provider exception.

The 19 remaining findings refer to ten `@typescript-eslint/*` rule identities.
Every message says that the rule definition was not found. The source comments
name legacy rules that are not registered in this declared Vue-only provider
environment. These are retained provider administration errors, not evidence
that Vize is missing 19 configured product rules, and not type-aware script
acceptance. They remain in the original packets and strict difference count.

The actual independent versions were ESLint 10.4.1, eslint-plugin-vue 10.9.2,
vue-eslint-parser 10.4.1 and @typescript-eslint/parser 8.65.0; `providers.json`
retains their entrypoint and manifest hashes. Complete evidence is in
[artifact 12058042908](https://github.com/ubugeeei-prod/vize/actions/runs/38928976908/artifacts/12058042908),
digest `sha256:ab3e3eda6574b25b470f6bba87dcea8731040d1db5d31bccdc053e0d8b6573e9`.
The original packets and failed gate have not been rewritten or rerun.

### Next minimum acceptance

1. Author registration range controls with a multiline opening tag, attributes
   and multibyte text before the component, plus registered/clean, suppression
   and noncode controls. Execute the whole 51-rule source CLI and independent
   profile, retaining both complete envelopes. Establish the product range
   contract before changing diagnostic spans; do not normalize away the 34
   pairs or rewrite the existing authored packets to obtain parity.
2. Isolate self-resolution with an explicit `defineOptions` name different
   from the filename, filename-derived recursion, an imported component and an
   actually undefined alias. Check Vue's real self-resolution as an independent
   semantic authority before choosing a product fix. Retain the foreign
   disagreement and all configured-rule findings.
3. Isolate a legacy TypeScript disable comment beside a registered Vue-rule
   suppression, preserving both provider results and original comments. Any
   future provider-environment qualification needs its own real registrations
   and identity receipt; do not add stubs, erase comments, filter these errors
   or reinterpret this completed campaign as green.

These are bounded acceptance inputs, not a request for another 1,369-SFC
campaign. The separate script-rule controls in #9275 require their own actual
source receipts. Neither this observation nor those controls transfer credit
to all-rule positive/clean coverage, public installation or full-monorepo,
editor and performance acceptance.

## Remaining adoption frontiers

| Frontier                          | Existing bounded evidence                                                              | What remains unproved                                                                                                                                                                                    |
| --------------------------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Configured 51 rules               | Exact profile and authored 28+15+18 inputs; complete 1,369-SFC observation above       | Independent positive/clean controls for every rule; agreement on the retained findings and range contract; native suppression/fix parity                                                                 |
| Public configuration and delivery | Source-built pure `vize/config`; bounded installed consumers on public 0.440.0/0.442.0 | Current fixes on freshly installed public artifacts; actual upstream shared package is not replaced by the owned functional projection                                                                   |
| Licensed monorepo                 | Pinned master sources, before/after compiler and bounded source editor/CLI campaigns   | Genuine installed dependency graph, whole-project compiler/typechecker/vue-tsc/formatter and editor lifecycle comparisons                                                                                |
| Historical Oxlint route           | Separate batching/cache and ignore-pattern product fixes; their issues are closed      | Exact n8n route acceptance for invalidation, script/template positions and unchanged existing script spans; no transfer of CLI-only evidence                                                             |
| Type-aware SFC scripts            | Upstream explicitly records this capability regression                                 | Genuine independent type-aware `.vue` script coverage, beyond the selected template rules                                                                                                                |
| Retired rules and local plugins   | Frozen eight historical omissions and upstream local-plugin requirements               | `comment-directive`, `jsx-uses-vars`, `no-deprecated-v-is`, `valid-v-is`, `valid-v-pre`, `valid-v-else-if`, `no-parsing-error`, `no-deprecated-filter`; macro-variable and design-system plugin behavior |
| Performance and stability         | Raw observation timings and upstream's descriptive speed table                         | Matched input/config/dependency/toolchain LSP, checker and lint timing/stability experiments                                                                                                             |

Tracking issue #8142 was closed by the user on 2026-10-16 at 12:14:35 UTC;
that closure is not acceptance evidence. Closed issues are not reopened merely
because a broader acceptance frontier remains. This complete observation does
not qualify any of the other frontiers by itself.

## New observation contract

The manual `n8n whole corpus observer` workflow checks out one exact source head
for both native producer and observer. It uses the normal cache trust policy,
one source build and its verified build receipt, the source-built public config
export, and the already pinned independent ESLint/Vue/TypeScript providers.
There is no alternate-source cache bypass or local Cargo build.

Both baseline and scoped configurations run twice over every original input.
The observer retains 36 native process receipts and 36 independent provider
observations: 5,476 complete file packets per arm. It also retains all original
source and license bytes, the nine actual licensed package manifests, public
config assets, exact effective per-file rule configurations, module/manifest
hashes, raw stdout/stderr/status/signal/errors, all clean packets, full messages,
suppressed messages, fixes, suggestions and provider envelope fields.

The independent provider uses all 51 explicit identities and options, including
`registeredComponentsOnly: true`, and the official `flat/base` processor with
its two administrative rules. Those extra identities and findings remain
visible. The editor warning retains `always`; each of the six literal entries
changes only its named rule. No router/dependency stubs or rule-status/preset
filters are introduced. Project dependencies are not installed in the copied
source observation workspace, so it grants no full-monorepo dependency credit.

Raw provider packets are saved before validation. A named sidecar changes only
known absolute workspace file paths. A separate derived multiset compares
identity, severity and complete physical byte spans without dropping unknown
identities, missing coordinates or duplicate findings. Native scalar and
Vue ESLint UTF-16 columns are converted against original source bytes. Both
providers use LF-based lines, independently checked for template and script
diagnostics; supplementary characters and CRLF/CR/Unicode separators are tested.
It does not
equate the two tools' different human messages or diagnostic-envelope formats.
The raw formats and all their additional fields remain available for review;
this derived comparison is not a claim that foreign fix/suppression metadata
or every rule's semantics are equivalent. Native CLI JSON currently does not
expose fixes or suggestions, so independent provider fix metadata is retained
for review without claiming native fix parity.

Capture completeness fails on missing/duplicate inputs, provider execution or
fatal errors, parse failures, source mutation or unstable repeated whole packets.
Unknown-rule definition diagnostics remain visible comparison findings, as the
observed result above demonstrates. All research
artifacts are uploaded before a separate strict semantic comparison gate.
Differences fail that gate with no blanket exception list, and remain failing
research until independently reviewed and fixed. Successful capture alone is
never a parity or adoption completion claim.
