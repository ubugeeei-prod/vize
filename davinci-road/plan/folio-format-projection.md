# Projection Folio Format (`projection-page@1`)

The projection page is a dialect's checkable projection of one block's
expressions with its span links: the P4-5a `ProjectionMapping` rows as a
folio page. It is `vize_extension_host::expression::ProjectionPage`,
hand-written under the [folio contract](./folio-format.md) and carried by the
expression-dialect WIT world (`contracts/wit/expression-dialect.wit`, P6-1b)
as `analysis.projection` with `schema-version: 1`. Its sibling,
`analysis.facts`, is the `expression-facts` α document
([fact-alpha-schemas.md](./fact-alpha-schemas.md)).

## Grammar

```text
[projection]
rows=<count>

[projection.rows]
row <gen-start>:<gen-end> <src-start>:<src-end> <kind> <features>
  sub <gen-start>:<gen-end> <src-start>:<src-end>

[projection.text]
<the generated text, verbatim to the end of the page>
```

- Generated ranges index the text; authored ranges are file-absolute (a
  `ProjectionMapping` with `authored_base` 0).
- `<kind>` is a `ProjectionSpanKind` in kebab case (`directive-expr`,
  `interpolation`, ...); `<features>` is `all`, `none`, or the enabled
  `ProjectionFeatures` names comma-joined in bit order.
- `[projection.rows]` is omitted when `rows=0`; `sub` lines follow their row.
- The text section is terminal and verbatim, so any generated text (newlines
  included) round-trips byte for byte.

## Laws

`Display` prints the same text as `Full`: the page holds only offsets, kinds,
flags and the text, so nothing is elidable. The host accepts only canonical
pages: a page whose reprint differs is refused with the first differing byte.
Beyond parsing, acceptance holds every row inside the generated text (on
character boundaries) and inside one batch expression's authored span, and
every sub-span inside its row. Each row converts one to one into a
`ProjectionMapping` entry (`push_with(VizeMapping, ProjectionMeta)`), which
`tests/expression_contract.rs` pins.
