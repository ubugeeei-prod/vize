# P2-11 increment — `vue.once` / `vue.memo` ops (no DOM emit)

**Not a P2-11 completion.** The task stays open. This is the op +
admission increment P2-9 installment 5 named: `v-once` / `v-memo` were
measured codegen-only (`has_v_once`, `get_memo_exp`); their ops land
with the stage that reads them. The ops exist; realization does not.

## Why `vue.*`, not `ui.*`

P2-16's fairness litmus: a lint written against `ui.*` must run
unchanged on SFC and JSX. One-shot / dependency-memoized render is
Vue's (`v-once`, `v-memo`); there is no JSX twin. Same home as
`vue.css-bind` and `vue.sync`.

## Ops

| mnemonic   | payload                                      | 64-bit size |
| ---------- | -------------------------------------------- | ----------- |
| `vue.once` | `VueOnceOp { span }` — presence flag         | 8           |
| `vue.memo` | `VueMemoOp { value: ExprRef, span }`         | 24          |

`v-memo`'s expression is P2-5b: admitted JS or `opaque` (the fixture
pins `opaque(parse-rejected "%")`). No new `OpaqueReason`. Drop-free
arena types; size asserts are `#[cfg(target_pointer_width = "64")]`.

## Admission

Well-formed spellings retire `defer.v-once` / `defer.v-memo`:

- bare `v-once` (no argument, modifier, or value) → `vue.once`
- `v-memo="…"` with a non-empty value and no argument or modifier →
  `vue.memo`

Ill-formed spellings still `defer.*` (Info), with messages that name
the well-formed shape. `v-html` / `v-text` / `v-show` / `v-cloak` /
`v-pre` stay deferred.

## Canary, proved by injection

Variants injected into `BindingOp` first. The crate's own exhaustive
match broke before the canary test: E0004 in
`crates/vize_disegno/src/folio/owned/binding.rs` (`own_binding`) —
`&BindingOp::VueOnce(_)` and `&BindingOp::VueMemo(_)` not covered.
That is the P2-5a ritual (folio `of` first). After those arms (print,
parse, verifier walk, ricalco pass matches) the canary in
`tests/op_family.rs` is exhaustive again, no `_` arm.

## Folio

Grammar in [`folio-format.md`](../folio-format.md):

```text
vue.once @s:e
vue.memo value=<expr> @s:e
```

TS-16 pin: `crates/vize_disegno/tests/folio_once_memo.rs` (exact Full
text). Exact rejections: trailing content on `vue.once`, missing
`value=` on `vue.memo`.

## Emit, deliberately left

`crates/vize_ricalco/src/emit/**` was not touched. `admit_bindings`
still hits `_ => Unsupported` for these bindings. Tests in
`tests/emit_static.rs` pin that `<div v-once>` and `<div v-memo="[id]">`
refuse. Realization (`has_v_once` / `get_memo_exp` / cache wrappers)
is a later P2-11 installment.

## House

New files ≤ 350; no `mod.rs`; no `impl Drop`; disegno / ricalco stay
`no_std + alloc`. Lowering lives in `lower/once_memo.rs` so
`binding.rs` stays under the budget.
