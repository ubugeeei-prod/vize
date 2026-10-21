# vue-jsx-vapor semantic compatibility inventory

This inventory records behavior-level compatibility between Vize's native JSX
Vapor backend and the real `vue-jsx-vapor` package. Both generated modules run
under the same pinned Vue Vapor runtime. Each row mounts both components in
Happy DOM, applies the corpus updates, and compares normalized rendered DOM,
live form state, diagnostics, and element identity.

The current bounded cohort covers eight behaviors. It is not a claim that the
full `vue-jsx-vapor` surface is compatible; issue #6884 remains open until the
corpus covers the complete dialect inventory.

## Pinned oracle

| Dependency      | Version       |
| --------------- | ------------- |
| `vue-jsx-vapor` | `3.2.26`      |
| `vue`           | `3.6.0-rc.10` |

Versions live in the `jsx-vapor-oracle` pnpm catalog. A version bump requires
a reviewed update to the executable verdict inventory.

## Verdicts

| Case                              | Verdict    | Mounted behavior                                                        |
| --------------------------------- | ---------- | ----------------------------------------------------------------------- |
| `children/dynamic-text`           | equivalent | initial DOM and three text updates retain the authored element          |
| `props/dynamic-attributes`        | equivalent | attributes and text update while the authored element remains live      |
| `events/click-closure`            | equivalent | two real clicks run the captured closure and retain the button          |
| `children/logical-and`            | divergent  | only vue-jsx-vapor replaces the branch across truthy value changes      |
| `control-flow/conditional-branch` | equivalent | falsy, array and nested-VNode conditions match both real JSX references |
| `props/live-input-value`          | equivalent | the live input value and sibling output update without replacing either |
| `directives/v-model-text`         | equivalent | input events and direct state patches preserve the live model binding   |
| `components/scoped-default-slot`  | equivalent | component props and a scoped slot closure update independently          |

The JSON corpus and verdict table are checked bidirectionally, so adding or
removing a case without a verdict fails. The runtime comparison is the source
of each `equivalent` verdict; generated-code shape is not used as a substitute
for mounted behavior.

The event row dispatches two bubbling, cancelable `click` events through the
mounted button. Its receipt pins dispatch success, `defaultPrevented`, the
closure's rendered counter side effect, diagnostics, and element identity for
every phase before checking complete unmount.

The model row assigns a real input value before dispatching each bubbling,
cancelable `input` event, pins the complete dispatch and live-input receipt,
and interleaves a direct state patch. Both directions must update the mounted
input and sibling output without replacing any authored element.

The component row resolves a registered VDOM fixture through each Vapor
compiler's component fallback, passes it a reactive prop and a scoped default
slot, and updates the prop and captured slot state independently. The mounted
article, prop output, slot container and authored slot element must all retain
identity while their complete DOM and diagnostics remain equal.

The logical-and and conditional rows are the value-sensitive counterparts of
the Babel oracle's equivalent `children/logical_and` and `children/ternary`
rows. Their mounted update sequences pass `0`, an empty string, `NaN`, `null`,
`undefined`, `false`, `true`, an array and a nested VNode through both
Vize/reference pairs. The comparison retains complete DOM, visible text,
diagnostics and branch identity separately for the VDOM and Vapor runtimes.
The logical-and references agree on DOM and text but differ in identity: Babel
retains the truthy branch, while `vue-jsx-vapor` replaces it when the truthy
condition value changes. The executable verdict pins that identity difference
while still requiring identical DOM, text and diagnostics across all phases.

Run the focused oracle with:

```text
cargo test -p vize_atelier_jsx --test vue_jsx_vapor_compat_oracle
```
