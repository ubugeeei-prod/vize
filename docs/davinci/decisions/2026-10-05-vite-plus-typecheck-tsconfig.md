# Relative TypeScript projects in Vite+ tasks

Issue: [#8017](https://github.com/ubugeeei-prod/vize/issues/8017).
Paired source decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8017#issuecomment-5988901037).

Status: source preparation for exact-source Draft qualification. No native
execution, Actions acceptance, queue admission, merge or release is claimed.
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

The genuine replay on actual main
`1744a7ee84c202ce78f03a0148f335a5022b72e3` preserves every owned source/input
blob and every incoming canonical line. This main already builds the default
CLI and writes its build receipt before PR/protected tooling tests. The oracle
is registered through that existing tooling suite, retaining the strict receipt
and native-provider requirement. Existing full Check also builds that recipe.
No new workflow, local native build/install or manual benchmark campaign is
needed. TODO: fresh actual Actions must establish all runtime vectors; then the
protected full suites, unchanged instruction ceilings, actual signed merge and
an approved release remain required. No typechecker speed or native-stage
adoption credit is claimed.
