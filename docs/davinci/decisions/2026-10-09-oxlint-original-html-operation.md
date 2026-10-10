# Proposed original HTML operation and CLI integration

Issue: [#7903](https://github.com/ubugeeei-prod/vize/issues/7903).
Source reviewed: `bd0f837555f0a260d38d6fcea8a098762d1871fd`.
Current private implementation base: `6a75086cf305ea576117cc48ec68d1ad6a6f2cdd`.
Only the lowest private retained-root/configured-execution layer is approved.
Public DTO/export, presentation, renderer/dependencies and CLI integration remain review boundaries; this record grants no wrapper, publication or issue closure.
The historical source-reviewed baseline above remains unchanged.

## Customer behavior and current failure

The original report explicitly includes built HTML under `dist/`, configuration
`ignorePatterns`, and CLI `--ignore-pattern`. Its literal executable fixture
contains Vue inputs; that fixture remains unchanged. Genuine HTML positives and all three original-path exclusion mechanisms are required in addition to it.

A bounded observation of actual production JavaScript at `7d645dbca9` and real
Oxlint 1.86 confirms two independently authored routing controls:

- HTML-only `src/Live.html` plus gitignored `dist/Built.html`: candidate discovery
  finds both, scoped selection returns before the engine call, and explicit
  relocated carriers for both are accepted. `--debug files .` through the wrapper
  reports both original paths. This proves routing, not native HTML diagnostics.
- Mixed Vue plus gitignored HTML: stock selects only `src/AppPanel.vue`, but
  ordinary wrapper `.` exits 1 because raw HTML presence triggers the mixed-target
  refusal. The copied Vue source is original; the core-only config and HTML are
  separate diagnostic controls.

Later `555619cb78` retains the identical whole Oxlint source tree and relevant
native sources. Both controls preserve authored bytes and clean temporary output.
They do not substitute for current source-built native execution or a public
installation. Their complete local packets are retained in the diagnosis receipt;
durable independently authored runtime fixtures belong to the implementation.

## One native operation

Add the named export `lintOxlintHtmlOriginals(request)`. Keep existing `lint`,
`lintPatinaSfc`, old DTOs, plugin loading, and the default collector unchanged.
The new operation owns admission, original-path selection, retained input/config
bytes, active Vize configuration, and one existing HTML lint per selected file.
It returns actual structured results; it is never a selector/query or a carrier.

The request is a new typed NAPI object:

```text
schemaVersion: 1
hostProfile: "oxlint-1.78.0" | "oxlint-1.86.0"
cwd: absolute original process cwd
literalTarget: one original literal file/directory argument
rootJson: regular original JSON configuration directly in cwd
expectedRootJsonBytes: Buffer from the existing CLI configuration snapshot
noIgnore: boolean
cliIgnorePatterns: original ordered strings
customIgnoreFilename: original filename, default .eslintignore
presentation: captured child context and requested/effective output format
```

The caller supplies no alternate rules, options, filename, source, or parsed
configuration. Expected bytes establish identity only. The private producer reads
and retains root bytes once, checks identity, and decodes once in its current root
configuration owner. The same decoded object feeds ignore construction and the
native execution plan. Selection and lint share retained `Original.bytes`; there
is no second source read, root reparse, selection export, token handoff, extra
native call, Git subprocess, serialization between levels, or new pipeline stage.
Independent host configuration loading remains Oxlint's existing ownership.

Refactor the private selector through a generic internal decoder/projector. The
existing `select` uses its current decoder and unit plan, preserving all historical
parse/refusal order and text. The new operation uses identity/unique-key decoding
and an owned execution plan; do not add public fields to existing `Selection`.
Keep all 45 producer cases and 90 dual-profile observations unchanged.

Completed results identify schema/capability/projection version, binding version,
host profile and pinned provider source. They retain root/VCS/custom-ignore
authority bytes, selected original paths/source bytes/origins, configured and
effective Vize rules, and actual diagnostics in native engine order. Each selected
original is executed exactly once. Include checked counts, measured operation
duration, original UTF-8 ranges and structured UTF-16 positions. Empty selection
is a validated completed operation with zero executions, never positive evidence.
Refusals carry typed cause/path/available bytes and no clean partial counts.
FFI failures remain actual thrown errors. Artifact identity is external evidence;
declared profile/version does not authenticate the loaded binary.

The new presentation object records the four-form format identity, quiet/silent,
resolved theme/color/link/width values and the actual stock child's piped stdout
and stderr context. Its resolver follows the immutable profile and only the
relevant captured display environment; it does not observe the parent's TTY or
silently choose another format. An unsupported effective format is explicit.
The completed operation also returns a native presentation operand: JSON row
fragments or a rendered native phase body/footer, with original engine counts
kept separately from displayed counts. It returns no fake Oxlint document,
configured-rule total, host thread count, or timing table.

Proposed additive presentation/result shape, finalized before the export ships:

```text
presentation request:
  requestedFormat: string | null
  effectiveFormat: default | json | unix | stylish | unqualified
  quiet, silent, color, hyperlinks: boolean
  theme: ascii | unicode; width: positive integer
  childStdoutIsTerminal: false; childStderrIsTerminal: false
completed result:
  schemaVersion, capability, projectionVersion, bindingVersion
  hostProfile, providerSourceSha, engineConfigValidation: not-performed
  rootDecision, sources[], projection, originals[]
  executedFileCount, errorCount, warningCount, timeMs
  presentation: { format, diagnosticFragments[] | body,
                  displayedErrorCount, displayedWarningCount }
refused result:
  schema/capability/profile identity, reason, path, JSON pointer if applicable
  available original/authority bytes; no clean partial counts/presentation
```

## Original root configuration authority

The initial operation admits the producer's exact one-literal-target, regular Git,
POSIX, regular JSON scope. Preserve its filesystem/ignore refusals and exact
ignore 0.4.33 traversal for both immutable host profiles. Do not introduce a file,
diagnostic, timing, or observation cap. The original customer `.`/root JSON shape
must work with VCS, root `ignorePatterns`, and ordered CLI patterns together.

Partition only `rules` entries under `vize/` and effective `settings.vize` into
the native plan. Retain foreign/core rules, settings, environment, globals,
plugins and resolver declarations untouched for the genuine host run. Do not
reject a supported root merely because it also configures core/custom rules, or
claim the native operation executes those rules inside HTML scripts.

Native projection accepts known Vize rule names, authored off/warn/error and
0/1/2 severities, tuples, the seven existing option schema families, locale/help,
and preset gating. Validate authored values before execution, including disabled
options. Build the explicit enabled set before options/severity overrides, so
incremental registry replacement cannot discard configured options. No unauthored
rule is enabled. Retain authored and effective values; omission is not an empty
array or a disabled rule. Existing JS normalization and option conversion are the
contract, including their recorded empty SFC-order groups and preset aliases.
Missing `rules` and an empty rule object produce an explicit empty enabled set,
not catalog defaults. Host categories affect built-in rules on both pins and do
not expand the explicitly authored Vize enabled set.

Allow foreign JS plugin declarations beside the canonical Vize declaration. Also
retain the authenticated own built plugin entrypoint used by the existing
standalone HTML fixture; do not replace its config with a package-name variant.
The CLI must establish that this is the actual owned plugin artifact; an arbitrary
module with a similarly named namespace is not Vize authority. The native
operation records the declaration but does not claim it imports Node plugins.
Derive self from the executing built CLI's sibling `index.mjs`; compare the
declared regular path and physical identity exactly. Resolve a canonical package
declaration from the original configuration directory to the same owned identity.
Do not admit a matching basename, suffix, version or package name alone, and do
not hardcode a checkout path or add a validation process. This keeps the original
standalone input/config/argv and its exact 9:15 one-error Stylish snapshot intact.
Root host `options.typeAware/typeCheck` stays in the actual stock project graph.
Native `settings.vize.typeAware:true` and active `vize/type/*` remain unqualified.
Project-wide `denyWarnings` must affect the combined exit; `maxWarnings`, unused
disable reporting and changed directive policy need explicit handling or refusal.

The initial projection must explicitly refuse unqualified inheritance, nonempty
overrides, nested configuration, JSONC/TS/MTS, duplicate JSON keys, alternate
Vize plugin namespaces, Vize type-aware/project settings and unsupported Vize
option/settings values. These are owed successors, not silently approximated
full-config support. Foreign fields remain host-owned, while fields affecting
Vize selection/execution need proven projection or explicit refusal.

Return `engineConfigValidation: "not-performed"`. Whole command admission needs
genuine stock normal/setup authority and unchanged root/authority custody.
Debug selection returns before JS rule-option setup; early all-excluded roots
can return before config loading. Neither empty debug output nor a native
completed empty set validates host/plugin configuration.

The no-duplicate orchestration proposed for review is:

1. Pass Vue candidates only to the existing scoped selection; retain the full
   candidate binding set for stock-selected host files. Prepare no HTML carrier.
2. With selected Vue, retain the existing normal original-project phase and
   normal Vue bridge. The source split keeps core/foreign values; the bridge split
   retains authored Vize tuples/options/settings and runs real JS option setup.
   Treat these as composed authority, not an unchanged-original normal process.
   Bind both projections to the same retained root snapshot. Do not add a third
   normal run or repeat Vize lint on the unbridged Vue source.
3. Without selected Vue, including HTML-only, retain the original unsplit normal
   host result. Surviving-root/walker-empty runs still execute plugin setup before
   NoFilesFound; early root exclusion does not. The normal result and stable
   original root identify the admissible branch.
4. Invoke the native operation once after the normal phases qualify. Its retained
   original-root plan executes only selected HTML, independently of carrier paths.

This composition needs direct review against both pinned setup paths and actual
whole failure/option controls. If bridge setup does not establish an authored
value's authority, the consumer remains unqualified for that value; strict native
projection cannot be renamed full host validation. The bottom native operation
can be implemented and qualified without claiming the consumer is complete.
Read-only review confirms this composition for active Vize options in the bounded
JSON envelope. Disabled tuples receive native strict validation only; do not
claim that they reached host option setup. Source-built runtime qualification of
both complete normal packets and their fixed root/phase identities is still due.

## CLI ownership and four reports

HTML is excluded from Vue carrier preparation. Raw HTML presence never decides
mixed admission. Reuse the qualified Vue selected-set/original-project path and
its report merge, then combine that baseline with the completed HTML operation.
HTML-only still performs the real original stock run; mixed Vue retains actual
stock/core/custom/parser/import/type-aware packets and qualified Vue diagnostics.
The native result is not cast into a fake five-field Oxlint report.

The private merge operand contains the whole original stock observation, optional
real Vue observation, and actual native operation. Capture argv/cwd, raw bytes,
status/signal/error, inherited presentation environment and provider identity
before assertions. Retain buffered bytes on spawn/stream failure. This extends
the current observation owner, not the number of provider queries or lint passes.

When native selection is empty, preserve the qualified existing baseline bytes,
status and stderr exactly. A refusal does not prove exclusion. Preserve complete
original packets on preparation, validation, signal, partial-output, conversion,
mapping or cleanup failure; append bounded failure detail, never clean success.

For nonempty native results, order stock rows, existing Vue rows, then selected
HTML in original-path order and native diagnostic order. Do not deduplicate fatal
or parser rows. Native severity/help/message is actual configured engine output;
byte columns for host presentation are derived from retained original bytes,
with independently checked CR/LF/CRLF and non-ASCII/astral controls.

| Format  | Whole merge contract                                                                                                                                                                                                                                                                                   |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| JSON    | Preserve the baseline's raw diagnostic fragments and untouched fields. Append actual native diagnostic fragments; patch distinct original file count and measured component duration only. Keep genuine host rule/thread authority. Add no invented warning/error totals or fake native host metadata. |
| Unix    | Keep actual baseline bytes and phase footers, then append native diagnostics and their own real problem footer. Original relative HTML names/ranges remain visible. Native clean output is empty.                                                                                                      |
| Stylish | Keep actual baseline bytes, then render genuine native findings under the captured profile/color rules and native problem footer. Retain unsorted engine order in evidence; stock bytes are never regrouped.                                                                                           |
| Default | Keep baseline diagnostic/timing/suppression text. Render native original-source diagnostics through a reviewed profile renderer, followed by actual native warnings/errors and a separately attributed Vize HTML file/time summary. Do not invent an Oxlint rule/thread or JS timing footer.           |

Proposed renderer ownership is private code inside the new operation's existing
final conversion, over already-computed diagnostics and retained originals:

- For 1.78, optional alias `oxc-miette = "=4.0.0"` supplies that pinned renderer.
  Keep the existing 3.0.1 dependency and old product rendering untouched.
- For 1.86, adopt only the immutable MIT-licensed renderer kernel: protocol,
  source handling, JSON/theme and graphical handler/report/snippet/gutter/label/
  line/span modules. Adapt its private label facade to the existing 0.142 `Span`;
  do not add another `oxc_span`/`oxc_diagnostics` family or the oxlint crate.
- Share the identical pinned Unix/Stylish body contract, with profile-specific
  original-byte scanning. A private diagnostic adapter supplies actual severity,
  exact original span/source and the message conversion defined by existing
  `formatPatinaMessage`, including Details/Help and locale/help-level behavior.
- Use explicit child presentation context. Stock stdout/stderr are pipes;
  blindly calling `GraphicalReportHandler::new()` would observe a different
  parent TTY. Do not change process cwd to obtain Stylish relative names.

This is a concrete dependency/notice choice proposed for root review, not an
approved or qualified renderer. The 1.78 archive matches its provider checksum;
the 1.86 assets are MIT and miette 4 is Apache-2.0. Preserve notices and apply
ordinary module-length limits to split private files, without exemptions.
The small kernel dependencies are bytecount, memchr, smallvec, textwrap, itoa,
unicode-width and unicode-segmentation. Main already locks all except bytecount
and the additional miette renderer family. Existing Patina formats/local miette 3
are not substitutes. Formatting remains final conversion, never another parse,
native query, lint pass or level stage.

Omitted format is environment-dependent. Preserve literal original argv/env;
never append `-f default`. Native selected HTML under an unqualified effective
Agent/Github/default context must preserve/refuse explicitly, not pretend that
empty output proves the Default contract. All four explicit forms need actual
whole-process source-native controls.

For HTML-only stock NoFilesFound, recognize only the exact pinned closed packet:
complete eligible native operation, stable root/source/ignore custody, actual
validated normal host result, genuine empty host selection, no signal/error or
extra bytes, and the profile/format's precise diagnostic-free terminal spelling.
Keep raw stock failure evidence; only this qualified empty stock component may
become zero files in the combined report. Never strip by substring/status alone,
invent configured rule counts, modify original argv or silently enable an
unmatched-pattern flag. All-excluded retains the original host exit contract.
Early excluded-root/null-rule packets, extra bytes/fatal spoof/plugin-option
failures, unknown effective format and incomplete metadata refuse the join.

## Minimal files and dependent delivery

| Layer                | Minimum implementation ownership                                                                                                                                                                                                                                                                                                                             |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Native operation     | `crates/vize_vitrine/src/napi/lint.rs` module/export wiring; new `lint/oxlint_html.rs`, `lint/oxlint_html/{config,dto,result}.rs`; `lint/file_collection.rs` private visibility seam and existing `file_collection/oxlint_html_profile{.rs,/policy.rs}` decoder/projector seam; reuse `lint_fix::lint_source` and existing linter option/conversion helpers. |
| Additive binding     | New typed declarations generated into `npm/native/index.d.ts` and existing `npm/native/index.js` synchronization; reuse `napi.rs`'s existing `pub use lint::*` export owner. Keep old exports/DTOs unchanged.                                                                                                                                                |
| Private presentation | New `lint/oxlint_html/report.rs` and `report/{diagnostic,unix,stylish,profile178,profile186}.rs` plus normally split licensed 1.86 kernel modules; optional NAPI-only renderer alias in Vitrine manifest/lock. New `npm/oxlint/src/cli/html-project-output.ts` and `html-json.ts` join the native operand without replacing the original serializer.         |
| CLI consumer         | `npm/oxlint/src/cli.ts`, `cli/scoped-selection.ts`, optional capability validation in `native.ts`/`model.ts`, plus new private `cli/html-operation.ts` and whole observation/merge owner. Keep existing Vue transport/default plugin behavior unchanged when HTML is absent.                                                                                 |
| Regression           | New separately authored `tests/_fixtures/differential/lint/oxlint-original-html-operation-7903/` and owning native/CLI tests; reuse existing source-native Actions/build/artifact paths. No new release gate.                                                                                                                                                |

Use three genuine dependent layers: private configured original execution;
complete additive NAPI/presentation; public CLI integration. Each child branches
from its parent's exact head with the parent branch as base. Register every
existing layer with `gh stack link`, verify
the same native Stack and ordered positions, and let root admit only the exact
head-green contiguous prefix with `gh stack merge`. Track all protected candidate
workflows, remove known-red candidates, and verify actual signed merges.

The bottom layer owns the internal plan, retained originals and actual execution
without adding a public export. The middle child adds the complete new DTO,
renderer final conversion and named export; the top child consumes that API.
Declare and qualify the whole new DTO shape before first public publication,
rather than publishing an incomplete presentation promise between layers.

No current native binding/declaration/export reservation conflicts with this
proposal. The held n8n 27-file helper lane reserves `npm/native/scripts` and its
package metadata; do not change those files or its unpublished module catalog.
The delivered #8323 staging, frozen original licensed 83/73 packets, n8n corpus,
formatter-history, transport helpers, and literal original Vue dual-host replay
remain unchanged. Original Vue keeps all twelve processes/twelve custody records
and its terminal record on each actual host; new HTML cases cannot replace them.

## Private bottom implementation

`lint/oxlint_html.rs` now builds one plan from the selector's retained root bytes
and borrows each retained original into existing `lint_source` once. Its result
records actual execution count and one-call elapsed duration. Unique-key decode,
strict bounded settings/severity/options and explicit empty-rule selection are
private; no export, presentation, dependency or public DTO changes are included.
Existing FFI annotations are feature-gated solely to reuse unchanged converters
in the existing pure Rust archive; the original profile module runs once.
The [new native corpus](../../../tests/_fixtures/differential/lint/oxlint-original-html-operation-7903/README.md)
authors 55 dual-profile cases, 11 whole diagnostic packets and a genuine engine
mutation law before execution. Hosted source execution is still pending; original
45/90, Vue, standalone and licensed n8n inputs/expectations remain unchanged.

## Required execution and remaining work

Independently author full HTML-only/mixed live, ignored and all-excluded packets
on both host profiles and all four explicit formats: VCS/config/CLI patterns,
ordered negations/ancestors, custom ignores, explicit/discovered paths, no-ignore,
clean/warning/error positives, same basenames, zero active rules, options/presets,
locale/help, directives and original-range controls. Include genuine original
Vue + excluded HTML, HTML + excluded Vue, both live, and both excluded.

Use existing Actions to build the exact source native binding, call the genuine
operation, and execute the real CLI with whole stock/native packets, source/tree
preservation, once-per-original execution and every admission/failure/refusal
control. Debug-files routing evidence cannot qualify the fix. Preserve original
producer, Vue, licensed n8n and full protected performance/history gates.

Design and independent implementation do not wait for v0.438 publication and add
no v0.438 gate. Public-installed closure remains separate: actual qualified
merged/tag/package/native identity and unchanged public-loader dual-host replay.
#7903 stays open until its genuine standalone/mixed contract is delivered.
No n8n/OXC upstream comment, change or fork publication is authorized.

[Original HTML provider constraints](./2026-10-08-oxlint-original-html-selection.md)
and [delivered private producer](./2026-10-08-oxlint-html-profile-producer.md)
retain all earlier scope and provider history.

Primary immutable sources:
[1.78 lint lifecycle](https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/apps/oxlint/src/lint.rs),
[1.86 lint lifecycle](https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/apps/oxlint/src/lint.rs),
[1.86 configured rule coverage](https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/crates/oxc_linter/src/config/config_store.rs),
[1.86 renderer](https://github.com/oxc-project/oxc/tree/2ae2939bb2fd98796393658b21556b2a2467e047/crates/oxc_diagnostics/src/handlers),
and [OXC license](https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/LICENSE).
