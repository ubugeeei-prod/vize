# Native decision ownership (2026-10-01)

Tracked in [#6839](https://github.com/ubugeeei-prod/vize/issues/6839).
The maintainer requested concrete files and explicit `todo!()` producers
before feature implementation, with ownership rearchitecture taking priority.
That skeleton now has a [bounded native producer](../decisions/2026-10-01-l3-native-artifact-decisions.md)
over the sealed canonical L2 artifact; it does not finish #6839.

| Responsibility                                           | Native owner                                   | State                                               |
| -------------------------------------------------------- | ---------------------------------------------- | --------------------------------------------------- |
| Semantic tree and binding scopes                         | `vize_l2::artifact::Artifact`                  | Sealed native owner; product integration unfinished |
| Static, dynamic-binding, placement and control decisions | `vize_l3::decision`                            | Conservative facts; placement always Inline         |
| Per-target eligibility criteria                          | `vize_l3::decision::policy::{dom, ssr, vapor}` | Cloak/event filtering only; full criteria unfinished |
| Borrowed L2 to shared L3 decisions                       | `vize_l2_to_l3::build_decisions`               | Single canonical event walk; no production caller   |
| Flat reactive program                                    | `vize_l3::op::Program`                         | Existing artifact; demand-only split unfinished     |
| Decision encoding, helper numbering and emission         | Future `vize_l4`                               | Separate #6840 ownership work; not implemented here |

`DecisionTables` uses the existing `NodeId` and `SideTable` infrastructure.
Keys belong to the L2 artifact, including attached binding ids; they are
not flat-program `OpId`s. Dynamic bindings retain authored order, and
control records identify containment. The existing `Placement` vocabulary
is reused instead of creating a second hoist/cache/group enum. Target policy
identities select the admitted output filter. Neutral static meaning is
separate from selected output meaning; every placement stays `Inline`.

The existing sparse side table is a storage starting point, not a measured
cost improvement. Empty tables are scratch state, not an analyzed result.
The producer returns an explicit error if its shared walk or row accounting
fails; incomplete scratch tables are not completed analysis. No compiler,
checker, linter, formatter or LSP enters this producer.
The current `lower()` and its eager program construction remain unchanged.
The producer therefore adds no production pipeline stage, serialization,
normal legacy dependency, extra analysis or observer work.

## Remaining work and acceptance

- Define complete branch/scope and grouping/ordering facts beyond the
  current conditional, loop and slot-outlet containment.
- Finish each target's criteria in L3, replacing scattered DOM decision
  work instead of running a second analysis in production.
- Compare the old DOM analysis and the native tables in tests only. Keep
  legacy output byte-identical, including cache/hoist numbering in L4.
- Split the flat program onto the Vapor demand path. DOM and SSR must not
  construct it only to retrieve decisions.
- Verify typed compilation, full feature recipes, differential output,
  allocations and the unchanged instruction-count gate in Actions before
  selecting a product path. This bounded producer has no product coverage credit.
- Extract L4 writing and module ownership separately under #6840. Do not
  place emission or identifier reparsing inside this decision module.

Local verification includes formatting, source bounds, storage policy,
two standalone Rust policy laws and all five native producer laws using
actual selected L2/L3 sources with real cached L0/OXC. Full current-workspace
integration, runtime checks and numeric acceptance remain unverified until
Actions executes the concrete provider/consumer source.
