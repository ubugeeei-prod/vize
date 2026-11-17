# SFC templates keep their authored structure

Vue 3.5's compiler builds a component template as written. It warns in
development about invalid HTML nesting but never repairs it. Vize's default
`templateSyntax: "standard"` applied the browser's HTML tree construction to
SFC templates. `<form @submit="a"><form @submit="b">`, `<p><div>`, `<p><ul>`
and `<p><p>` failed to compile, `<table><div>` failed for SSR, and nested `<a>`
and `<button>` and omitted `<li>` end tags were split into siblings. Vue
compiles all of them as written, so drop-in users such as shadcn-vue broke.
This record extends the
[authored table rows record](./2026-11-18-sfc-table-authored-rows.md) from
tables to every tree-construction fix-up. It supersedes that record's clause
that foster parenting and the closing of an open row or cell do not change:
for a component template they are gone too.

## Decision

A component template keeps its authored element structure in every template
syntax mode, on both the legacy parser and the native L1 parse. The legacy
parser (`vize_armature`) applies these fix-ups only in document mode, for the
Vue 1/2 dialects or under `ParserOptions::html_tree_construction`:

- implied end tags and HTML scopes,
- the nested `<a>`/`<button>` split,
- the ignored nested `<form>`,
- the adoption agency,
- table parts,
- foster parenting.

Every Vue 3 component parse on the native side builds the authored tree too:
the compile parse (`vize_l1::markup::parse_with_options_and_caps`), the
component surface (`parse_component*`, `parse_with_authored`) and the native
lint tag facts that the formatter and Patina's native rules read. Only
`SurfaceParseOptions::html_tree_construction` asks for the repaired tree. The
legacy and native paths stay byte-identical with each other, and their repaired
lanes still agree with each other.

The repairs remain where the browser really parses the markup. Documents keep
them, and so does Patina's in-DOM parse, unchanged by this record.

### Vapor keeps the browser's repairs

Vapor sets both switches because `innerHTML` parses its `template()` strings
again, so its output is byte-identical to before. A nested `<form>`,
`<p><ul>`, `<p><div>`, `<p><p>` and `<table><div>` still fail to compile
there, and a nested `<a>` or `<button>` is still split, exactly as before.
Vue 3.6 Vapor instead splits each invalid nesting into its own template and
inserts it, keeping the authored tree. Adopting that split is the follow-up
[#13310](https://github.com/ubugeeei-prod/vize/issues/13310).

### Diagnostics

Invalid nesting stays a lint concern (`vue/permitted-contents`); the
compiler adds no nesting warning. A misnested formatting end tag
(`<b><i></b></i>`) still fails, now with Vue's missing-end-tag and
invalid-end-tag errors instead of the adoption-agency notice.

## Known difference: omitted optional end tags

An omitted optional end tag (`<li>a<li>b`, `<dt>a<dd>b`, `<option>a<option>b`,
`<td>a<td>b`, `<div><p>a<div>b</div></div>`) nests as in Vue, so the compiled
tree and markup are the same. Vue's compiler also reports "Element is missing
end tag." for it as an error; Vize reports nothing. Vize accepts what Vue
rejects, which is the less harmful direction, and `vue/html-end-tags` reports
the omission in lint.

## Evidence

- `tests/_fixtures/differential/compiler/sfc-authored-structure` pins the SSR
  module for a nested `<form>`, `<a>` and `<button>`, `<p><ul>`, `<p><div>`,
  `<p><p>` and `<table><div>`; its markup equals Vue 3.5.35's.
- `crates/vize_atelier_sfc/tests/sfc_authored_structure.rs` compiles that case
  and each shape for DOM and SSR against Vue 3.5.35's markup and tree, pins the
  omitted optional end tags and the misnested formatting errors, and pins
  Vapor's unchanged, repaired output.
- Parser pins: `vize_armature::parser::tests::authored_structure` (component
  versus repaired lane) and the document-mode expectations in
  `crates/vize_armature/src/parser/tests.rs`.
- Native pins: `davinci/vize_l1/tests/implied_end_tags_legacy_oracle.rs` (both
  parsers agree in the authored and the repaired lane) and
  `davinci/vize_l1/tests/authored_projection.rs`.
