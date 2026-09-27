# Fixed v-for provenance construction (#6868)

This is a private, unpushed source candidate based on
`b3e3f944a11d2f28cfc928eac980503fae866e7f`, which includes the separately
reviewed Patina empty-binding optimization. It changes only fixed provenance
construction in L1-to-L2 lowering. It does not add the proposed ASCII expression
classifier or change any stack guard.

## Decision and scope

`lower/cx/for_parts.rs` builds `fact value=... key=... index=...` using
`vize_l0::String::with_capacity` and `push_str`. Capacity is the exact sum of
the UTF-8 byte lengths of all literals and `spell()` values. Pending `?`, absent
`-`, name bytes, provenance ownership/order/node/span, and fact-map insertion
remain unchanged. Numeric `scope #N bindings=M` formatting remains unchanged.

`lower/forop.rs` applies the same construction to `ui.for source=... value=...`.
Both existing `desc()` calls remain, including their opaque/foreign spellings.
This second bounded site reduces more real lowering work without shifting it
outside the measured window. Neither change adds a pipeline stage or serialization.

The frozen #6906 failure is `150340` in all three executions versus cap
`148047`. Its complete `+2293` is under the first `pthread_getattr_np` stack-limit
query: one extra `/proc/self/maps` line, with `sscanf` and `getline` calls
`56 -> 57`. Vize/OXC self costs and call counts are unchanged. The additional
mapped object is unknown because maps/ELF snapshots were not archived. Root/shallow
stack guards stay intact; warming them outside the window is not part of this change.

Frozen formatting costs are `2004` inclusive Ir for two `attach_for_parts`
formatting calls and `730` for the `lower_for` formatting call. Only the fixed
text calls change. These costs are not savings estimates: whether this candidate
recovers the excess is unknown until the unchanged Actions measurement runs.

## Local source evidence

- `rustfmt --edition 2024 --check` passes for both production files and the new
  `tests/vfor_provenance.rs` integration test.
- The actual assertion scanner reports zero findings in the new test, with no
  allowlist change. Eight authored controls compare complete three-record
  provenance vectors: named positions, destructuring, absent positions, Unicode,
  long ASCII/Unicode ownership, and pending key positions. A two-scope control
  compares the complete six-record vector, including page order and every span.
- Both production edits invert byte-exactly to the base outside their fixed text
  construction and necessary imports. All touched source/docs files stay below
  350 lines.
- All 35 protected raw paths match the base, including every benchmark source,
  both verification workflows, driver/library/harness, both budget registries,
  expression guard, and stack guard. The instruction budget SHA-256 remains
  `17c3947044c033d03b0a86d13b2c3fa6dc2d1e90cb2222033921f649e97a826f`.
- The 100 IDs, raw input digests, measured windows, protocol and caps are unchanged.
  No benchmark source, fixture, baseline, ceiling, counter ID or workflow was edited.

No local Cargo build, Rust test execution, dependency install, publication or
workflow dispatch was performed. Rust runtime correctness and numeric acceptance
remain pending. Source-authored expectations are not regenerated runtime captures.

## Required Actions evidence

After root review, the owner may verify one temporary ref with unchanged
`check.yml` (full Rust tests and differential corpus) and
`level-instruction-counts.yml` (actual 100 probes, three executions each).
The new provenance controls, existing v-for/loop snapshots and small-stack
recursion control must pass. All 100 probes must satisfy existing ceilings.
No numeric acceptance, public-eight restack or gate exception follows from these
local checks. Further optimization, if needed, requires a separate reviewed change.

The paired issue record is on #6868; its URL is recorded after posting the source
candidate. The central Performance section links this companion.
