import Std

namespace Davinci

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

end Davinci
