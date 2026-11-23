import Std

namespace Davinci.TemplateProjection

/-- The payload distinguishes ordered decisions, labels and links with their proof keys. -/
structure Row (α : Type) where
  payload : α
  start : Nat
  stop : Nat

def valid (length : Nat) (boundary : Nat → Bool) (r : Row α) : Bool :=
  decide (r.start ≤ r.stop ∧ r.stop ≤ length) && boundary r.start && boundary r.stop

def shift (base : Nat) (r : Row α) : Row α :=
  { r with start := base + r.start, stop := base + r.stop }

/-- Fail on the first range rejected by the current block's UTF-8 boundary predicate. -/
def project (base length : Nat) (boundary : Nat → Bool) : List (Row α) → Option (List (Row α))
  | [] => some []
  | r :: rest => if valid length boundary r then
      (project base length boundary rest).map (shift base r :: ·)
    else none

theorem exact (base length : Nat) (boundary : Nat → Bool) (rows : List (Row α)) :
    project base length boundary rows =
      if rows.all (valid length boundary) then some (rows.map (shift base)) else none := by
  induction rows with
  | nil => rfl
  | cons r rest ih =>
    cases first : valid length boundary r <;> cases tail : rest.all (valid length boundary) <;>
      simp [project, first, tail, ih]

/-- An addressable original root bounds both machine additions in the Rust projection. -/
theorem endpoints_fit (base length limit : Nat) (r : Row α)
    (ordered : r.start ≤ r.stop) (inside : r.stop ≤ length) (root : base + length ≤ limit) :
    (shift base r).start ≤ limit ∧ (shift base r).stop ≤ limit :=
  ⟨Nat.le_trans (Nat.add_le_add_left (Nat.le_trans ordered inside) base) root,
   Nat.le_trans (Nat.add_le_add_left inside base) root⟩

#print axioms exact
#print axioms endpoints_fit

end Davinci.TemplateProjection
