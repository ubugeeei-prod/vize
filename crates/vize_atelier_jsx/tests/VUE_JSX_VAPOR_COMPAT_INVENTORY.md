# vue-jsx-vapor semantic compatibility inventory

This inventory records behavior-level compatibility between Vize's native JSX
Vapor backend and the real `vue-jsx-vapor` package. Both generated modules run
under the same pinned Vue Vapor runtime. Each row mounts both components in
Happy DOM, applies the corpus updates, and compares normalized rendered DOM,
live form state, diagnostics, and element identity.

The first slice intentionally covers four core behaviors. It is not a claim
that the full `vue-jsx-vapor` surface is compatible; issue #6884 remains open
until the corpus covers the complete dialect inventory.

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
| `control-flow/conditional-branch` | equivalent | same-branch updates retain identity; branch changes replace the child   |
| `props/live-input-value`          | equivalent | the live input value and sibling output update without replacing either |

The JSON corpus and verdict table are checked bidirectionally, so adding or
removing a case without a verdict fails. The runtime comparison is the source
of each `equivalent` verdict; generated-code shape is not used as a substitute
for mounted behavior.

Run the focused oracle with:

```text
cargo test -p vize_atelier_jsx --test vue_jsx_vapor_compat_oracle
```
