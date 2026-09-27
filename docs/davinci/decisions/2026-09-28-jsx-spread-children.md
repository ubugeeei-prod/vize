# JSX spread children

Tracked in [#6888](https://github.com/ubugeeei-prod/vize/issues/6888).

## Decision

Native VDOM output spreads `{...items}` into its own Fragment block:
`(_openBlock(), _createBlock(_Fragment, null, [...items], -2 /* BAIL */))`.

- The block registers with the parent block, so parent patching reaches it.
  `BAIL` makes Vue normalize raw children (strings, numbers, arrays, VNodes,
  `false`/nullish) and diff the whole list, like Babel's unoptimized vnodes.
  A bare spread inside an optimized block would neither mount primitives nor
  patch length changes.
- `[...items]` keeps JavaScript iterator semantics: one iteration per render,
  and a `TypeError` for non-iterables.
- The change lives in the JSX lowering and VDOM entry only. `vize_atelier_core`
  is untouched, so SFC output and its instruction ceilings do not move.
- Vapor and SSR have no raw-children form. They keep the stringified value and
  report the gap. JSX inside the spread argument is reported too, because the
  argument is copied as source text.
- Babel compatibility mode keeps its existing `[...items]` output.

The only observable difference from Babel is that server HTML contains the
fragment's hydration markers (`<!--[-->`/`<!--]-->`), because Babel's children
array is flat. Client DOM, updates, keyed identity and hydration otherwise
match.

## Evidence

The fixtures `tests/_fixtures/differential/jsx/spread-child-*.jsx|tsx` are
frozen in `spread-child.fixed-legacy.json`. `tests/spread_child.rs` executes
them through real Vue and `@vue/babel-plugin-jsx`, covering mount, updates,
keyed retention, SSR, hydration and `TypeError` cases.
