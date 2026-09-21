# Cross-component HTML conformance

Every template in `src/` is valid HTML on its own. Composed, three of them are
not — and the browser, which re-parses the server-rendered page, builds a
different DOM than Vue expects:

| Parent (`src/App.vue`)        | Child root                   | What the browser does                                                   |
| ----------------------------- | ---------------------------- | ----------------------------------------------------------------------- |
| `<p> … <InfoCard /></p>`      | `InfoCard.vue`: `<div>`      | closes the paragraph before the `<div>`                                 |
| `<table><PriceRow /></table>` | `PriceRow.vue`: `<tr>`       | inserts an implied `<tbody>` around the row                             |
| `<a href> … <LinkButton />`   | `LinkButton.vue`: `<button>` | keeps it, but an interactive `<button>` inside a link is non-conforming |

No per-file check can see these. `vize lint --cross-file` composes the
templates through each parent's imports and reports every collision at the
usage site, naming the child's root element and its position:

```bash
cd examples/html-conformance
vize lint --cross-file 'src/**/*.vue'
```

`SafeBadge` (a `<span>` inside the same `<p>`) is conforming and stays silent.
The exact report is pinned by `crates/vize/tests/lint_cross_component_cli.rs`.
