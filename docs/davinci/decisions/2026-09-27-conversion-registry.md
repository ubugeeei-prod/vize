# L2 to L3 conversion registry slice

Issue: [#6832](https://github.com/ubugeeei-prod/vize/issues/6832).
Original slice: `7735300586c8e3fb656448a7e27002732e05bc40`.
Replayed after stage paths `9eeea29b1a021f0f9852ebb4fcf835338ae8fcdd`.

`vize_l2_to_l3` already exists as a physical workspace package and exports
`lower`. The shared `CONVERSIONS` metadata omitted it even though the
pipeline name mapping already recognized `l2-to-l3`.

Register `L2_TO_L3` with the physical package/dependency name
`vize_l2_to_l3` and endpoints `L2.id` and `L3.id`. Keep conversions in level
order. The existing registry invariant now checks exactly one conversion
for each adjacent artifact-stage edge, rejects nonadjacent/reversed edges
and duplicate identifiers, and derives canonical names from the endpoints.
The original layer-count assertion remains, retaining its invariant and
avoiding an unused test import. Existing Cargo metadata tests independently
check actual package/dependency identities and the one-way graph.

This is registry metadata only. It adds no dependency, compiler stage,
pass execution, or serialization between levels. A registered conversion
does not imply every product reaches L3: DOM can emit directly from L2,
and attempted Vapor/SSR bridges may still decline into retained lanes.

The remaining path/type/string renames, `vize dump` command, real pipeline
binding, and production capture shared with the playground remain unfinished.
They require separate small slices. #6832 stays open; #6833 starts after
#6832 is complete. The CLI/production capture design is preparation, not
implemented functionality.

Original `5ce` verification passed 33 metadata/storage controls and offline
metadata; those historical results retain their source identity. This replay
uses source-only storage/module gates and parsed-manifest mutation checks.
Cargo metadata and Rust execution remain pending Actions; static checks do
not imply executed conversions or newly available runtime artifacts.
