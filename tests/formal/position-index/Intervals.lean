import Std

namespace Davinci.PositionIndex

structure Site (α : Type) where
  start : Nat
  stop : Nat
  row : α

/-- In-order traversal exposes the physical start-sorted rows after the Rust
    constructor discards empty spans. The theorem permits arbitrary tree shapes. -/
inductive Tree (α : Type) where
  | empty
  | node (left : Tree α) (site : Site α) (right : Tree α)

def rows : Tree α → List (Site α)
  | .empty => []
  | .node left site right => rows left ++ site :: rows right

def Ordered : Tree α → Prop
  | .empty => True
  | .node left site right =>
      Ordered left ∧ Ordered right ∧
      (∀ x ∈ rows left, x.start ≤ site.start) ∧
      (∀ x ∈ rows right, site.start ≤ x.start)

inductive Index (α : Type) where
  | empty
  | node (left : Index α) (site : Site α) (right : Index α) (maxEnd : Nat)

def high : Index α → Nat
  | .empty => 0
  | .node _ _ _ maxEnd => maxEnd

def mark : Tree α → Index α
  | .empty => .empty
  | .node left site right =>
      let l := mark left
      let r := mark right
      .node l site r (max (max site.stop (high l)) (high r))

def covers (offset : Nat) (site : Site α) : Bool :=
  decide (site.start ≤ offset ∧ offset < site.stop)

def point (offset : Nat) : Index α → List (Site α)
  | .empty => []
  | .node left site right maxEnd =>
      if maxEnd ≤ offset then [] else
        point offset left ++
          (if site.start ≤ offset then
             (if offset < site.stop then [site] else []) ++ point offset right
           else [])

private theorem stop_le_high (t : Tree α) (x : Site α) (hx : x ∈ rows t) :
    x.stop ≤ high (mark t) := by
  induction t with
  | empty => simp [rows] at hx
  | node l s r il ir =>
    simp only [rows, List.mem_append, List.mem_cons] at hx
    simp only [mark, high]
    rcases hx with hl | he | hr
    · exact Nat.le_trans (il hl) (Nat.le_trans (Nat.le_max_right _ _) (Nat.le_max_left _ _))
    · subst x
      exact Nat.le_trans (Nat.le_max_left _ _) (Nat.le_max_left _ _)
    · exact Nat.le_trans (ir hr) (Nat.le_max_right _ _)

private theorem ended (t : Tree α) (offset : Nat) (h : high (mark t) ≤ offset) :
    (rows t).filter (covers offset) = [] := by
  apply List.filter_eq_nil_iff.mpr
  intro x hx selected
  have upper := Nat.le_trans (stop_le_high t x hx) h
  have lower := (of_decide_eq_true selected).2
  omega

/-- Equality of lists preserves each physical occurrence, including duplicate
    spans, the first rejected callback, and the order of all earlier callbacks. -/
theorem point_eq_scan (t : Tree α) (offset : Nat) (ordered : Ordered t) :
    point offset (mark t) = (rows t).filter (covers offset) := by
  induction t with
  | empty => rfl
  | node l s r il ir =>
    obtain ⟨ol, oright, _, after⟩ := ordered
    by_cases doneHere : max (max s.stop (high (mark l))) (high (mark r)) ≤ offset
    · have h := ended (.node l s r) offset doneHere
      simp only [mark, point, if_pos doneHere]
      exact h.symm
    · simp only [mark, point, if_neg doneHere]
      by_cases started : s.start ≤ offset
      · by_cases beforeEnd : offset < s.stop <;>
          simp [started, beforeEnd, rows, covers, il ol, ir oright]
      · have untouched : (rows r).filter (covers offset) = [] := by
          apply List.filter_eq_nil_iff.mpr
          intro x hx selected
          exact started (Nat.le_trans (after x hx) (of_decide_eq_true selected).1)
        simp only [if_neg started, List.append_nil,
          rows, List.filter_append, List.filter_cons, untouched, il ol]
        simp [covers, started]

#print axioms point_eq_scan

private theorem discard_empty (source : List (Site α)) (offset : Nat) :
    (source.filter (fun x => decide (x.start < x.stop))).filter (covers offset) =
      source.filter (covers offset) := by
  rw [List.filter_filter]
  apply List.filter_congr
  intro x _
  by_cases h : x.start < x.stop
  · simp [h]
  · have impossible : ¬(x.start ≤ offset ∧ offset < x.stop) := by omega
    simp [h, covers, impossible]

/-- Sorting may reorder tied starts. Any permutation of the nonempty rows
    produces precisely the same selected row occurrences as the source scan. -/
theorem selected_rows (source : List (Site α)) (t : Tree α) (offset : Nat)
    (ordered : Ordered t)
    (sorted : (rows t).Perm (source.filter fun x => decide (x.start < x.stop)))
    (select : Site α → Option β) :
    ((point offset (mark t)).filterMap select).Perm
      ((source.filter (covers offset)).filterMap select) := by
  rw [point_eq_scan t offset ordered]
  have sameRows := sorted.filter (covers offset)
  rw [discard_empty] at sameRows
  exact sameRows.filterMap select

#print axioms selected_rows

end Davinci.PositionIndex
