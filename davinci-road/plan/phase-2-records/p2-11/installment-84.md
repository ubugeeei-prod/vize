# P2-11 Installment 84 - Module Mode

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5615](https://github.com/ubugeeei-prod/vize/pull/5615), merged
> 2026-09-03 at `e9923c0f8`.

This installment opens the production option surface. Through installment 83
the S2 DOM lane only matched `compile_template`'s *defaults* - function mode,
no prefixing - while production SFC compiles go through
`vize_atelier_sfc::compile_template_block`, which sets module mode,
`prefix_identifiers`, binding metadata, `inline`, `cache_handlers`, `is_ts`
and a component name. A field the emitter does not read is not a default it
may assume; it is production surface the series has not reached, and the
witness for it does not exist.

`DomEmitOptions` is that surface, added here as the growing mirror of
`CodegenOptions`, with its first entry: `DomEmitMode`. Module mode writes
`import { … } from "vue"` instead of the `const { … } = Vue` destructure, and
`export function render(_ctx, _cache)` instead of the six-argument signature -
which returns once binding metadata is present. The runtime module and global
names are options of their own.

The allocation gate `s1_to_s2_emit_p2_11_dom_surface` is exact over the
*default* lane, so every option added here must cost zero allocations while it
is off.
