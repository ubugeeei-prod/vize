# Existing legacy Vapor style fixture runtime coverage

Paired decision: [#7600 comment](https://github.com/ubugeeei-prod/vize/issues/7600#issuecomment-5982783345).
The report remains closed; this change strengthens its existing regression.

The audit source is immutable main `61c975f8889a002a1b53265d86d0985447ffab63`.
At that source, `tests_static_style.rs` already verifies the reporter's
static-first transformed IR, preserve/recovery output and JavaScript syntax.
The ecosystem tool matrix verifies default DOM CLI JSON/output paths, while
the source-bound compiler corpus checks SSR CLI and SFC/SSR/Vapor API output.
Neither supplies a runtime style update witness for this reporter fixture.
The app conformance lane compiles SFCs with its default options; native runtime
packs cover their own authored inputs and do not establish this retained case.

## Bounded E2E

The test reads the complete existing
`crates/vize_atelier_vapor/tests/fixtures/static-style-merged-binding.input.txt`,
including its final newline. Its SHA-256 is
`c2ff52714553b478eccdfc036277e0e932a3ce71cbc4b17731a5adeac0c2e983`.
No fixture, reference output or current 32-case differential input changes.

`style_binding_runtime` is a test example compiled from the candidate source
with `cargo run --locked --profile ci -p vize_atelier_vapor`. It invokes the
public `compile_vapor` API with prefix identifiers and the explicit retained
selector, once for each condense/preserve whitespace strategy. This does not
change published defaults or introduce a production pipeline stage.

Each complete generated module runs unchanged in its own Node process. The
only module hook resolves `vue` to the original pinned Vue 3.6.0-rc.9 browser
runtime. DOM globals are installed before that runtime loads. Real reactive
updates and `nextTick` establish these independent expectations:

- Static red coexists with the initial dynamic blue background.
- Dynamic green overrides red and removes the old background.
- Empty and null dynamic values restore red and clear the background.
- Every update retains the original div and its `x` text.
- Vue emits no warning/error, and unmount disconnects all rendered nodes.

The parent test requires both compiler modes, complete original source bytes,
empty compiler diagnostics, successful child exits and empty child stderr.
Actions logs retain the full compiler packets and runtime receipts, including
the SHA-256 of each executed module. The existing tooling planner discovers
the test for source PRs and the complete merge suite without an optional skip.
Exact-head Actions execution, protected checks and actual merge remain pending.

## Limits and remaining work

This is a legacy runtime regression, with zero native acceptance credit.
happy-dom does not prove Chromium behavior, hydration, full Vue parity or
completion of [#6880](https://github.com/ubugeeei-prod/vize/issues/6880).
Broader compiler history/runtime fixtures and all-product E2E remain open work.
The published compiler output, dependencies, allocation/instruction ceilings
and original differential corpus stay intact. Source readiness grants no
publication or queue admission while the coordinated release hold is active.

The genuine #7600 reporter is Danila Poyarkov (`dannote`); GitHub's public user
record verifies `dev@dannote.net`. The source commit carries that coauthor.
