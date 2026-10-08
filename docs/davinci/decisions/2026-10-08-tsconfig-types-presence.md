# Configured type-library inheritance (#3984)

The bounded follow-up to #8229 repairs `compilerOptions.types` collection for
multiple `extends` entries. A permanent visited set previously suppressed a shared
ancestor on the later visit, combined unrelated parent arrays, and lost a later
explicit empty array. Official TypeScript 6.0.3 and 7.0.2 select the last present
array, including `[]`, while an absent field leaves an earlier value intact.

Reuse the parent's operation-local active-ancestry/completed-value helper with
`Option<Vec<_>>`. Replay complete acyclic values and cache successful absence;
cut only an active cycle. Assign the last present parent array and preserve local
array overrides. Read/parse failures keep the existing sibling boundary: skip
that failed parent, retain earlier valid siblings or local overrides, release the
active path and leave the failed result uncached. There is no global memo, native
cycle-parity claim, config-error suppression or new pipeline stage.

The complete authored corpus freezes all common and case-specific inputs,
broken/repair sources, seven effective configurations and fourteen untrimmed
stock `--showConfig` packets. Whole raw stock diagnostics and exit codes remain
separate for TypeScript 6.0.3 and native 7.0.2. Plain TS controls require the
selected typed global and component props, reject a discarded package and
missing/wrong props, and preserve `.d.mts`/`.d.cts` declaration graphs. Positive
Vue template roots add Vize acceptance beyond the stock plain-TS oracle. Empty
arrays retain explicit imports, triple-slash type directives and included local
declarations; configured automatic libraries remain absent.

Four pure laws cover all seven complete configurations, failed/missing parents,
within-operation absence, edited next-operation recovery and ancestry-sensitive
cycles. The two standalone public CLI laws compare complete JSON for default and
explicit clean/broken/repair selections: thirty Vize invocations and fifteen
whole official native packets. Fixture dependencies are regular files in each
isolated project. Expected DTOs are authored from the preserved config and stock
packets; actual responses never define or filter expectations.

The existing required-native source qualifier adds only the new Cargo target,
a receipt directory and that directory's upload path. Every existing target,
flag, stage, job, timeout, packet and instruction budget remains intact. The
new law's complete corpus and actual runtime packets are captured when that
required source lane executes. Ordinary disable-TSGO workers provide zero
native-body qualification.

The extracted source helper and four new pure laws pass locally together with
its three existing memo laws; this supplies no compiled public CLI acceptance.
Fresh exact-head source Actions, required native execution and full fresh
protected suites are pending. The broad #3984 project/declaration/build/watch/
LSP/parity/performance roadmap remains open; no legacy provider is replaced.
