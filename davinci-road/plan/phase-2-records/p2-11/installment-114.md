# P2-11 Installment 114 - S2 DOM Source Maps

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5798](https://github.com/ubugeeei-prod/vize/pull/5798), merged
> 2026-09-06 as `99ee05a1b`.

This installment lets source-map requests enter the S2 DOM production selector
without pretending S2 already owns structured mapping spans. The S2 emitter
still produces the render module bytes and section boundaries; source maps are
attached around that result by borrowing the compatibility generator's map
only after the compatibility render output proves byte-for-byte and
section-for-section equality.

## What changed

`compile::source_map::attach_compat_map` is the bridge. If `source_map` is
off, the S2 result is returned untouched. If a map is requested, compatibility
codegen runs once to build the existing map contract. The S2 result keeps that
map only when preamble, code and section boundaries match; otherwise the whole
compile falls back to compatibility output.

That keeps P3-9's structured S4 source-map work honest. P2-11 may route
source-map compiles through S2 for code generation, but it does not claim that
S2 has become the source-map author.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_source_map.rs` compares template and
SFC section compiles with source maps enabled against the compatibility lane,
asserts identical preamble, code, sections and map payloads, and checks that
the selected compile records one `davinci.s2_dom.files` counter. The parsed map
must remain version 3 with non-empty mappings.

`crates/vize_atelier_dom/tests/davinci_s2_production_selector.rs` also covers
the profiled production selector: source-map requests keep producing a map and
use S2 once the map is verified against compatibility codegen. The tooling
boundary classifies `source_map` as handled around S2 rather than projected
into `DomEmitOptions`.

This installment does not tick P2-11. The full production-lane switch remains
open because experimental in-tag comments, unsupported option shapes and the
explicit legacy flag still require the compatibility path.
