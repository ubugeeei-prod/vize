import Impeto.Schedule

/-!
C-23: independent S3 Folio checker (Lean4Lean discipline). It re-checks the
graph invariants that the TS-27 phase validator enforces from the committed
Folio text alone and shares no code with the compiler. Each invariant is a
decidable proposition. `WellFormed` is their conjunction, and
`CheckerLaws.violations_nil_iff` proves the executable checker exact: it reports
no violation exactly when the program is well formed. Every definition here is
structurally recursive or fuel-bounded, hence total.
-/

namespace Impeto.Checker
open Schedule

def contains (outer inner : Span) : Bool := outer.start ≤ inner.start && inner.stop ≤ outer.stop

def ids (program : Program) : List Nat := program.ops.map (·.id)

/-- The parent chain from `region` reaches a parentless region within `fuel` steps. -/
def reachesTop (program : Program) : Nat -> Nat -> Bool
  | 0, _ => false
  | fuel + 1, region =>
      match program.regions.find? (·.id == region) with
      | some { parent := some parent, .. } => reachesTop program fuel parent
      | some { parent := none, .. } => true
      | none => false

def forward (program : Program) (edge : StateEdge) : Bool :=
  match indexOf? (ids program) edge.source, indexOf? (ids program) edge.target with
  | some i, some j => i < j
  | _, _ => false

/-- S3V001: op, region and effect ids are unique. -/
def UniqueIds (p : Program) : Prop :=
  (ids p).Nodup ∧ (p.regions.map (·.id)).Nodup ∧ (p.effects.map (·.id)).Nodup

/-- S3V002: region 0 exists and is the parentless, ownerless root. -/
def Rooted (p : Program) : Prop :=
  ∃ r ∈ p.regions, r.id = 0 ∧ r.parent = none ∧ r.owner = none

/-- S3V005: every other region names a resolvable parent and an owner op in it. -/
def Resolved (p : Program) : Prop :=
  ∀ r ∈ p.regions, r.id ≠ 0 ->
    (∃ q ∈ p.regions, r.parent = some q.id) ∧
    (∃ o ∈ p.ops, r.owner = some o.id ∧ r.parent = some o.region)

/-- S3V006: spans nest (region ⊆ parent, region ⊆ owner, op ⊆ region) and every
parent chain is finite. -/
def Nested (p : Program) : Prop :=
  (∀ r ∈ p.regions, ∀ q ∈ p.regions, r.parent = some q.id -> contains q.span r.span = true) ∧
  (∀ r ∈ p.regions, ∀ o ∈ p.ops, r.owner = some o.id -> contains o.span r.span = true) ∧
  (∀ o ∈ p.ops, ∀ r ∈ p.regions, r.id = o.region -> contains r.span o.span = true) ∧
  (∀ r ∈ p.regions, reachesTop p (p.regions.length + 1) r.id = true)

/-- S3V003: ops live in existing regions. -/
def OpsResolved (p : Program) : Prop :=
  ∀ o ∈ p.ops, ∃ r ∈ p.regions, r.id = o.region

/-- S3V007: ops name existing effects; an effect's owner lives in its region (or
below) and its span nests in that region. -/
def Scoped (p : Program) : Prop :=
  (∀ o ∈ p.ops, ∀ e ∈ o.effect.toList, ∃ s ∈ p.effects, s.id = e) ∧
  ∀ s ∈ p.effects, (∃ o ∈ p.ops, o.id = s.owner ∧ isOrDescendant p o.region s.region = true) ∧
    (∃ r ∈ p.regions, r.id = s.region ∧ contains r.span s.span = true)

/-- S3V004 / S3V007: edges resolve and effect-scoped edges stay in scope. -/
def EdgesResolved (p : Program) : Prop :=
  ∀ e ∈ p.edges, (e.source ∈ ids p) ∧ (e.target ∈ ids p) ∧
    ∀ sc ∈ e.effect.toList, ∃ s ∈ p.effects, s.id = sc ∧
      opInside p e.source s.region = true ∧ opInside p e.target s.region = true

/-- S3V008: in the scheduled phase every edge points strictly forward. -/
def Ordered (p : Program) : Prop :=
  p.phase = .scheduled -> ∀ e ∈ p.edges, forward p e = true

def WellFormed (p : Program) : Prop :=
  UniqueIds p ∧ Rooted p ∧ Resolved p ∧ Nested p ∧ OpsResolved p ∧ Scoped p ∧
    EdgesResolved p ∧ Ordered p

instance (p : Program) : Decidable (UniqueIds p) := by unfold UniqueIds; infer_instance
instance (p : Program) : Decidable (Rooted p) := by unfold Rooted; infer_instance
instance (p : Program) : Decidable (Resolved p) := by unfold Resolved; infer_instance
instance (p : Program) : Decidable (Nested p) := by unfold Nested; infer_instance
instance (p : Program) : Decidable (OpsResolved p) := by unfold OpsResolved; infer_instance
instance (p : Program) : Decidable (Scoped p) := by unfold Scoped; infer_instance
instance (p : Program) : Decidable (EdgesResolved p) := by unfold EdgesResolved; infer_instance
instance (p : Program) : Decidable (Ordered p) := by unfold Ordered; infer_instance

def code (holds : Bool) (name : String) : List String := if holds then [] else [name]

/-- The executable checker: one code per violated invariant, in validator order. -/
def violations (p : Program) : List String :=
  code (decide (UniqueIds p)) "S3V001" ++ code (decide (Rooted p)) "S3V002" ++
    code (decide (Resolved p)) "S3V005" ++ code (decide (Nested p)) "S3V006" ++
    code (decide (OpsResolved p)) "S3V003" ++ code (decide (Scoped p)) "S3V007" ++
    code (decide (EdgesResolved p)) "S3V004" ++ code (decide (Ordered p)) "S3V008"

end Impeto.Checker
