# n8n adoption requirements (2026-10-07)

Tracking issue: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).

The maintainer prioritizes n8n compatibility first, completion of every
acceptance criterion in the 61 original P0 delivery issues, sustained LSP and
typechecker performance/correctness/stability, and frequent successful releases.
Work proceeds in isolated `wt` worktrees and independent agent lanes. Dependent
slices use native GitHub Stacks; green ready prefixes and independent PRs enter
the protected merge queue. Track checks, actual merges and public releases
concurrently without leaving PRs unattended. This issue must stay open until
its acceptance requirements are met; fixture registration alone does not close
the adoption umbrella or any original P0 issue.

The upstream [adoption spike](https://github.com/n8n-io/n8n/pull/40393) is still
open at audit revision `aa173be0c65c0646a7fcec32d2c18e1eaacbc8ff`. Upstream is
read-only: no comments, issues, pull requests, pushes or other state changes.

The public submodule pins licensed master revision
`e882e8a483f433facb47bab9b407d0ec00a81172`. Its `LICENSE.md` excludes branches
other than master; enterprise source has separate development/testing terms in
`LICENSE_EE.md`. Preserve both license files and keep the fixture as a gitlink.
The adoption PR is requirement evidence, not the fixture revision or a merged
adoption claim.

`tests/_fixtures/n8n-adoption.json` records the exact 51 Vize rule settings and
options, package warning/override scopes and eight retired rules from the
adoption revision. It also records the actual master corpus: 1,369 Vue SFCs,
including 19 without script, across all nine Vue package roots. This differs
from the spike's reported editor-ui count and 11 scriptless files; do not
transfer those numbers between commits.

| Acceptance requirement         | Required evidence                                                                            | State                                                                     |
| ------------------------------ | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| Complete application fixture   | Pinned gitlink, all `packages/**/*.vue`, exact package/scriptless inventory                  | Pinned inventory passed; acceptance unfinished                            |
| Custom rules and options       | All 51 rules, casing/order options, editor-ui warning and six scoped overrides               | Frozen; source plugin acceptance pending                                  |
| One native call per file       | Custom rule map batching, override/config/source invalidation, matched timing                | Pending [#8144](https://github.com/ubugeeei-prod/vize/issues/8144)        |
| Accurate diagnostic locations  | Real template coordinates and valid Oxlint script positions; unchanged existing script spans | Pending exact-source adoption replay                                      |
| Scriptless SFC coverage        | Every scriptless input gets template checks without shifting script diagnostics              | Pending exact-source adoption replay                                      |
| TypeScript template unions     | `el as Element \| null` has no Vue 2 filter false positive                                   | Current fix exists; adoption replay pending                               |
| Config inheritance/suppression | Explicit per-package settings, scoped overrides and actual `.mts` discovery                  | Pending [#7903](https://github.com/ubugeeei-prod/vize/issues/7903) replay |
| Type-aware Vue coverage        | Whole SFC script/template diagnostic parity                                                  | Unfinished: spike retains ESLint for type-aware `.vue` rules              |
| Retired rule coverage          | Independent oracles for all eight omissions                                                  | Unfinished; preserve the recorded gap                                     |
| Delivery                       | Exact-head source Actions, protected merge and verified public package                       | Pending                                                                   |

The existing Real Project Matrix automatically includes the new registry row
in compiler, linter, typechecker, formatter, highlighting and LSP lifecycle
shards. The targeted `n8n adoption fixture` Actions job hydrates n8n and fails
on missing or changed corpus. Generic matrix coverage and fixture inventory do
not establish custom plugin parity or a speed improvement. n8n dependency
installation, vue-tsc parity and authored LSP oracles remain follow-up work;
no typechecker performance baseline is claimed by this registration.

The dependent authored-editor slice copies three complete, pinned master source
files (NullEmptyCellRenderer, BlockUi and its actual TypeScript barrel) into a
disposable project. It prepares exact template definition/reference packets,
component navigation through the barrel, an unsaved missing-property diagnostic
and a clean diagnostic/navigation restore. A dedicated Actions job requires the
current-source build receipt and real typechecker; missing dependencies fail
closed. These assertions are prepared and unexecuted until that job passes.
The generic n8n matrix row still has no full authored LSP lifecycle oracle or
whole-monorepo vue-tsc baseline; this small test grants no broader coverage or
speed credit.

The source tooling dependency unblock is [#8148](https://github.com/ubugeeei-prod/vize/pull/8148).
It changes the SDK cohort independently; its success does not qualify the
fixture, bridge, protected merge or installed-package acceptance by itself.

The unchanged n8n corpus exposed nine complete-template differences and two
UserSelect production-module differences at fixture source
`72aac62d7c2932c71075d579c76459b270d909da`; its compiler gate remains failed.
The existing fixture PR retains all original files and the original 16-error
allowlist and ceilings. Repair qualification requires the whole 43,000-plus
template corpus, complete production code/maps/results, and authored runtime
behavior. A matching result from two changed producers alone is insufficient.

The before/after job authenticates an immutable capture-only ancestor against
that original source and a reviewed immutable repair anchor that genuinely
descends from the capture ancestor. The baseline's production and dependency
bytes stay identical; both phases use identical capture drivers, input hashes
and build recipes. The candidate's exact source tree, toolchain and binary hash
remain recorded. Baseline-to-candidate ancestry is reported honestly because
protected squash merges need not retain it; baseline-to-repair-anchor ancestry
must pass. Complete raw packets are saved before parity/runtime assertions, so
a failed gate retains its evidence and receives no completion credit.
The immutable capture-only commit is
`f5699c195e627ad26aa1c8efd4a59266120cd9b9`; the reviewed descending repair
anchor is `311e3c8a0e1a42d91381ba3769ef9fcfc6c1f826`. Preserve their real
commit chain when integrating the existing PR. The ordinary default-mode
baseline has two complete legacy errors with missing locations; retain those
`null` locations as failed historical evidence. Its two native diagnostics
already have byte spans. The official per-prior-branch law expands their full
wire metadata to three without dropping stage, severity, parts, witness or
debug values; current legacy locations independently match the frozen official
UTF-16 positions converted to UTF-8 spans.

The failed first source `744885f1b78f744db6b5772d6bf5ed1d335272ea`
and its baseline `1f7b027ab26e2c031592f3c8546ea6fa7872eeeb` and repair
anchor `ac09d4d3624d3e0fe5b1852300d9860af2172ba3` remain historical evidence.
[The failed before/after run](https://github.com/ubugeeei-prod/vize/actions/runs/37607203124)
retains artifact `11475138675`, including complete before-phase compiler stderr.
The old baseline's example could not convert `SfcError` to `Box<dyn Error>`;
no original before/after modules or runtime results were produced. Its new
capture-only successor propagates the complete parse error through explicit
conversion and is merged into the actual current chain. Separate current-source
repairs borrow the retained allocator, remove extracted native imports and
annotate trusted compiled test-module evaluation consistently with the existing
runtime harness. All thirty-four capture-only paths, thirty-three identical
driver files, original inputs and protected ceilings remain unchanged in scope.
Fresh source compilation, before/after capture and behavior qualification are
required; the failed run provides no behavior or completion credit.

[The next failed custody run](https://github.com/ubugeeei-prod/vize/actions/runs/37608888161)
retains complete packets in artifact `11477270053`. Its before phase genuinely
compiled `67e7d0f1392d8bdd08156ac4de900cf2065c98fd`, but the after command
reused the same executable from a shared Cargo target directory without
recompiling. Both binary hashes are
`8688996e9a9dab6de04f18ac16467b560878cb16f3b0509818a6ee1ecf37c1b0`;
the alleged current native packets retain old behavior and receive no current
source credit. The unchanged complete diagnostic comparator rejected them.
Each phase now uses its own source-revision-bound Cargo target in a unique
runner temporary directory, outside the uploaded packet root. The same locked
recipe explicitly binds the source manifest and working directory; each receipt
retains those exact arguments, source/tree, manifest/lock/toolchain and actual
producer binary hashes. Compiler cache reuse does not share target fingerprints.

The new capture-only successor moves the unchanged example into its ordinary
module directory and derives TypeScript mode from both original script blocks.
Both legacy/native template modes record that flag; the judge checks it against
the independently parsed pinned stock descriptor, rejecting missing or false
plain-JS flags. Historical plain-JS packets remain raw evidence. All thirty-four
capture paths and thirty-three identical drivers remain explicit. All seven
shared-support targets declare their existing differential feature. All four
new n8n regression targets run explicitly in one hosted invocation; default
feature skips earn no qualification. Ordinary module layout, byte-wise digest formatting, an equivalent
exemption predicate and the [reviewed storage inventory](./2026-10-07-n8n-native-key-storage.md)
preserve all input bytes, diagnostic metadata, assertions and instruction caps.

The actual historical default-slot runtime failed before rendering default
content: stock `createSlots` received an undefined descriptor, yielding five
cumulative `TypeError` notifications and empty trees across the render/update
phases. Preserve that complete failure vector, its source/module/packet hashes
and the separately executed baseline-native trace. The unchanged current and
stock six-phase golden still requires complete trees, keyed identity, one
iterator call per render, updates, toggles, empty diagnostics and unmount cleanup.
The before failure earns no successful runtime or performance credit.

[The third custody run](https://github.com/ubugeeei-prod/vize/actions/runs/37612980317)
retains complete source packets in artifact `11478485937`, using historical
anchor `5eacecad19972f8522c9320de2ec0bd3c294d0d2`. Its authenticated
`f569` before binary has SHA256
`1c682f6a9e72af2aae35efcb57bfaaaad46832c5518995e1305bbcf9a3de6373`;
the genuinely rebuilt `99d4839bde65afe26cc4afc89ec3f4a71a28894c` after binary
has SHA256
`37693807728bb8cab2b1a9e6904d61c41641d333f7f6122a2c5a8c94137ed4a4`.
All forty complete current public SFC results were `Ok` and equal between
selected and forced legacy, including code, CSS, maps, warnings, bindings and
macro artifacts. All eighteen normal raw template modules were equal. The job
still failed at an overly strict empty raw-notice assertion before hosted
runtime execution. Local pinned-Vue replay of the actual authenticated packets
passed the complete current/stock six-phase golden and historical failure
golden; this does not make the failed hosted workflow complete.

FormInput, DevPanel and WorkflowHistoryVersionSelect retain respectively two,
four and one exact parser compatibility notices in both raw template modes.
The assertion-only judge freezes every full historical/current vector, exact
message and byte span at all three paths, and independently derives the seven
self-closing nonvoid HTML elements from the pinned official original parse.
All other normal raw notice arrays remain empty. No generic `ExtendPoint`
exception, filtering or diagnostic projection is permitted. Native normal
diagnostic arrays stay empty; whole SFC `Ok` results still require empty errors
and complete equality. Existing SFC policy intentionally excludes this exact
self-closing rewrite message from warnings; preserve the actual full warnings
field rather than inventing propagation.

Exact source `6a37d1d27006f75a6b414367513633d5c48cc2df` has terminal
failed Check `37615725081` and custody `37615724363`, with successful native
run `37615724317`. Artifact `11479619918` retains genuinely isolated source
producers, complete original/official/template/SFC checks and successful hosted
mounted-runtime traces. Those narrow results do not qualify the complete job:
the next four-target invocation failed its empty-props control and did not
execute the other three binaries. The canonical sweep still reported 44,367
files, 43,993 templates, 43,977 compared, the unchanged sixteen old-error skips,
zero refusals, 285 DOM divergences, one diagnosed comparison and 233 production
differences in each inline/module mode. Capped windows leave 265 historical DOM
and 223 per-mode production inputs unclassified; do not infer their families.

The bounded successor restores authored slot-carrier helper registration at
the existing VNode boundary, emits `null` when existing child-key suppression
leaves no props, and preserves original multiline padding after safe prop
comment conversion. Native output had dropped that padding; the original
legacy module retains it. Authored parser-notice controls compare complete
literal vectors separately from branch-key semantic diagnostics. All original
inputs, independent official packets, complete assertions, capture drivers,
the sixteen-error allowlist and every ceiling remain unchanged.

The same four-target command uses `--no-fail-fast`: all four binaries execute
and any failure remains fatal. Optional full authored control packets go into
the existing uploaded custody directory, recording original bytes, exact
options, whole legacy/native code/maps/errors and actual test-producer hashes.
All twenty-four core key packets use the existing Rust-shard artifact. No
additional stage or target is added. Fresh exact-source Check/native/custody,
mounted runtime, all four regressions, whole corpus and protected delivery
remain mandatory; the earlier narrow passes do not transfer to the successor.
The corrective source genuinely merges signed main
`8c7727613de6213e0a52cde96eeb295d01c9bef5`, retaining incoming product,
fixture and oracle bytes. Only the canonical decision paragraph conflicts;
its full incoming clauses and the complete n8n suffix remain within the
existing 350 lines. The composed compiler keeps main's slot-child emission
and the reviewed named-slot-only loop guard. This source union receives no
historical runtime, corpus, queue or release credit.

The independent reference is n8n's locked Vue/compiler-sfc 3.5.26 browser
bundle, authenticated by SHA256. All ten complete originals retain parse,
script metadata, unbound and real-binding template modes, inline component
modules, maps and diagnostics. The assertion-only judge recomputes complete
code, preamble, maps and metadata with the same pinned API/mode without adding
a production stage or rewriting the retained official packet. Unexpected API
refusals fail qualification; unsupported style preprocessing stays explicitly
unqualified. InstanceAi is accepted in module/prefix mode, while function mode
must retain its three ordered duplicate-key diagnostics and diagnosed code;
that mode receives no successful module/runtime credit. Official UTF-16 offsets
and Vize UTF-8 spans are distinct coordinate systems. The official maps-on
shared-location anomaly remains raw evidence, separate from accurate authored
source locations. Current source before/after, full corpus, protected merge and
public consumer acceptance remain pending.

The spike reports approximately 11 seconds of Vize overhead in editor-ui and
one native lint call per rule/file. Measure the complete custom rule set against
the same source/dependency cohort before claiming a speed-up. n8n's remaining
ESLint parse cost is separate from bridge overhead. Keep exact-head source,
merge-queue and installed-package proof separate, and update the owning issue
when each requirement gains real evidence.

Exact source `9327e0698eb4049b058e6a50f8611ab621764a13` remains failed overall. Check `37628536484` evaluates synthetic merge `adc87b70b7a3a1c4bdb55358e49664366b3615f1`, whose authenticated tree is exactly the owned source tree `0c3b5fb69a862a8e8c6b2baf715dc77e0277023a`. Its complete canonical job `112816629817` passes the strict finalizer: 44,367 files, 43,993 templates, 43,977 comparisons, the unchanged sixteen old-error skips, zero native refusals, zero DOM differences and one diagnosed comparison. The full production scope has 43,992 templates and 37,345 inline/37,346 module whole-output comparisons, both with zero differences. Complete SSR also has zero differences. Artifact `11485889867` retains these current results; the earlier capped 265 DOM and 223 per-mode failures remain historically unclassified.

Native run `37628535030` succeeds. Custody run `37628535012`, artifact `11484817829`, retains sixty complete source/official comparisons and successful hosted mounted runtime with distinct before/current producers. The separate SFC runtime also passes. The entire custody job still fails three of its four actually executed targets: spread-with-key module parity, an original TypeScript input's plain-JS prefix comparison count, and the corresponding helper-registration count. Check additionally fails the path-attributed test layout and single-line prop-comment padding regression. These failures remain fatal and receive no adoption, P0 completion, protected-merge or release credit.

The successor must preserve all original inputs, runtime goldens, independent oracle packets, thirty-three immutable capture drivers, the sixteen-error allowlist and every ceiling. Authored TypeScript prefix comparison must use the original script's TypeScript mode; the unchanged generic plain-JS prefix recipe remains separate, with its actual complete refusal diagnostics retained and independently qualified. No refused mode receives successful module or runtime credit. Every successor requires fresh full source/native/custody/runtime/all-four and unchanged whole canonical zero-difference gates.

The reviewed corrective anchor is `311e3c8a0e1a42d91381ba3769ef9fcfc6c1f826`, genuinely descending from capture baseline `f5699c195e627ad26aa1c8efd4a59266120cd9b9` and signed current main `486c390d81643c16b865754239a987cfe4c9861e`. Historical `8f36b5f15e7f1227cc48232ff991a585a5741b24` and its failed overall `9327` result remain separately recorded. The controlled source merge has no product conflicts. Preserve the complete frozen common decision text plus the signed main's additional cross-file rule configuration clause at line 208; all 350 lines remain intact. The old anchor sentence in that historical common record does not override this corrected immutable workflow pin.

The bounded correction preserves merge-props selection when an authored child key was suppressed, trims only horizontal padding owned by terminal line comments while retaining newline layout, and moves the unchanged empty-props test into ordinary module discovery. Only the two named TypeScript originals opt into their actual SFC-language prefix recipe. All seventeen helper controls and the original one-input comparison counts remain strict; generic prefix JavaScript and all other recipes remain unchanged. Complete wrong-language refusal packets and independently pinned stock records are retained before assertions, separately from admitted typed modules. Source-derived current wrong-JS contracts remain unqualified until fresh hosted execution; old reused baseline packets grant no current-source credit.

Metadata-only pinning changes no producer, immutable capture driver, original file, oracle, runtime golden, skip allowance or ceiling. Fresh source Check/native/custody/runtime/all-four and complete canonical production gates must qualify this corrected source. No adoption or original P0 criterion is closed; protected merge and public release remain required.

The first authored source run built the current CLI but failed the exact props
reference packet: `includeDeclaration: true` omitted the real `defineProps`
variable declaration. The correction adds missing declarations only when
Canonical references agree with an exact OXC lexical occurrence in the same
script-setup local symbol group; unrelated names and cross-file scope are not
joined. The unchanged full n8n inputs now also check the false toggle. Both
toggles, subsequent navigation, dirty diagnostics and repair require a fresh
source run; the failed run grants no completed runtime credit.

The first lexical supplement did not repair the source packet. Inspection of
the complete virtual TypeScript confirmed separate original and synthetic
`props` symbols. The final correction records their existing setup/template
shadow edge at emission, from the existing pre-template anchor to the generated
props declaration. It adds no parse, pipeline stage, code bytes or enum kind;
the ineffective supplement is removed. The exact oracle queries both original
script and template positions under both reference toggles. Fresh source
execution remains required.
