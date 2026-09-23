# P2-11 Installment 115 - S2 DOM In-Tag Comments

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5803](https://github.com/ubugeeei-prod/vize/pull/5803), merged
> 2026-09-06 as `bcf9616e1`.

This installment routes experimental `//` opening-tag comments through the S1
parse option and into S2 DOM emit selection. The comments remain parser trivia:
they are preserved in S1 source fidelity, but they do not become DOM comment
nodes or `_createCommentVNode` calls in the S2 render output.

The witness set is split at the stage boundary. `vize_s1` pins byte-fidelity for
in-tag comments without source holes, `vize_s1_to_s2` proves the parse option
does not project those trivia comments into comment vnode output, and
`vize_atelier_dom` selects S2 codegen for both ordinary output and source-map
requests by attaching the compatibility map around the matching S2 render.

The executable evidence is:

- `crates/vize_s1/tests/surface_fidelity.rs` covers the in-tag comment
  byte-fidelity cases.
- `crates/vize_s1_to_s2/tests/emit_comments.rs` covers the S1 parse option and
  absence of comment vnode output.
- `crates/vize_atelier_dom/tests/davinci_s2_production_selector.rs` covers the
  S2 production selector case.
- `crates/vize_atelier_dom/tests/davinci_s2_source_map.rs` covers source-map
  requests on the same surface.
- `tests/tooling/davinci-dom-production-boundary.test.ts` and
  `tests/tooling/davinci-storage-policy.test.ts` keep the production-boundary
  and storage-policy ledgers aligned with the shipped surface.

This installment does not tick P2-11. The full production-lane switch remains
open because unsupported option shapes and the explicit legacy flag still
require the compatibility path.
