# Reviewed release-only native input scope

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision and boundary

The ordinary type-checker native phase workflow previously treated every
`tests/tooling/**` change as a native compiler/editor input. Keep that broad
positive pattern and every existing positive. Exclude only these exact paths,
in order after the broad tooling pattern:

- `tests/tooling/release-input-scopes.test.mjs`
- `tests/tooling/release/release-public-acceptance.test.ts`
- `tests/tooling/support/release-retirement-transport-law.ts`

These files qualify release input selection, public publication/absence receipts
and bounded registry transport. They remain ordinary Check tooling tests.
Unknown siblings, all fixtures and native driver/editor/checker inputs retain
their previous triggers. A mixed change still runs the native workflow when
any other changed file matches a retained positive pattern.

[GitHub's official path-filter semantics](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onpushpull_requestpull_request_targetpathspaths-ignore)
apply matching patterns in order: a later negative excludes that file and a
later positive can include it again. The new law reads the actual YAML and
implements its limited literal/`*`/`**` pattern subset, rejecting new syntax.
It verifies the original broad control, each exact negative, unknown neighbors,
mixed changes and a later-positive ordering control. This is source-policy
verification; hosted event selection remains an acceptance obligation.

Only `pull_request.paths` changes. Its equivalent multiline YAML flow form
keeps the workflow at 350 rows. The source/runtime installation, authentic
receipts, immutable source recipe, compiler/editor commands, profiles, timeout,
manual input, metadata-edit handling and cancellation remain unchanged.
All Check, protected full/merge-group and release obligations remain unchanged.

## Complete reviewed source authority

`tests/tooling/fixtures/typechecker-native-release-inputs.json` records complete
local-import closures and SHA-256 hashes for three separately reviewed cohorts:

| Reviewed source                                                | Present excluded roots | Full source closure |
| -------------------------------------------------------------- | ---------------------: | ------------------: |
| Main `d55b8238f89bab4418a1a961bd8db1c84911cb42`                |                      2 |            34 files |
| Retirement PR #8644 `d24c5d459fa5adda1e21928ee47f36953151cab4` |                      3 |            36 files |
| Current main `7cb4fc3c5ad08d969df5327f464e2c120cb29220`        |                      3 |            36 files |

The third file is absent on the original independent main base. Its prospective admission
is explicitly bound to the already reviewed #8644 source, including the existing
two roots and every transitive import. A review matches one entire physical
source cohort. Partial presence, mixtures, unresolved or nonliteral imports,
linked members, unknown imports and native product/benchmark dependencies fail.

Hashing complete source bodies also rejects a new native command added through
`process.getBuiltinModule("node:child_process")` with unchanged imports. The law
demonstrates this rejection in both an excluded root and a transitive helper.
It scans every tools source, actual actions, workspace Rust/npm sources and the
root package manifest for new consumers of the excluded files. Existing release
selection tests and the full merge tooling inventory remain mandatory.
The parsed job commands, workflow/job/step environment and reusable-workflow
inputs of every current workflow are also checked, including manual/reusable
input defaults and options. Future command, environment and input controls
refuse new consumers. Trigger metadata and descriptive comments are not command
consumers.

If any reviewed source changes, Check must fail until the exclusion is removed
or the complete source is explicitly reviewed and repinned. Native authority
must be readmitted to the native campaign. Do not bless unknown behavior by
updating hashes mechanically. The manifest, guard and its consumer-scan helper
remain outside the exclusions; changing any of them triggers the native
workflow for that exact PR.
Shared helper changes may therefore require a reviewed pin refresh even when
the three roots retain their imports. This is the intended conservative boundary.

The Check source-CLI debug/test artifact is not a native phase artifact. Native
qualification still uses `ci-opt`, inheriting release with thin LTO and 16
codegen units; no artifact or profile alias is introduced.

## Incoming reviewed planner composition

After #8644 actually merged, observed main
`1d988f6bcac8afc6a0fcad9b76b0d5f2da27be71` matches every source in the 36-file
retirement cohort. Independently reviewed #8651 at
`953bede777e4648239e06ab716c36eee0be103b0` changes one member of that closure:
`tools/support/compat/github/plan-source-checks.ts`, Git blob
`6701207eec6feb5236399d5300d6164a7d4c0358`, SHA-256
`e23189d80dc8bae3e2a431167b2c6f6a4219e9c67377b69c411534e3ba86c079`.
Its prior hash is
`eef5244e8c2987e15b1505a92e5cb6d42b6302d3bdfd09496b398c3766e4b117`.
The reviewed diff adds an exact Node Docs ledger comparison and existing tooling
selection branch, with no new imports or commands.

PR #8651 actually merged at `2c6f1abebf1c4f4353e76aaf4f85cd3f646b50a1`.
The fresh main cohort above authenticates all three physical roots and every
one of the 36 transitive files against its actual Git source. Its complete
import closure is unchanged; 35 file hashes remain identical to #8644 and only
the reviewed planner body has changed. Both historical cohorts retain their
original complete bytes and hashes. The old two-cohort guard correctly refuses
this current source; no one-file projection or automatic broad repin is admitted.

Physical-source checks reject linked ancestor directories as well as linked
files. Consumer checks include actual workflow-level environment values and
manual/reusable input defaults and options, while preserving trigger metadata
and descriptive text. Controls reproduce the inherited ancestor-link and
workflow omissions before the corresponding refusal checks are added.

The separately reviewed
`tests/tooling/support/typechecker-native-release-consumers.mjs` keeps the full
consumer scan below the source-row budget without shrinking it. Physical files
are scanned; linked consumer directories, ancestor directories and workflow
files refuse qualification. A real directory-link control reproduces the old
walker's silent omission before the new check. The helper is outside all three
excluded import closures, and its own edit retains the native trigger and
selects the Check source guard.

The expanded tools scan finds one existing literal Check-selection reference
in `tools/config/vite-plus/tooling-test-scopes.ts`, SHA-256
`93dbdb887477d9726fe1f0da848b429ad21ff95e3ba387cdcb30f2fae8c75900`.
This whole file contains only exported test inventories/scopes and is already
authenticated identically in all three complete cohorts. Consumer admission
uses only the selected, verified atomic cohort's whole-body hash after physical
source checks. No path exemption, any-version fallback or additional pin is
introduced. Unknown delegation from the original native producer into a new
tools helper, a root package script consuming an excluded test, and any change
to the known metadata body all refuse; both authored consumer escapes reproduce
the old scanner's omissions while the excluded cohort remains unchanged.

## Historical observation

Retirement-only #8644 changed seven release/tooling/prose paths. Its exact source
`d24c5d459fa5adda1e21928ee47f36953151cab4` ran
[native workflow 38144256007](https://github.com/ubugeeei-prod/vize/actions/runs/38144256007)
successfully. Job
[114484337756](https://github.com/ubugeeei-prod/vize/actions/runs/38144256007/job/114484337756)
used `blacksmith-32vcpu-ubuntu-2404`, from 13:43:23 to 13:59:19 UTC on
2026-10-11: **956 seconds**.

| Existing step                                            | Observed seconds |
| -------------------------------------------------------- | ---------------: |
| Template emits, config-scoped helpers, actual CLI/editor |              344 |
| Outside-import native/CLI qualification                  |              175 |
| Original reference-path native/batch/CLI qualification   |              225 |
| Production CLI and projection build                      |               67 |
| Every actual shard capture                               |               63 |
| Strict phase parser/projection probe                     |               21 |
| Complete evidence pack/reconstruction                    |               25 |
| Optional callable defaults                               |                4 |

Artifact `11704735888`, `typechecker-native-phases`, retains 95,059,537 bytes
of authentic evidence. These timings measure the old unrelated job's cost.
They do not establish a new source's wall-time gain or transfer acceptance.

## Validation and remaining delivery

The implementation carries actual-YAML ordering/unknown/mixed controls,
whole-source body/import/presence/link refusals and real consumer scans.
Existing metadata/concurrency/manual and release-selection laws must also pass.

The original 19 combined laws remain represented. Current focused validation
passes all 21 guard/event/release laws, including the guard/event subset's 13.
All three complete source cohorts authenticate to actual Git bytes and complete
local imports: 34 historical main files, 36 retirement files and 36 current-main
files. Both inherited pinned objects remain byte-exact. The authentic #8644
envelope is admitted, while replacing its retirement observer with the older
main observer refuses the mixed cohort. The old two-cohort guard refuses the
current planner body before the third complete cohort is explicitly reviewed.
Targeted lint, formatting and diff checks pass. Parsed workflow comparison keeps
all 34 original positives and every other YAML value, with bytes outside the
path stanza unchanged. Removing the sole line-341 companion suffix restores
every original foundation byte; both shared files retain 350 rows.

TODO: run exact-source Actions, verify the intended hosted PR event selection,
retain genuine native execution for the manifest/guard change, and complete
protected full checks and actual merge. Refresh whole reviewed pins explicitly
when composing incoming release or tooling helper changes. Publication and any
end-to-end performance gain remain unverified.

## Installed reported Vapor composition

PR #8645 source `8020492684d9688bd188d50244ae198cc975bfec` adds an installed
Vue rc.10 release contract. The three excluded roots still have a complete
36-file local-import closure. Only `release-input-scopes.test.mjs` changes:
it registers the literal `release-public-vapor.test.ts` contract in the
previously broad release inventory and adjusts the corresponding exact counts.
Its imports, all eight release-selection laws and all historical inventory
assertions remain; it introduces no native command or new import.

The former whole-root SHA-256 is
`c4ac93b82a1d29ed851879e8426fb4d7039a77172aa19046610795a9144436f5`;
the reviewed current body is
`189e031b5c5768880acb47a2740d421cca46c674332dce297a967d2415e9fd06`.
Every other reviewed member retains its whole bytes and hash. Source `802049`
and the failed protected composition
`f68b29a58027a8f1677327f6e5683ed40b7e0a88` authenticate the same complete
cohort, as does the actual main-rebased successor
`c902b204df178d366a4a862200d56d321b0bd880`.

Keep all three historical review objects unchanged and admit this fourth
complete cohort only. The inherited guard reproduces exactly three source
refusals on the protected composition before this explicit review. Every
unknown body/import, native-command, linked/partial source, workflow/input,
delegated-tools, root-script and changed-metadata refusal remains required.
The actual all-tools/actions/Rust/npm/root-manifest/workflow consumer scans
pass on both the protected composition and the main-rebased successor.
The native workflow, Rust/CLI/editor sources, lockfiles and existing native
drivers retain their exact bytes; the separate Check registration adds only
the named public-Vapor TypeScript project. No native build input is replaced.

TODO: qualify the repaired exact PR source, then its fresh protected full
candidate and all consumers through actual merge. Installed rc.10 acceptance
still needs an included public release; neither publication nor a CI wall-time
gain is established by this source-cohort review.
