# Fact-group α schemas

Every fact group with an α form (P4-2, [`vize_davinci::fact::alpha`](../../crates/vize_davinci/src/fact/alpha.rs))
has one row here: its group name, its `fact::ids` id, its α schema version,
and what its α page's keys and values mean. The α page is what the P5-2
per-SFC summary serializes, so this table is the summary's contract.

## Rules

- **One row per group, before the code lands.** A group joins
  `fact::alpha::ALPHA_GROUPS` in the same change that implements
  `AlphaExport` for it, and `tests/fact_alpha` fails when a registered group
  has no row here, or a row whose id or version disagrees with the code
  (`check_alpha_schema_doc`).
- **Versions move independently.** Bump a group's `alpha_schema` whenever its
  page's shape or meaning changes — never another group's, never in bulk.
- **Owned, versioned, printed.** α values are owned and `'static` (the P1-11
  arena/cache contract: an α page never borrows an arena or a source). Every
  page prints inside the `[fact-alpha]` header carrying `group=` and
  `schema_version=`; a page read under another version is refused, never
  migrated silently.
- **Keys are identities, not positions.** Keys name what the fact is about
  (a binding name, a `SymbolId`, a resolved component identity), never a
  source offset that an edit above the block would shift (P5 key rule).

## Groups

| group              | id  | alpha_schema | key                                                                          | value                                                                      | status             |
| ------------------ | --- | ------------ | ---------------------------------------------------------------------------- | -------------------------------------------------------------------------- | ------------------ |
| `bindings`         | 1   | 1            | binding name as authored (`String`); `SymbolId` once S2 carries script scope | binding kind (`BindingType` spelling) and declaration span                 | planned — P4-3a    |
| `undefined-refs`   | 2   | 1            | identifier name (`String`)                                                   | reference count and first-reference span                                   | planned — P4-3a    |
| `expression-facts` | 12  | 1            | expression id the host assigned in the batch (`u32`)                         | referenced bindings (comma-separated, source order), exactness, const-ness | registered — P6-1b |

A `planned` row fixes the key and value shape a wave implements; the wave
flips it to `registered` in the change that adds the group to
`ALPHA_GROUPS`. Groups without an α form (β-only, never crossing a compile
boundary) have no row.
