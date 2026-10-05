# Relative TypeScript projects in Vite+ tasks

Issue: [#8017](https://github.com/ubugeeei-prod/vize/issues/8017).
Paired source decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8017#issuecomment-5988901037).

Status: Source qualification passes all 39 complete CLI processes. The first
protected candidate fails the full L1 corpus's original Counter inventory and
one independently derived remark registration; it is no longer queued. Fresh
full corpus qualification, protected checks, merge and release remain required.
The initial private source was reviewed at
`63a4d2801f8c5d68563508af9952eb7d292fa718`; one necessary P0 Draft is
authorized to establish the actual whole-process evidence.

On source `62f55b45f0b40a1499f595688fd1a7396901b690`, `runNative()` writes
the merged Vize config into the OS temporary directory. `relocateTaskConfig()`
rebases other project paths but leaves `typeChecker.tsconfig` relative.
`ProjectModel` correctly resolves configured paths relative to that serialized
config's directory, selecting a missing temporary-directory TypeScript project.
The two no-input branches then emit an empty success. This is a source diagnosis;
the original task has not been executed locally.

Rebase this one configured path against the task's project directory before
serialization. Preserve absolute paths and checker options without changing the
original config. Do not change `ProjectModel`: ordinary Vize config paths still
belong to their config directory, and explicit `--tsconfig` belongs to the cwd.

A selected project must name a real file. A selected or discovered TypeScript
project whose effective roots produce no supported workload must refuse with
operational exit 2 and an explicit stderr explanation, including quiet mode.
JSON keeps its existing complete empty envelope and zero diagnostic counts;
the process status reports the operational failure. Truly unconfigured empty
and unmatched invocations retain exit 0 and their existing text/JSON output.
An intentionally disabled checker remains an intentional skip. A files-empty
solution with nonempty referenced projects must still check those projects.

The authenticated reporter is `ubugeeei`, public user ID `71201308`. The source
commit retains `Co-authored-by: ubugeeei <71201308+ubugeeei@users.noreply.github.com>`.
The corpus retains the entire issue body and every original fenced input:

| Original input      | Bytes | SHA-256                                                            |
| ------------------- | ----: | ------------------------------------------------------------------ |
| `vite.config.mts`   |   163 | `7ee92d78a008d385a87beac83fefaebaf0353dcbf01c6a92edd26c70520e7689` |
| `tsconfig.app.json` |   199 | `35feafeab4c6c9026e88096f22231f2232dda9a954add34b2b77538656738236` |
| `src/Counter.vue`   |   118 | `6b6c71eb17b0f5be731bf41566cccdcfb19e0f1be0575ad31f458ee4c1986082` |

Thirteen pure path/input-custody and snapshot-catalog controls pass. The prepared whole-process oracle
checks original invalid → repair → invalid diagnostics and full JSON streams,
effective program membership/options, task/direct generated TypeScript bytes,
absolute/nested/config-directory/explicit-override paths, missing and empty
projects, ignored workloads, disabled checking and nonempty references.
The native disabled checker retains its genuine process receipt; the separate
source-factory `typecheck: false` control records only actual status and zero
executor calls, without inventing a CLI report. Both factory and task execute
in the fixture's actual project directory.
It saves original input bytes, serialized temporary configs, source/build and
provider hashes and complete process streams/status before assertions.

The runtime scope is current-source `defineConfig`/`runTools`/`runNative` plus
the actual default-source Rust CLI. The original factory arguments are retained;
only its package import points to the source provider. An explicit test dispatcher
launches the Rust executable rather than the published Node launcher. This is
not evidence for the reported package versions or the full published `vp run`
bootstrap. Generated TypeScript equality is not decoded source-map proof.

The first automatic source Check, [37271515293](https://github.com/ubugeeei-prod/vize/actions/runs/37271515293),
failed at source `21994cf79d82c523fdcbd1fcf72ad580e71a8119`. All five new
oracle groups stopped before native execution because provider custody resolved
`vize/config` from the tests directory instead of the plugin's dependency context.
Separate gates found the missing assertion-only oracle catalog entry, the new
original SFC absent from the L3 file inventory, and an old empty-test scaffold
that created a discovered tsconfig while expecting unconfigured success.
The authenticated four logs and small official ZIP are retained; no 39-process
runtime acceptance was established. Native-phase and ordinary source build
success are separate from this failed fixture. The bounded fixture successor
was independently source-reviewed at tree
`3ecfc9549e0155386dda8cae4f83066f38c4e1ba`; runtime qualification remains pending.

Prepare the current CLI's JavaScript config export explicitly with `vp pack`
inside the existing source-built tooling oracle, capturing full preparation
streams before assertions. Resolve it through the real plugin context and
require config normalization to be the exact local source addon's function;
retain both provider hashes. Link the fixture's actual Vue and Vite+ providers,
without depending on a preceding test's package preparation or root hoisting.
Keep all original inputs and every full oracle expectation unchanged. Extract
the old empty test without behavior changes first, then use an actual empty
directory outside the repository to avoid its inherited tsconfig for genuine
unconfigured success and add a strict discovered-empty
exit-2/full-JSON/stderr pair. The actual L3 run observed 450 files, 150 remarks,
15 applied, 135 missed and zero remark changes; add only the original Counter
path to its census, preserving every remark and explanation. No ceiling moves.

The paired [debug-stream correction](https://github.com/ubugeeei-prod/vize/issues/8017#issuecomment-5990026339)
retains the second automatic source [Check37274579401](https://github.com/ubugeeei-prod/vize/actions/runs/37274579401), which
uses PR source `2b431bc6455aff13c553500a16204a0aa140eb35` and actual default
build checkout `14597fe41399e09842fbbc5a471c6c227078b5d2`, binary SHA-256
`8c1b00d3f936b0ec323ca3467df5a544d8546ffee4ac7d8eabcc3e912b5d6cb2`.
All four Rust workers pass, including both complete unconfigured/discovered-empty
laws and the original SFC inventory. All 39 native CLI processes are retained in
the official tooling artifact `11328988967`, SHA-256
`12d60898f0cb9dc0b66384f616cdbf093d4f23c84158f933e2656f9f61df92f9`.
Four oracle groups pass; the original group fails only because it expected empty
stderr under explicit `--show-virtual-ts`. That flag intentionally dumps the
complete shared helper and generated Counter source, including in quiet mode.
The task/direct debug stdout and stderr are byte-identical; each stderr has
31,872 bytes, SHA-256
`9bafaedf842dd2e605cf91b33891094d62a3a92e7e4713f592a9877ab980d848`.
A read-only replay checks all 39 complete ordered reports, statuses, raw UTF-8
streams and input hashes against the unchanged expectations. Correct only this
debug expectation: retain complete task/direct stderr equality, exactly the
helper and authored Counter headers, and the entire JSON `virtualTs` bytes plus
the output newline as the dump suffix. All original inputs, production modules,
stdout/report/status expectations and 39 process calls stay unchanged. Fresh
source Actions must execute this corrected assertion; this replay transfers no
successor or protected-suite acceptance.

The genuine replay on actual main
`d8c3f46657f849e1839d55349b6eabdb766bfd57` preserves every owned source/input
blob and every incoming canonical line. This main already builds the default
CLI and writes its build receipt before PR/protected tooling tests. The oracle
is registered through that existing tooling suite, retaining the strict receipt
and native-provider requirement. Existing full Check also builds that recipe.
No new workflow, local native build/install or manual benchmark campaign is
needed. TODO: fresh actual Actions must establish all runtime vectors; then the
protected full suites, unchanged instruction ceilings, actual signed merge and
an approved release remain required. No typechecker speed or native-stage
adoption credit is claimed.

The paired [L1 Counter registration](https://github.com/ubugeeei-prod/vize/issues/8017#issuecomment-5990869755)
retains successful source [Check37278230013](https://github.com/ubugeeei-prod/vize/actions/runs/37278230013)
at `3f33cffb993dde08342e878576d09e666de5bf5c`: all 39 complete process
observations pass with default-build checkout
`2f3cf5e2f532f2e7d267d91ba4646270d30f6a7b`, the same source tree, and binary
SHA-256 `e3dcd3c0591adf674073d7f00b180c92d5acd907704fd007f6f13254959d85c9`.
Its protected candidate `4d1dce911c1dcc7dfae836f04d678fd61b7832bb` fails
[Check37279798431](https://github.com/ubugeeei-prod/vize/actions/runs/37279798431):
the actual full L1 corpus reports 450 files, 1,033 remarks, 159 applied,
874 missed and one change. Its first inventory assertion stops later remark
and backlog comparisons. The candidate was removed; its separate unchanged
100+4 instruction passes establish no full-suite or merge acceptance.

Add only the original Counter file and the independently authored
`s2.hoist-static missed static-subtree @87:105 tag="p" blocker="child" op="ui.interpolation"`.
The entire 118-byte input contains one such element, whose sole interpolation
is dynamic text; empty attributes and bindings produce no additional prop
remark. Independent source review confirms the exact span and frozen rule.
All original 449 file rows and 1,032 remark rows remain byte-exact, including
the final blank line. The existing backlog reason changes only from 185 to
186 hits and 81 to 82 files, with its first example and all 24 groups preserved.
The existing automatic native workflow runs plain complete lowering, DOM and
remarks legacy-differential suites at the exact source root for this PR before
readmission. Collapsing only three existing shell command continuations keeps
their argv/captures exact and the prospective workflow composition below
350 lines. No production, original input or 39-process expectation changes.
TODO: these full corpus/output/span and canonical backlog comparisons must
pass in fresh source Actions, followed by new protected gates and actual delivery.
