# Lexical dollar bindings in template checking

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

A `$` prefix does not change the lexical owner of a slot prop, loop alias or
handler parameter. The instance-global emitter now checks the expression's
actual Croquis scope before emitting a synthetic component-context read.
Undefined-reference records carry only offsets, so their fallback resolves the
owning expression rather than granting bindings from an enclosing source span.
A loop's iterable uses its parent scope; its own alias cannot excuse an unknown
iterable reference. This reuses existing scope metadata and identifier walks.

The motivating authenticated VueFes packet has 18 `TS2339` diagnostics on valid
`$form`/`$field` slot reads and three independent `TS2307` diagnostics in a TS
barrel. Its source is pinned to gitlink
`8eff3cd7dd79a7f47bbf2b78202b9aa2070ec5a6`; run
[38158172287](https://github.com/ubugeeei-prod/vize/actions/runs/38158172287),
artifact `11709424805`, SHA-256
`098a9bf8fdf5cb2320bace6c5085155cc485a95d0bda1291a6782b2e3e466653`.
The barrel diagnostics, original 81-versus-99 source membership, every original
App golden and the historical differential registrations remain unchanged.

## Executable reproduction

The [fixture directory](../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879)
retains eight complete physical inputs and nine normal-batch observations.
[inputs.json](../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879/inputs.json)
binds every authored byte by size and SHA-256. The focused native law checks
that complete input list and the independently authored full expected vectors,
including file, line, column, severity, code, message and block type, in returned
order. It requires a genuine installed Vue package; a dependency facade refuses
qualification. The normal native-runtime requirement is preserved; local proof
sets `VIZE_TEST_REQUIRE_TSGO=1`, so the actual execution cannot skip.

Enrollment is the ordinary Rust test module chain
`type_checker/tests.rs` → `recent_issues.rs` → `lexical_dollar_slots.rs`.
The fixture contract names that exact registered native law and pins the whole
expected body by size and SHA-256; the law checks both before comparing results.
Run it with `VIZE_TEST_REQUIRE_TSGO=1 cargo test --locked -p vize_canon --lib
lexical_dollar_slots_preserve_complete_native_diagnostics_and_repairs`.
The existing full nextest profile selects every ordinary workspace case and
its merge workflow requires native execution. This additive fixture receives
no historical differential-adapter or full #6879 completion credit. A focused
hosted driver must explicitly execute the law before receiving native credit.

The durable [whole before observations](../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879/before-native.json)
bind the source-only reproduction commit
`5a3e78423ee113a1085bf2527c4057778d511a92`. These observations retain complete
native stdout/stderr/status and diagnostic vectors, without becoming a new
oracle or replacing original snapshots. The production instance source there
is byte-exact main `a6633fb68b110d07abd5ee5a6049bc15f20f9c7f`.

Local native 7.0.2, genuine Vue 3.6.0-beta.10, Rust 1.99.0, and the cached default
Canon test profile produced these complete results:

| Observation                                                      | Before errors | After errors |
| ---------------------------------------------------------------- | ------------: | -----------: |
| valid strict slot, nested slot, loop and handler                 |             6 |            0 |
| wrong typed slot property, strict                                |             7 |            1 |
| wrong typed slot property, permissive                            |             1 |            1 |
| slot outside its body and parent props in either attribute order |             7 |            4 |
| foreign script callback parameter                                |             1 |            1 |
| unknown loop source                                              |             2 |            1 |
| undeclared strict instance global                                |             1 |            1 |
| original permissive instance-global fallback                     |             0 |            0 |
| repaired typed slot                                              |             6 |            0 |

The remaining wrong-property error is exactly `TS2339` at authored `10:61`:
`missing` is absent from the concrete slot payload. A separate installed
vue-tsc 3.3.11/TypeScript 6.0.3 check of byte-identical sources confirms the clean
positive, that typed error and the four parent/outside errors. Context type
names differ between backends; the native vector retains its original names.

Both new producer laws fail before and pass after; all four existing instance
laws remain green. The producer receipt retains the complete summary,
expression scope IDs, scope parents, binding declarations and source spans.
Strict mapping assertions retain every real instance-read authored window.
Seven neighboring strict-template laws and the public-instance guard also pass.
Production-only strict Clippy passes. The wider default-feature test Clippy run
has unrelated existing lint failures; it does not receive a green claim or a
waiver. The new helper formatting follows the existing carton-string policy.

## Receipt authority and delivery boundary

The actual normal factory selected native CLI for these nine cases. Its
optional test-only receipt captures the actual Command arguments and cwd,
executable SHA-256, full authored/generated config, all generated sources/maps,
whole raw native byte streams and actual terminal status. Pending custody is
written before execution; the original IO result and every parser/refusal branch
are preserved. A separate session receipt runs only if that route is actually
used. No session receipt occurred in the local proof, so API/LSP ownership,
native PID, native normalized options and the engine's full loaded-file vector
are explicitly unobserved. Generated input membership does not grant those
claims. The complete local packets remain indexed in the review handoff.

This is correctness evidence. Test receipt instrumentation runs inside the
existing profiling block and supplies no timing or speed credit. Exact-head
Actions, full App requalification, protected merge and release remain pending.
The five original App failures continue to block their existing Stack; this
bounded lexical correction grants no broad provider or golden migration credit.

## Finite source qualification for the original400

The existing warm-provider driver must admit source changes before it can run
its unchanged original400 workload. A separate lexical-dollar qualification
pins the reviewed `adf221274a50272c6fa354df37e78c2ca73615eb` tree's nine complete
Rust bodies against main `a6633fb68b110d07abd5ee5a6049bc15f20f9c7f`, including
four original absences. It also binds every complete new corpus body (21) and
preserves six original400 fixture members and the historical32 manifest.
Unknown, missing, duplicate, changed or falsely absent bodies refuse admission.
The existing path, timing and default-migration qualifications keep their
original outputs; the driver records the additional authority only when it
qualifies. Eligibility, literal-cut checks, dependency locks, all400 inputs,
whole oracles, budgets and actual measurements remain unchanged.

Self-contained source laws retain five full original Rust bodies from actual
Git, plus mode/blob/absence receipts, in a separate test-only host capsule. They
check their full bytes and each actual regular proposed file. These laws need
no historical checkout, network fetch or unavailable-history skip. Runtime
admission still hashes both actual Git source sides through the existing
regular-blob reader. This is finite source enrollment and grants no native,
original400 result, performance, full App or release credit.

## Hosted supplemental enrollment

The existing native-phase driver invokes a purpose-specific lexical helper only
after its original default-prop/imported-runtime/editor receipt is complete.
It requires that same official native 7.0.2 executable and package SHA, successful
version probe and source identity. One cached `ci-opt` Canon library `--no-run`
builder emits the executable receipt; that same executable runs the six exact
producer laws and the one exact nine-observation native law. Default features,
Rust/Cargo 1.99.0, the original profile arguments and required-native environment
are retained. This does not introduce another full Vize builder.

The observer binds its copied helpers, nine Rust bodies, 21 complete corpus
bodies and build inputs to actual no-replace Git HEAD/tree/mode/blob identities.
It retains the complete Cargo artifact/argv/status, test and native binary bytes
by SHA/size, the original native receipt, installed Vue manifest, six test names,
complete producer scope packets and unique ordered `000` through `008` CLI
packets. Each observation must match its complete expected vector, authored
inputs, actual cwd/config/runtime, generated code and nonempty complete source
maps. Whole raw native streams and terminal status must agree with that result.
Generated source-map custody supplies no independently derived mapped semantics.
PID, engine-normalized options and engine loaded-file membership remain explicitly
unobserved. Existing original400, App and performance gates keep their authority.

Separate nonexecuted packet fixtures exercise the observer's complete positive
contract and missing/skipped/partial/wrong-runtime/changed-source/vector/config
refusals. These parser laws execute no native process and grant no native or
performance credit. Hosted lexical execution remains pending until the actual
exact-head run and complete artifact pass these checks.

The first hosted run at `83f4965f8faf6d665bca47bd7571fc5abe27dc29`
executed all six producer tests and all nine native vectors, then refused its
receipt because the configured native path retained a pnpm symlink spelling.
Keep that raw path and require its real path to equal the independently
attested actual Command executable. A missing target, foreign executable or
distinct same-byte copy still refuses custody; the original whole binary,
package, config, source and diagnostic checks remain required.

The complete [4032 native run](https://github.com/ubugeeei-prod/vize/actions/runs/38186287047)
passed all six producer laws and nine native observations with zero ignored
tests. Artifact `11719264412` retains the accepted receipt, all 39 copied source
bodies and complete packets; its ZIP SHA-256 is
`c16d3103c1cadf9e1ea3621f653472cbc7571879e0a8a90d22025282cbb57b85`.
The same head's [original400 run](https://github.com/ubugeeei-prod/vize/actions/runs/38186287043)
also passed all 79 whole comparison pairs. These are exact-source results;
the necessary fresh-main replay must qualify and execute independently before
queue admission. They grant no performance, full App, merge or release credit.

TODO: the whole App/Stack remains blocked independently. The authenticated 7be
full App vector leaves ReKa's three `TS7006` and one `TS2305` errors; Nuxt UI has
534 `TS2308` collisions involving synthetic `Props`/`Emits`/`Slots` exported
through authored star barrels (source attribution only). Prove authored
star-barrel exports against a genuine native baseline and retain real duplicate
authored-export refusals before a separate repair or any golden migration.
No barrel production or test body changes belong to this lexical slice.
