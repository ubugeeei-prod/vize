# S1 Folio Format (`s1-page@1`)

The S1 page is the lossless surface tree (`vize_s1::SurfaceTree`) as a folio
page: `vize_extension_host::SurfacePage`, hand-written under the
[folio contract](./folio-format.md) and carried by the input-dialect WIT world
(`contracts/wit/input-dialect.wit`, P6-1a) as `lowered-block.surface` with
`schema-version: 1`. `Display` prints the same text as `Full`: the page holds
only offsets, so nothing is elidable.

## Why offsets, not text

Every S1 string is a slice of the block source, and in canonical render
order the slices tile the source exactly (TS-19). The page therefore records
structure and **block-relative byte offsets only**. A token is
`start:text:end`: `start..text` is its verbatim `leading` gap, `text..end` its
own bytes. A reader that holds the block source rebuilds the tree
(`SurfacePage::materialize`) after checking that the tokens tile it
(`SurfacePage::check_tiles`), so a guest can neither drop nor invent a byte,
and no escaping rule can lose one. S2 page spans stay file-absolute; the
block's `base` is the offset between the two.

## Grammar

Two sections, fixed order: `[s1]` with the single field `bytes=` — the
printer's computed statement of the last token's end (parse validates the
integer and discards it) — then `[s1.tree]`, omitted when the tree is empty.
Nesting is two-space indentation. A token line is `<role> <start>:<text>:<end>`
with a trailing ` missing` for a typed hole (zero-width, `text == end`).

| line                                                         | children, in order                                                                                                                                                      |
| ------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `element`                                                    | `lt-name`, then `attr`\*, then `slash`?, then `gt`, then child nodes\*, then exactly one close: `close-tag` / `close implicit` / `close missing` / `close not-expected` |
| `attr`                                                       | `name`, `eq`?, then either nothing or `open-quote`? `value` `close-quote`?                                                                                              |
| `close-tag`                                                  | `lt-slash-name`, `gt`                                                                                                                                                   |
| `interpolation`                                              | `open`, `content`, `close`                                                                                                                                              |
| `text` / `comment` / `cdata` / `pi` / `unexpected` + a token | none (leaf nodes)                                                                                                                                                       |

The close variants and hole tokens are `vize_s1`'s hole policy one to one:
`close missing` is `ElementClose::Missing`, `unexpected` is
`SurfaceChild::Unexpected`, a `missing` token is `TokenStatus::Missing`.

## Laws and refusals

- `parse(print(page)) == page` and `print(parse(text)) == text` for canonical
  text; `materialize` renders the source bytes back and mirrors to the same
  page. Pinned over the TS-19 battery and every prefix and suffix truncation
  of it by `crates/vize_extension_host/tests/surface_page_laws.rs`, which also
  pins a reference page and every parse refusal with its exact message and
  1-based line.
- Parse is lenient exactly where the printer normalizes (blank lines, the
  `bytes=` value, integer spellings) and strict elsewhere. Tiling is checked
  against the source, not by parse: a gap, a split UTF-8 sequence, an
  out-of-order or wide-hole token, and leftover bytes are each refused with
  an exact `TileError`.
- A host accepts a guest's page only when it is canonical and tiles the block
  (`vize_extension_host::accept`).
