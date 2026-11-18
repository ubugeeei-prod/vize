# SFC templates keep authored table rows

Vue 3.5's compiler emits `<table><tr>` as written: no `tbody` is implied for a
row or cell placed directly in a `<table>` (it only warns in development that
the pair is invalid HTML nesting). Vize used to insert an implied `tbody` (and
a `tr` for a direct cell), so `table > tr` selectors, `$el.children` and the
hydration of SSR output that Vue itself emits as `<table><tr>` differed from
Vue. Nothing documented that as intentional. This record supersedes the table
clauses of the [native `vue:` component record](./2026-10-09-native-vue-is-component.md)
("ordinary and plain-`is` rows retain the existing HTML tree normalization",
"both attribute orders retain the native `tbody`") for component templates.

## Decision

A Vue component template keeps its authored table structure on every target
that builds the DOM node by node: DOM render functions and SSR strings. The
legacy parser (`vize_armature`) and the native L1→L2 lowering
(`vize_l1_to_l2::lower::table`) stop inserting the implied owners together, so
the two paths stay byte-identical with each other and with Vue 3.5.

The owners are still implied where the HTML parser really reads the markup:

- **Documents.** The legacy document-mode parser is unchanged, and Patina lints
  a standalone HTML document through `L2Template::lower_in_dom`, which keeps
  the previous L2 owners.
- **Vapor.** `template()` strings are parsed again by `innerHTML`, which puts a
  direct row in a `tbody`; child paths must see that owner. Vapor's legacy
  parse sets `ParserOptions::implied_table_sections` and its L3 bridge lowers
  with `LowerPolicy::with_implied_table_sections`, so Vapor output is
  unchanged. Vue 3.6 Vapor instead splits an invalid nesting into its own
  template and inserts it (`table > tr` in the DOM); adopting that split is a
  separate decision.

Rows and cells under an authored section keep the earlier rule
([#12387](https://github.com/ubugeeei-prod/vize/pull/12387)): no implied `tr`.
Foster parenting, the closing of an open row or cell by the next one, and Vue's
`vue/permitted-contents` silence for `<tr>` in `<table>` do not change. Vize
does not emit Vue's development-only nesting warning, before or after this
change.

## Evidence

`tests/_fixtures/differential/compiler/table-authored-rows` pins the SSR module
for direct rows, `v-for` rows, `<template v-for>` rows, a direct cell and
authored sections; `crates/vize_atelier_sfc/tests/table_authored_rows.rs`
compiles it through the public SFC API for DOM, SSR and Vapor. The parser and
lowering pins are `vize_armature::parser::tests::table_rows` and
`davinci/vize_l1_to_l2/tests/emit_table.rs`.
