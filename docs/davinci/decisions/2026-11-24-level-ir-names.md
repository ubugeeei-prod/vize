# One name per level IR

The root type of each level is named after its level. The aliases added by
the [2026-11-22 graph-query record](./2026-11-22-ir-graph-queries.md) become
the types' only names:

| Level | Type            | Was                                     | Owns                                                                   |
| ----- | --------------- | --------------------------------------- | ---------------------------------------------------------------------- |
| L1    | `vize_l1::L1Ir` | `container::vue::DescriptorObservation` | The original file: source, container segments, selected blocks, issues |
| L2    | `vize_l2::L2Ir` | `file::FileArtifact`                    | The native file: the sealed template artifact plus script facts        |
| L3    | `vize_l3::L3Ir` | `op::Program`                           | Scheduled operations with state, effect and data edges                 |

This replaces the 2026-11-22 sentence that let existing callers keep the old
spellings. Two names for one type made it hard to see where an IR begins: on
2026-11-24 the old spellings had 609 references and the new ones 32.

The rename is mechanical. The pinned `Debug` form of `L1Ir` still prints
`DescriptorObservation`, because reviewed fixtures record it; that string is
an output contract, not a type name. Dated decision records keep the names
they were written with. A private `DescriptorObservation` in
`vize_atelier_sfc::croquis` is a different type and keeps its name.

Still owed, in separate changes:

- The DOM and SSR compile paths do not build an `L2Ir`. They read
  `vize_l2::artifact::ArtifactParts` through `vize_l1_to_l2::Lowered`, so the
  compile route and the editor route hold different L2 values.
- Each level has many entry functions (`parse*`, 26 `lower*`, 23 `emit_dom*`),
  one per option combination. Each should have one entry taking an options
  value.
- L4 has no IR type. Emitters write strings directly.
