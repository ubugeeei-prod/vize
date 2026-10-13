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
has no qualified executable. Original child head `b38e12bb3b1d14383a58389ecb02ecedaa7a6797`
passed the complete native control in source run `38151936023`, tooling shard 1,
job `114507444790`. Its retained whole packet has SHA-256
`bc81bab6c4deb679952ce1e3709f031b54c6c570320d350aa7a0face6fd54e98`,
producer source `3440ed3ffad6e985d55ecf6f3e10d24848798690` and binary SHA-256
`2c693dd1343f16d5f167e4bb317b55cb1adebf7983ef3a13467b6d8853b2731f`.
It preserves the old exit-2 refusal, new exit-1 findings, both complete
`TS2322` diagnostics and all three checked members. This predecessor evidence
does not qualify a successor head.

The shared observer insertion and adjacent canonical rows require reviewed
composition with #8667, so existing #8673 becomes its GitHub native Stack
child. Both complete original Rust test modules remain byte-identical; runtime
code uses the automatic three-way merge. The frozen project inventories and
original complete native packet remain unchanged. The original composition
required fresh source Actions and a new native packet. The ready parent could
enter the protected queue without waiting for this child.

#8667 actually merged at `2026-10-11T16:49:36Z` as
`0421bce9025f1d533876fa74d0cc987b889f6f07`. Its exact protected Check
`38155675612` and all four consumer workflows succeeded. Composed child
`c789dab6c55e2dda802864caf0a57cc09167ede1` separately passed full source Check
`38153109076`; tooling shard 2, job `114511239433`, retained a new native packet
with SHA-256 `785e5145578cd1440b13504ef19c16a3a7c3f339ff64fcbf51aae117d86b0815`,
producer `8acfee3bab0ced9497e58910b0dd5b37c9ce84a6` and binary SHA-256
`d2b0204e203ec6751d8b804bc9bd5be6b820cc02422d5ad3012aea4c83774608`.
Its complete before/after stdout, parsed reports, source scope and coverage
equal the original child packet; producer and binary identity are distinct.

The child is replayed onto the actual parent merge. All 13 runtime, test and
fixture evidence blobs remain byte-identical to the composed child. The
canonical resolution preserves all 350 fresh-main rows and appends only the
owned row 325. The companion records these authorities; a fresh successor
must repeat source Actions and native packet custody before queue admission.
Child protected suites, actual merge, all four
full original-project runs, the remaining Matrix failures and public release
acceptance remain unfinished. This observer repair claims no measured gain.
