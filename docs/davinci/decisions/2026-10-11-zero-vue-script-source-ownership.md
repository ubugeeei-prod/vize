# Generated fixture projects with no Vue SFCs

Paired decision: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6110616031).

Strict Real Project Matrix [38147644073](https://github.com/ubugeeei-prod/vize/actions/runs/38147644073)
ran release candidate `9466fdbb90bfae8803d5e57ae267b656be051721` over delivered
product source `1d988f6bcac8afc6a0fcad9b76b0d5f2da27be71`.
Four core typechecker runs failed with exit 2 and a complete empty JSON report:
vue-native-core (shard 0), docsify (9), wakapi (10), and petite-vue (11).
Their registry entries deliberately freeze Vue selection at zero SFCs while
retaining typechecker coverage. No entry or upstream file is changed.

Two observer inputs caused the failure: the generated fallback tsconfig had
only `compilerOptions: {}`, and the invocation still explicitly selected
`**/*.vue`. An `allowJs` change alone would leave that explicit workload empty.
The production CLI correctly refuses a selected project without checked roots;
that refusal and all production code remain unchanged.

The exact pinned archives were inspected as physical files. Every script byte
matches its blob in the complete primary Git tree at the registry revision.
The additive compressed inventories in
`tests/tooling/fixtures/zero-vue-script-source-evidence/` retain paths, sizes,
SHA-256, Git blob hashes, archive identity, revision and extension counts.
They are evidence, not replacement fixture inputs or native acceptance.

| Project         |  JS | CJS |  TS |
| --------------- | --: | --: | --: |
| docsify         | 106 |   1 |   2 |
| vue-native-core | 194 |   0 |   9 |
| wakapi          |  21 |   0 |   0 |
| petite-vue      |   1 |   0 |  20 |

Only a generated fallback for an explicitly zero-Vue registry project opts
into the script branch. It scans the case-sensitive closed JS/TS extension set,
records declarations separately from executable roots, excludes Git/package
metadata, and refuses escaping links, cycles or no executable roots. An actual
unexpected Vue input also refuses. The generated config owns the complete
physical file list through `files`, empty `include`, `allowJs`, `checkJs` and
preserved JSX; the CLI selects those exact project roots without a Vue-only
positional glob. Nonzero Vue projects and explicit source configurations retain
their original command arguments and generated configuration bytes.

The strict observer requires every executable root in the checked output and
in the one exact generated owning program, including its compiler options.
Full diagnostic arrays, counters, geometry, status, program metadata, Vue
requested manifest and transitive coverage remain intact. Additional raw
`typecheckerSourceScan` custody binds every physical byte and actual generated
config hash; both are reread after checking. Missing/substituted/foreign inputs,
changed config/source bytes and zero checking remain fatal. No source skip,
zero-as-success law, baseline edit, diagnostic waiver or budget increase occurs.

Local existing coverage/program/envelope laws pass (18); the four new observer
and canonical-Rust controls pass, with three new Rust laws. The actual native
control reproduces the unchanged old invocation’s whole exit-2/empty-report
refusal before checking the repaired generated owner. It requires the exact source-built CLI receipt on Actions and freezes the
whole two-error JS/TS diagnostic vector plus the declaration member. Its full
raw report and physical source custody are retained under `target/differential/`.
The pinned whole source inventory regression also passes. The local machine
has no qualified executable, so this control remains pending
hosted execution. Fresh source Actions, protected suites, actual merge, all four
full original-project runs, the remaining Matrix failures and public release
acceptance remain unfinished. This observer repair claims no measured gain.
