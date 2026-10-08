# Original-path standalone HTML selection

Issue: [#7903](https://github.com/ubugeeei-prod/vize/issues/7903).

Record the remaining standalone/mixed HTML gap and the acceptance contract
for a future replacement selector. The architecture is **unqualified**; this
documentation supplies no implementation, executable acceptance or release
credit. Preserve the [literal original Vue replay][original-replay] unchanged.

## Verified current constraints

Read-only tag resolution and source inspection fixed these provider identities:

| Actual host   | Inspected source commit                    |
| ------------- | ------------------------------------------ |
| Oxlint 1.78.0 | `c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd` |
| Oxlint 1.86.0 | `2ae2939bb2fd98796393658b21556b2a2467e047` |

Both [1.78][walk-178] and [1.86][walk-186] walkers filter extensions during
traversal. Both [partial loaders][loader-178] support JavaScript/TypeScript
plus Vue, Astro and Svelte, [excluding HTML][loader-186]. The walker SHA256 is
`28fc892d33488c80c328d2cdef3cf1d2bb513b29578304cd3ad3234659474d39` on both hosts;
the loader SHA256 is
`e3ff3fde4b560eb7d17dffb7850c1c5cdde9ab82f77c7e3ea5d278bbd81ad6ae` on both.
Stock `--debug files` reports supported files after selection/config filtering.
Unignored and ignored genuine HTML can both be absent: empty stock output,
success status or an unrelated Vue positive cannot qualify HTML ignores.
[1.78 selection order][lint-178]; [1.86 selection order][lint-186].

`Walk::with_extensions` is inside private `mod walk`; `config_loader` is
private too. Inner Rust `pub` items are not externally exported selectors.
[1.78 exports][lib-178]; [1.86 exports][lib-186]. Published npm exports and
native entry points expose no extension-independent HTML selection hook.
[1.78 npm][npm-178]; [1.86 npm][npm-186]; [1.78 native][run-178];
[1.86 native][run-186].

Source-public Rust pieces exist: `oxc_config::{GitignoreChecker,
all_paths_have_vcs_boundary, configure_walk_builder}` for VCS traversal and
`oxc_linter::LintIgnoreMatcher` for config patterns. They do not supply the
complete host selection/config-loading operation. [1.78 shared API][shared-178];
[1.86 shared API][shared-186]; [1.78 matcher][matcher-178];
[1.86 matcher][matcher-186]. Both crates set `publish = false`; `oxc_config`
has version `0.0.0`. Source visibility is not a published selector contract.
[Config manifests][config-manifest-178], [1.86][config-manifest-186];
[linter manifests][linter-manifest-178], [1.86][linter-manifest-186]. Vize's OXC
pin `fc702c1` [lacks the newer `GitignoreChecker` export][vize-pin]. A new git
dependency requires explicit identity/compatibility qualification, not an
incidental whole-family OXC update.

The [current wrapper][wrapper] collects raw HTML candidates. HTML-only inputs
take the existing relocated adapter before host ignore selection; mixed
Vue/HTML refuses even when HTML is ignored. Existing Vize CLI/native collectors
have different filter/default/config contracts and are not exact substitutes.
Preserve the adapter's supported HTML positive behavior; ignore selection and
mixed routing remain unfinished.

## Proposed replacement contract

Select genuine `.html`/`.htm` originals before creating carriers. Replace the
existing pre-lint collection/selection operation; add no native stage, provider
query, serialization between levels or second lint pipeline. Stock remains
authoritative for its supported files. HTML requires an independently proved
contract. Shared provider primitives are preferred, but the complete public
replacement boundary is currently missing; this proposal does not qualify it.

The single operation must own:

1. Original cwd, ordered original target arguments, exact host identity and
   existing loaded config envelope. Preserve explicit-file versus directory
   discovery provenance; expanding discovered files into explicit arguments
   changes VCS semantics.
2. Original-path CLI/ignore-file precedence and ordered negations, `--no-ignore`,
   VCS traversal/root checks/boundaries/symlinks, config-relative patterns and
   effective nested ownership. Extension eligibility is separate from ignores.
3. The original supported and eligible HTML paths with ownership/order retained.
   No probe, renamed input or carrier path participates. Reuse existing
   in-process data ownership without serialization between native levels.
4. Precise failure evidence and selected-original-only adapter creation.
   Preserve ordered overrides, rules/options/settings, original source authority,
   complete core/custom packets, totals and exit semantics. A root-only scope
   must state its limit; nested/JSONC/inherited requirements cannot disappear.

Branch on selected sets. Selected Vue plus excluded HTML must keep the qualified
Vue result without blanket refusal; selected Vue plus selected HTML owes both
positives. An unqualified envelope fails explicitly while retaining the original
report. TODO: establish this replacement boundary before transport implementation.
An appended native HTML helper, duplicated host pipeline, or `git check-ignore`
without explicit-file/index fidelity is not acceptance. Renaming extensions,
rewriting inputs and blanket suppression are not ways to satisfy this contract.

## Exact acceptance scope

Keep all original and older fixture inputs, configuration/ignore bytes, four
original paths, argv, complete authored expected outputs and limits untouched.
The literal Vue replay retains real `git init -q` and three repeated default/JSON
stock/wrapper runs: **12 processes + 12 custody records + terminal record per
actual host**, on both 1.78 and 1.86. Add HTML evidence without replacing these
records, reducing counts or changing batching/instruction controls.

- Genuine `.html` and `.htm` positives: raw/scriptless markup and the adapter's
  supported scripted form retain whole expected Vize packets and original ranges.
  Include clean negatives and an error positive that fails the complete command.
  Author expectations independently from reviewed source/rule contracts.
- Genuine original-path exclusions: VCS, config `ignorePatterns`, CLI patterns,
  `.eslintignore` and custom ignore paths; filename/directory/extension patterns,
  ordered negations and ignored ancestors. Exclude before carrier creation.
- Explicit gitignored files remain eligible; directory discovery is excluded.
  `--no-ignore` disables ignore-file/CLI sources while keeping config and VCS
  discovery filtering. Cover linked worktrees, VCS boundaries, info/exclude,
  hidden files, ignored global/generic ignore sources, minified names, symlinks
  and overlapping targets. Fix behavior to each pinned host. [Ignore contract][ignore-doc].
- Discovered/explicit root JSON and TS/MTS retain config-relative roots, object
  inheritance, ordered overrides, later re-enables, settings and options. Nested
  configuration/JSONC need separate required qualification; unsupported envelopes
  have explicit failure controls retaining the whole original report.
- Mixed Vue/excluded HTML matches qualified Vue; HTML/excluded Vue retains HTML;
  both selected retain both positives; all excluded follows the host's empty-set
  exit contract. Candidate order, basename collisions and multiple roots cannot
  change results. Raw HTML presence is never blanket refusal or suppression.
- Supported-extension sets and complete foreign/core/custom packets agree with
  same-host stock. HTML sets and whole positives/negatives have independent genuine
  original expectations; stock's unsupported HTML absence provides no credit.
- Every currently supported report form retains diagnostic ordering, totals and
  exit status. Keep original import/type-aware and write-refusal controls; an
  adapter-only packet cannot replace authored core/custom diagnostics.
- Save complete status/signal/error/stdout/stderr/setup packets before post-reads
  or assertions. Verify every source/config/ignore byte and complete owned recursive
  tree before/after. Cover selection/config/carrier/spawn/mapping/cleanup failures,
  retaining raw failure custody and cleaning all owned temporary paths.
- Reuse source-native Actions, both actual host installations, existing artifact
  upload and unchanged batching/instruction controls. Qualify source identity and
  the single-operation contract; add no query, native stage or install lane.

## Merge and publication boundary

#7903 stays open while genuine standalone/mixed HTML and its required
configuration or installed acceptance remain unqualified. n8n adoption
[#8142](https://github.com/ubugeeei-prod/vize/issues/8142) has an independent
current-contract acceptance gate; this historical wrapper selector record
grants it no adoption credit.
This record changes no implementation, acceptance count, pipeline or timing claim.
Pair the decision in its issue and canonical record when delivering the docs.

A future implementation needs independent review, its own exact-head source-built
dual-host Actions, unchanged protected full validation and actual merge. Historical
green scope/replay checks do not qualify its successor. Root owns queue admission
and official publication; repair/remove known-red candidates without blocking peers.

Release completion needs the exact qualified merged commit, immutable tag identity,
terminal publish workflows, fresh public registry package/native identities and
hashes, then installed replay of unchanged original Vue and genuine HTML/mixed
controls on both hosts. Use the public native loader without local/source-path
substitution. Local bindings, development wrappers, renamed probes, alternative
fixtures and unreleased packages are not public-installed proof. Upstream n8n and
OXC remain read-only: no comments, changes or fork-publication are authorized.

[original-replay]: ./2026-10-08-oxlint-original-ignore-replay.md
[wrapper]: ../../../npm/oxlint/src/cli/scoped-selection.ts
[walk-178]: https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/apps/oxlint/src/walk.rs
[walk-186]: https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/apps/oxlint/src/walk.rs
[loader-178]: https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/crates/oxc_linter/src/loader/partial_loader/mod.rs
[loader-186]: https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/crates/oxc_linter/src/loader/partial_loader/mod.rs
[lint-178]: https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/apps/oxlint/src/lint.rs
[lint-186]: https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/apps/oxlint/src/lint.rs
[lib-178]: https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/apps/oxlint/src/lib.rs
[lib-186]: https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/apps/oxlint/src/lib.rs
[npm-178]: https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/npm/oxlint/package.json
[npm-186]: https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/npm/oxlint/package.json
[run-178]: https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/apps/oxlint/src/run.rs
[run-186]: https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/apps/oxlint/src/run.rs
[shared-178]: https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/crates/oxc_config/src/lib.rs
[shared-186]: https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/crates/oxc_config/src/lib.rs
[matcher-178]: https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/crates/oxc_linter/src/config/ignore_matcher.rs
[matcher-186]: https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/crates/oxc_linter/src/config/ignore_matcher.rs
[config-manifest-178]: https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/crates/oxc_config/Cargo.toml
[config-manifest-186]: https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/crates/oxc_config/Cargo.toml
[linter-manifest-178]: https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/crates/oxc_linter/Cargo.toml
[linter-manifest-186]: https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/crates/oxc_linter/Cargo.toml
[vize-pin]: https://github.com/oxc-project/oxc/blob/fc702c1fa9f0412d06ec6908b58cd395b826cf7f/crates/oxc_config/src/lib.rs
[ignore-doc]: https://oxc.rs/docs/guide/usage/linter/ignore-files
