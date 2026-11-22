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

end Davinci
