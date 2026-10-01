# Native L3 decisions over the canonical L2 artifact (2026-10-01)

Tracked in [#6839](https://github.com/ubugeeei-prod/vize/issues/6839), with
[#6838](https://github.com/ubugeeei-prod/vize/issues/6838)'s sealed native
artifact as the real provider. This is a bounded producer, not completion
of #6839 or a product replacement.

`vize_l3::decision::build_decisions(&Artifact, TargetPolicy)` borrows the checked
L2 owner and returns [owner-bound `NativeAnalysis`](./2026-10-01-l3-owner-bound-analysis.md)
or an explicit walk/accounting error. The conversion edge re-exports this sole
producer. It consumes the owner's enter/leave events once. Every region op
and attached binding receives exactly one row under the supplied `NodeId`;
L3 never mints another numbering, reparses an expression, serializes the
tree or constructs a flat L3 program.

The static classification is conservative. Text is static; direct
interpolation makes its native element `DynamicText`, while interpolation
under a nested element makes the ancestor dynamic. Authored comments,
components and conditional/loop/slot-outlet constructs remain dynamic.
Every non-cloak attached binding has dynamic neutral meaning. These facts
do not change when the selected output target changes.

`NodeDecision::output_level` separately records selected output meaning.
DOM and Vapor retain every non-cloak binding; SSR omits event handlers on
native elements, while component events and slot-outlet props remain
dynamic. Omission propagates to ancestor output classification without
changing neutral meaning. Dynamic binding lists keep authored order.
Every other binding family remains conservatively dynamic; this is no
claim about Vapor effect generation or complete SSR target eligibility.

Conditional branches, loop bodies and slot-outlet fallback have their
actual nearest containing control owner. Slot props are inside the outlet
control. The control node itself retains its outer owner; root siblings
do not inherit a previous control. Authored slot-content and legacy
slot-scope carriers are accounted for as binding rows, but their canonical
grouping and scoped content regions remain unfinished.

All placement rows are `Inline`. No hoist/cache allocation, handler cache
eligibility, cache number, DOM patch flag, effect group or L4 encoding is
invented. The existing eager `lower()` path and every product stay as they
are. This producer adds no pipeline stage, normal/build legacy dependency,
release verifier or observer work. The compiler fix-history gate #6880 and
all differential/instruction/CI gates remain in force.

Native arena fixtures cover all 14 attached families, all eight region op
families, exact total/region/attached cardinality, authored binding order,
target-neutral stability, output propagation, direct/nested interpolation,
comments, cloak, nested if/for/slot containment and an empty artifact.
Two policy laws cover every target/owner combination. The implementation
uses one owned traversal stack and the existing owned binding-id lists;
the reviewed storage row adds one `alloc::vec::Vec` path and five bound
uses in L3's `decision/build.rs`. Sparse side tables are retained with no cost-reduction claim.

The existing instruction registry covers zero executions of this producer
in its 100 pinned rows. Its collector has no `vize_l2_to_l3` suite: the
Vapor lowering windows measure `transform_to_ir`, and selected fused
compiler routes still request the existing flat `lower()` program. The
L0 pass-name/fact prototypes and Croquis hoist-gate pair also never consume
canonical L2. Substituting a different workload into their pinned identities
would not demonstrate native adoption. A directly identified native L3
measurement over the existing six-fixture ladder remains follow-up work;
no existing input identity, measurement window or ceiling changes here.

Local evidence includes formatting, source bounds, storage-policy checks,
two standalone Rust policy laws and all five native producer laws. A
lightweight Rust 1.98 harness compiles byte-exact selected canonical L2,
L3 decision/placement and conversion modules against real cached L0/OXC;
it uses no stubs and does not rebuild dependencies. This proves these
native laws, not full integration with the current workspace's L0 surface.
Full crate tests, Clippy, legacy differential output and instruction gates
require Actions on the actual provider/consumer source. Publication and
product selection remain separate steps, and #6839 stays open.

The filled native producer removes `vize_l2_to_l3` from the skeleton ratchet
using the real tool's `--write` output. On the initial `cf47e857` head,
seven modules remained in two crates. Replay on the repaired provider at
`9c36db024` reported six remaining modules, all in `vize_l4`. After #7353
actually merged as `3185ba57`, replay on that main preserves the independently
filled runtime module and the real writer reports five, all in `vize_l4`;
the actual current writer output is authoritative.
The initial exact-head prefix failed inherited formatting/source inventory
and unchanged DOM instruction gates; those provider repairs are separate.
The provider's protected queue Check and all 100 unchanged instruction
ceilings passed before its actual merge. The rebased consumer still requires
fresh exact-head required/full Actions and unchanged instruction acceptance
before queue admission. Owner-bound analysis remains a separate child.
