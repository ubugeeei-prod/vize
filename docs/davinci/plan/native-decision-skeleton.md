# Native decision ownership skeleton (2026-09-28)

Tracked in [#6839](https://github.com/ubugeeei-prod/vize/issues/6839).
The maintainer requested concrete files and explicit `todo!()` producers
before feature implementation, with ownership rearchitecture taking priority.
This change establishes that native boundary; it does not finish #6839.

| Responsibility                                           | Native owner                                   | State                                               |
| -------------------------------------------------------- | ---------------------------------------------- | --------------------------------------------------- |
| Semantic tree and binding scopes                         | `vize_l2::op`, `vize_l2::scope`                | Existing artifact; unchanged                        |
| Static, dynamic-binding, placement and control decisions | `vize_l3::decision`                            | Native type skeleton                                |
| Per-target eligibility criteria                          | `vize_l3::decision::policy::{dom, ssr, vapor}` | Target identities only; criteria unfinished         |
| Borrowed L2 to shared L3 decisions                       | `vize_l2_to_l3::build_decisions`               | Explicit `todo!()`; no production caller            |
| Flat reactive program                                    | `vize_l3::op::Program`                         | Existing artifact; demand-only split unfinished     |
| Decision encoding, helper numbering and emission         | Future `vize_l4`                               | Separate #6840 ownership work; not implemented here |

`DecisionTables` uses the existing `NodeId` and `SideTable` infrastructure.
Keys belong to the L2 artifact, including attached binding ids; they are
not flat-program `OpId`s. Dynamic bindings retain authored order, and
control records identify containment. The existing `Placement` vocabulary
is reused instead of creating a second hoist/cache/group enum. Target policy
identities select DOM, SSR or Vapor criteria; no eligibility behavior is
fabricated by their namespace constants.

The existing sparse side table is a storage starting point, not a measured
cost improvement. Empty tables are scratch state, not an analyzed result.
The missing producer is one deliberate, function-local
`#[expect(clippy::todo, reason = "...")]`; the workspace denial remains in
force. No compiler, checker, linter, formatter or LSP enters this producer.
The current `lower()` and its eager program construction remain unchanged.
The skeleton therefore adds no production pipeline stage, serialization,
normal legacy dependency, extra analysis or observer work.

The current L3 alias still resolves to its pre-rename physical package.
When the held physical-L3 move is replayed on main, the script must move
all then-current tracked files and preserve these new module registrations;
transporting a stale whole `lib.rs` would discard this change.

## Remaining work and acceptance

- Implement the producer with the same L2 page-order identity law used by
  the owning artifact. Define complete branch/scope and grouping/ordering
  facts where the current containment skeleton is insufficient.
- Implement each target's criteria in L3, replacing scattered DOM decision
  work instead of running a second analysis in production.
- Compare the old DOM analysis and the native tables in tests only. Keep
  legacy output byte-identical, including cache/hoist numbering in L4.
- Split the flat program onto the Vapor demand path. DOM and SSR must not
  construct it only to retrieve decisions.
- Verify typed compilation, full feature recipes, differential output,
  allocations and the unchanged instruction-count gate in Actions before
  selecting a product path. This skeleton has no native coverage credit.
- Extract L4 writing and module ownership separately under #6840. Do not
  place emission or identifier reparsing inside this decision module.

Local verification is formatting, source bounds and source-preservation
inspection only. Typed compilation, runtime checks and numeric acceptance
remain unverified until Actions executes the concrete PR source.
