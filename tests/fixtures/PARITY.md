# Core-directive backend parity matrix (issue #1161)

This table documents which core directives / built-in components are exercised
per backend, and where backend behavior intentionally differs. The shared input
matrix comes from issue #1161.

The fixtures live in:

- VDOM: `tests/fixtures/vdom/parity-core-directives.pkl`
  (expected: `tests/expected/vdom/parity-core-directives.snap`)
- Vapor: `tests/fixtures/vapor/parity-core-directives.pkl`
  (expected: `tests/expected/vapor/parity-core-directives.snap`)
- SSR: `crates/vize_atelier_ssr/tests/ssr_snapshot.rs`
  (insta snapshots in `crates/vize_atelier_ssr/tests/snapshots/`)

All three are run by the existing compiler test suite / CI: VDOM and Vapor via
the `coverage` runner (`cargo run -p vize_test_runner --bin coverage`), SSR via
`cargo test -p vize_atelier_ssr`.

| Input                                                | VDOM | Vapor                 | SSR   |
| ---------------------------------------------------- | ---- | --------------------- | ----- |
| `<div v-pre :id="raw" @click="raw">{{ raw }}</div>`  | yes  | yes                   | n/a\* |
| `<div v-cloak>{{ msg }}</div>`                       | yes  | yes                   | n/a\* |
| `<div v-focus:top.lazy="value" />` (element)         | yes  | yes                   | n/a\* |
| `<MyComp v-focus:top.lazy="value" />` (component)    | yes  | KNOWN FAILURE (#1161) | n/a\* |
| `<div v-once>{{ msg }}</div>`                        | yes  | yes                   | n/a\* |
| `<div v-memo="[id]">{{ msg }}</div>`                 | yes  | not applicable\*\*    | n/a\* |
| `<Teleport to="body"><span /></Teleport>`            | yes  | yes (VaporTeleport)   | yes   |
| `<KeepAlive><component :is="current" /></KeepAlive>` | yes  | yes (VaporKeepAlive)  | n/a\* |
| `<Suspense><AsyncComponent /></Suspense>`            | yes  | yes (fallback)        | yes   |

`yes` = a parity snapshot asserts the compiled output.

\* SSR has its own dedicated snapshot surface and a different output model
(string-concatenation render fns). The directive-payload cases (`v-pre`,
`v-cloak`, custom directives, `v-once`, `v-memo`, `KeepAlive`) are runtime/VDOM
concerns that do not have meaningful SSR-specific output, so they are not
duplicated in the SSR matrix. `Teleport` and `Suspense` use dedicated SSR
helpers and are already covered in `ssr_snapshot.rs`
(`teleport_uses_ssr_helper`, `suspense_uses_ssr_helper`).

\*\* `v-memo` is a VDOM block-caching optimization with no equivalent in the
Vapor backend (Vapor uses fine-grained effects instead of memoized vnode
sub-trees), so it is intentionally omitted from the Vapor matrix rather than
marked as a failure.

## Tracked known failure

`vapor/parity-core-directives :: parity custom directive on component (payload
loss)` is registered in `tests/vize_test_runner/src/coverage.rs`
(`KNOWN_FAILURES`). When a custom directive is applied to a **component**, the
Vapor backend currently drops the directive entirely: it emits the component
creation with no `_resolveDirective` / `_withDirectives`, losing the binding
value, argument, and modifiers. The expected snapshot encodes the desired
payload-preserving output (mirroring the VDOM backend) and therefore fails
against today's compiler. This is intentional per the #1161 acceptance criteria
("at least one snapshot fails on current Vapor custom-directive payload loss
before the implementation fix"). The element-level custom-directive case
(`v-focus:top.lazy` on a `<div>`) already preserves its payload and passes.
