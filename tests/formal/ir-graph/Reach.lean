import Std

namespace Davinci

variable {Op : Type} (edge : Op → Op → Prop) (root : Op)

/-- Reachability includes the empty path. The relation already incorporates
    a query's direction and its selected edge kinds. -/
inductive Path (edge : Op → Op → Prop) (root : Op) : Op → Prop where
  | seed : Path edge root root
  | extend {a b} : Path edge root a → edge a b → Path edge root b

/-- The checker consumes rows in discovery order. The latest row is the head
    here, so membership in the tail represents an earlier witness. -/
inductive Supported (edge : Op → Op → Prop) (root : Op) : List Op → Prop where
  | seed : Supported edge root [root]
  | push {rows parent child} :
      Supported edge root rows → parent ∈ rows → edge parent child →
      child ∉ rows → Supported edge root (child :: rows)

theorem supported_sound {rows : List Op} (h : Supported edge root rows) :
    ∀ x ∈ rows, Path edge root x := by
  induction h with
  | seed =>
    intro x hx
    cases hx with
    | head => exact Path.seed
    | tail _ h => cases h
  | @push rows parent child _ mem e _ ih =>
    intro x hx
    cases hx with
    | head => exact Path.extend (ih parent mem) e
    | tail _ old => exact ih x old

theorem contains_seed {rows : List Op} (h : Supported edge root rows) : root ∈ rows := by
  induction h with
  | seed => exact List.Mem.head _
  | push _ _ _ _ ih => exact List.Mem.tail _ ih

theorem exact_answer {rows : List Op} (h : Supported edge root rows)
    (closed : ∀ {a b}, a ∈ rows → edge a b → b ∈ rows) (x : Op) :
    x ∈ rows ↔ Path edge root x := by
  constructor
  · exact supported_sound edge root h x
  · intro path
    induction path with
    | seed => exact contains_seed edge root h
    | extend _ e ih => exact closed ih e

#print axioms exact_answer

/-- A single root is a complete answer exactly when every selected edge
    leaving it ends at the root. The relation includes direction and kind. -/
theorem singleton_exact :
    (∀ x, Path edge root x ↔ x = root) ↔ (∀ x, edge root x → x = root) := by
  constructor
  · intro exact x step
    exact (exact x).mp (Path.extend Path.seed step)
  · intro closed x
    constructor
    · intro path
      induction path with
      | seed => rfl
      | extend _ step ih => exact closed _ (ih ▸ step)
    · intro eq
      exact eq ▸ Path.seed

#print axioms singleton_exact

/-- The input contains the original relationship ordinals in order. A stable
    adjacency list selects those ordinals at one endpoint. -/
def incidentRows {E O : Type} [DecidableEq O] (edges : List E)
    (origin : E → O) (atOp : O) : List E :=
  edges.filter fun e => decide (origin e = atOp)

/-- One indexed pass per leg. The two patterns may use different orientations
    and kind filters. A relationship can occur at most once in a trail. -/
def indexTrails {E O : Type} [DecidableEq E] (out₁ out₂ : O → List E)
    (seed : O) (target₁ : E → O) (kind₁ kind₂ : E → Bool) : List (E × E) :=
  ((out₁ seed).filter kind₁).flatMap fun e =>
    ((out₂ (target₁ e)).filter fun f => kind₂ f && decide (e ≠ f)).map fun f => (e, f)

private theorem push_selection {E : Type} (outer inner : List E)
    (left : E → Bool) (right : E → E → Bool) :
    (outer.filter left).flatMap (fun e => (inner.filter (right e)).map (Prod.mk e)) =
    outer.flatMap (fun e => (inner.filter (fun f => left e && right e f)).map (Prod.mk e)) := by
  induction outer with
  | nil => rfl
  | cons e es ih => cases selected : left e <;> simp [selected, ih]

/-- A direct relational product in original ordinal order. Equality of these
    lists includes multiplicity and every result prefix used for early exit. -/
theorem exact_trails {E O : Type} [DecidableEq E] [DecidableEq O]
    (edges : List E) (out₁ out₂ : O → List E) (seed : O)
    (origin₁ target₁ origin₂ : E → O) (kind₁ kind₂ : E → Bool)
    (left : ∀ o, out₁ o = incidentRows edges origin₁ o)
    (right : ∀ o, out₂ o = incidentRows edges origin₂ o) :
    indexTrails out₁ out₂ seed target₁ kind₁ kind₂ =
      (edges.flatMap fun e => edges.map fun f => (e, f)).filter fun (e, f) =>
        (kind₁ e && decide (origin₁ e = seed)) &&
        ((kind₂ f && decide (e ≠ f)) && decide (origin₂ f = target₁ e)) := by
  unfold indexTrails
  simp only [left, right, incidentRows, List.filter_filter, List.filter_flatMap, List.filter_map]
  exact push_selection edges edges _ _

#print axioms exact_trails

/-- The original bytes of an operation survive projection from a retained
    block into the document for any prefix and suffix, including empty spans. -/
theorem authored_slice {Byte : Type} (before block after : List Byte) (lo hi : Nat)
    (ordered : lo ≤ hi) (inside : hi ≤ block.length) :
    ((before ++ (block ++ after)).drop (before.length + lo)).take (hi - lo) =
      (block.drop lo).take (hi - lo) := by
  rw [List.drop_length_add_append, List.drop_append_of_le_length (Nat.le_trans ordered inside)]
  exact List.take_append_of_le_length (by simpa using Nat.sub_le_sub_right inside lo)

#print axioms authored_slice

/-- Retained diagnostics, related labels and witness links may repeat or overlap.
    After projection their payloads, order, file ranges and exact spelling agree
    with directly reading every occurrence from the current document. -/
theorem recovery_sites_exact {Byte Label : Type} (before block after : List Byte)
    (sites : List (Label × Nat × Nat))
    (valid : ∀ site ∈ sites, site.2.1 ≤ site.2.2 ∧ site.2.2 ≤ block.length) :
    sites.map (fun (label, lo, hi) =>
      (label, before.length + lo, before.length + hi,
       ((before ++ (block ++ after)).drop (before.length + lo)).take (hi - lo))) =
    sites.map (fun (label, lo, hi) =>
      (label, before.length + lo, before.length + hi, (block.drop lo).take (hi - lo))) := by
  apply List.map_congr_left
  intro site member
  obtain ⟨label, lo, hi⟩ := site
  obtain ⟨ordered, inside⟩ := valid _ member
  exact congrArg (fun bytes => (label, before.length + lo, before.length + hi, bytes))
    (authored_slice before block after lo hi ordered inside)

#print axioms recovery_sites_exact

/-- A row identity looks up its two endpoints in the original selected relation.
    A failed lookup covers a missing ordinal or a kind excluded by the query. -/
inductive ConnectedTrail {E V : Type} (original : E → Option (V × V)) :
    V → List E → V → Prop where
  | stop (v) : ConnectedTrail original v [] v
  | extend {a b z edge rest} : original edge = some (a, b) → edge ∉ rest →
      ConnectedTrail original b rest z → ConnectedTrail original a (edge :: rest) z

def replay {E V : Type} [DecidableEq V] (original : E → Option (V × V)) (at_ : V) :
    List E → Option V
  | [] => some at_
  | edge :: rest => do
      let (src, dst) ← original edge
      if src = at_ then replay original dst rest else none

/-- Checking physical relationship identity independently of endpoint replay is
    equivalent to the edge-distinct trail derivation, at every finite length. -/
theorem bounded_witness_exact {E V : Type} [DecidableEq V]
    (original : E → Option (V × V)) (lo hi : Nat) (rows : List E) (start finish : V) :
    (lo ≤ rows.length ∧ rows.length ≤ hi ∧ rows.Nodup ∧
      replay original start rows = some finish) ↔
    (lo ≤ rows.length ∧ rows.length ≤ hi ∧ ConnectedTrail original start rows finish) := by
  have path : ∀ (xs : List E) (a z : V),
      (xs.Nodup ∧ replay original a xs = some z) ↔ ConnectedTrail original a xs z := by
    intro xs
    induction xs with
    | nil =>
      intro a z
      constructor
      · rintro ⟨_, h⟩
        have h' : a = z := Option.some.inj h
        subst z
        exact .stop a
      · intro h
        cases h
        exact ⟨by simp, rfl⟩
    | cons e rest ih =>
      intro a z
      constructor
      · rintro ⟨unique, ran⟩
        obtain ⟨unused, tail⟩ := List.nodup_cons.mp unique
        cases cell : original e with
        | none => simp [replay, cell] at ran
        | some pair =>
          obtain ⟨from_, to_⟩ := pair
          by_cases starts : from_ = a
          · subst from_
            exact .extend cell unused ((ih to_ z).mp ⟨tail, by simpa [replay, cell] using ran⟩)
          · simp [replay, cell, starts] at ran
      · intro h
        cases h with
        | extend cell unused tail =>
          obtain ⟨unique, ran⟩ := (ih _ _).mpr tail
          exact ⟨List.nodup_cons.mpr ⟨unused, unique⟩, by simpa [replay, cell] using ran⟩
  exact and_congr_right (fun _ => and_congr_right (fun _ => path rows start finish))

/-- info: 'Davinci.bounded_witness_exact' depends on axioms: [propext] -/
#guard_msgs in
#print axioms bounded_witness_exact

end Davinci
