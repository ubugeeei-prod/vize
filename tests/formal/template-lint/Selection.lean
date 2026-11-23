import Std

namespace Davinci.TemplateLint

variable {α : Type} (before : α → α → Bool)

/-- Insert after ties, as a stable decreasing-priority sort does. -/
def insertFull (row : α) : List α → List α
  | [] => [row]
  | old :: rest =>
    if before row old then row :: old :: rest else old :: insertFull row rest

/-- Visit at most the retained slots. Inserting into a full buffer displaces its last row. -/
def insertBounded : Nat → α → List α → List α
  | 0, _, _ => []
  | _ + 1, row, [] => [row]
  | k + 1, row, old :: rest =>
    if before row old then row :: (old :: rest).take k
    else old :: insertBounded k row rest

theorem insert_prefix (k : Nat) (row : α) (rows : List α) :
    insertBounded before k row (rows.take k) = (insertFull before row rows).take k := by
  induction k generalizing rows with
  | zero => simp [insertBounded]
  | succ k ih =>
    cases rows with
    | nil => simp [insertBounded, insertFull]
    | cons old rest =>
      cases h : before row old with
      | false => simp [insertBounded, insertFull, h, ih]
      | true =>
        have keep : ((old :: rest).take (k + 1)).take k = (old :: rest).take k := by
          rw [List.take_take, Nat.min_eq_left (Nat.le_succ k)]
        simpa [insertBounded, insertFull, h] using keep

/-- The invariant holds for every prefix, any capacity (including zero),
    any comparison, and any initial state of the reference ranking. -/
theorem fold_prefix (k : Nat) (rows ranked : List α) :
    rows.foldl (fun acc row => insertBounded before k row acc) (ranked.take k) =
      (rows.foldl (fun acc row => insertFull before row acc) ranked).take k := by
  induction rows generalizing ranked with
  | nil => rfl
  | cons row rows ih =>
    simp only [List.foldl_cons, insert_prefix]
    exact ih (insertFull before row ranked)

-- Stable insertion preserves the multiset and orders all rows. A total order
-- suffices for equality with the library merge sort on distinguishable keys.
private theorem insertion_perm (row : α) (xs : List α) :
    (insertFull before row xs).Perm (row :: xs) := by
  induction xs with
  | nil => exact List.Perm.refl _
  | cons a xs ih =>
    cases h : before row a with
    | true => simp [insertFull, h]
    | false =>
      simp only [insertFull, h, Bool.false_eq_true, if_false]
      exact (ih.cons a).trans (List.Perm.swap row a xs)

private theorem insertion_ordered (le : α → α → Bool)
    (trans : ∀ a b c, le a b → le b c → le a c)
    (total : ∀ a b, le a b || le b a)
    (row : α) {xs : List α} (sorted : xs.Pairwise (fun a b => le a b = true)) :
    (insertFull (fun a b => !le b a) row xs).Pairwise (fun a b => le a b = true) := by
  induction xs with
  | nil => simp [insertFull]
  | cons a xs ih =>
    obtain ⟨head, tail⟩ := List.pairwise_cons.mp sorted
    cases h : le a row with
    | true =>
      simp only [insertFull, h, Bool.not_true, Bool.false_eq_true, if_false, List.pairwise_cons]
      refine ⟨?_, ih tail⟩
      intro b hb
      have := (insertion_perm (fun x y => !le y x) row xs).mem_iff.mp hb
      rcases List.mem_cons.mp this with rfl | hb
      · exact h
      · exact head b hb
    | false =>
      have first : le row a = true := by simpa [h] using total row a
      simp only [insertFull, h, Bool.not_false, if_true, List.pairwise_cons]
      refine ⟨?_, head, tail⟩
      intro b hb
      rcases List.mem_cons.mp hb with rfl | hb
      · exact first
      · exact trans row a b first (head b hb)

private theorem fold_perm (rows acc : List α) :
    (rows.foldl (fun best row => insertFull before row best) acc).Perm (acc ++ rows) := by
  induction rows generalizing acc with
  | nil => simp
  | cons a xs ih =>
    simp only [List.foldl_cons]
    exact (ih _).trans (((insertion_perm before a acc).append_right xs).trans List.perm_middle.symm)

private theorem fold_ordered (le : α → α → Bool)
    (trans : ∀ a b c, le a b → le b c → le a c)
    (total : ∀ a b, le a b || le b a)
    (rows acc : List α) (sorted : acc.Pairwise (fun a b => le a b = true)) :
    (rows.foldl (fun best row => insertFull (fun a b => !le b a) row best) acc).Pairwise (fun a b => le a b = true) := by
  induction rows generalizing acc with
  | nil => exact sorted
  | cons a xs ih => exact ih _ (insertion_ordered le trans total _ sorted)

private theorem full_mergeSort (le : α → α → Bool)
    (trans : ∀ a b c, le a b → le b c → le a c)
    (total : ∀ a b, le a b || le b a)
    (rows : List α)
    (antisymm : ∀ a ∈ rows, ∀ b ∈ rows, le a b → le b a → a = b) :
    rows.foldl (fun best row => insertFull (fun a b => !le b a) row best) [] =
      rows.mergeSort le := by
  have perm := fold_perm (fun a b => !le b a) rows []
  simp only [List.nil_append] at perm
  exact List.Perm.eq_of_pairwise
    (fun a b ma mb => antisymm a (perm.mem_iff.mp ma) b (List.mem_mergeSort.mp mb))
    (fold_ordered le trans total rows [] (by simp))
    (List.pairwise_mergeSort trans total rows)
    (perm.trans (List.mergeSort_perm rows le).symm)

-- The standard library decorates rows with their input index to express
-- stability. A newly seen row is later than every row in the window, so
-- dropping those indices leaves exactly the strict comparison in Rust.
private theorem erase_insertIdx (le : α → α → Bool) (row : α) (i : Nat)
    (ranked : List (α × Nat)) (earlier : ∀ x ∈ ranked, x.2 < i) :
    (insertFull (fun a b => !List.zipIdxLE le b a) (row, i) ranked).map Prod.fst =
      insertFull (fun a b => !le b a) row (ranked.map Prod.fst) := by
  induction ranked with
  | nil => rfl
  | cons old rest ih =>
    have age : old.2 ≤ i := Nat.le_of_lt (earlier old (by simp))
    have rank : List.zipIdxLE le old (row, i) = le old.1 row := by
      cases h : le old.1 row <;> simp [List.zipIdxLE, h, age]
    have tail : ∀ x ∈ rest, x.2 < i := fun x hx => earlier x (List.mem_cons_of_mem old hx)
    cases h : le old.1 row <;> simp [insertFull, rank, h, ih tail]

private theorem erase_foldIdx (le : α → α → Bool) (rows : List α)
    (start : Nat) (acc : List (α × Nat)) (earlier : ∀ x ∈ acc, x.2 < start) :
    ((rows.zipIdx start).foldl (fun best row => insertFull (fun a b => !List.zipIdxLE le b a) row best) acc).map Prod.fst =
      rows.foldl (fun best row => insertFull (fun a b => !le b a) row best) (acc.map Prod.fst) := by
  induction rows generalizing start acc with
  | nil => simp
  | cons a rows ih =>
    simp only [List.zipIdx_cons, List.foldl_cons]
    have ages : ∀ x ∈ insertFull (fun u v => !List.zipIdxLE le v u) (a,start) acc, x.2 < start + 1 := by
      intro x hx
      have h := (insertion_perm (fun u v => !List.zipIdxLE le v u) (a,start) acc).mem_iff.mp hx
      rcases List.mem_cons.mp h with rfl | h
      · simp
      · exact Nat.lt_trans (earlier x h) (Nat.lt_succ_self _)
    rw [ih _ _ ages, erase_insertIdx le a start acc earlier]

private theorem zipIdx_antisymm (le : α → α → Bool) (rows : List α) :
    ∀ a ∈ rows.zipIdx, ∀ b ∈ rows.zipIdx, List.zipIdxLE le a b → List.zipIdxLE le b a → a = b := by
  rintro ⟨a,i⟩ ma ⟨b,j⟩ mb hab hba
  cases ab : le a b <;> cases ba : le b a <;> simp [List.zipIdxLE, ab, ba] at hab hba
  have h := Nat.le_antisymm hab hba
  subst j
  have ha := List.mem_zipIdx_iff_getElem?.mp ma
  have hb := List.mem_zipIdx_iff_getElem?.mp mb
  simp_all

/-- On every finite input, stable bounded selection emits exactly the labels
    of the original rule: filter, stable priority sort, truncate, then stable
    source sort. Distinct rows may tie in both orders. -/
theorem labels_exact (k : Nat) (le : α → α → Bool)
    (trans : ∀ a b c, le a b → le b c → le a c)
    (total : ∀ a b, le a b || le b a)
    (positive : α → Bool) (sourceOrder : α → α → Bool) (rows : List α) :
    (((rows.filter positive).foldl (fun acc row => insertBounded (fun a b => !le b a) k row acc) []).mergeSort sourceOrder) =
      ((((rows.filter positive).mergeSort le).take k).mergeSort sourceOrder) := by
  let eligible := rows.filter positive
  have erased := erase_foldIdx le eligible 0 [] (by simp)
  rw [full_mergeSort (List.zipIdxLE le) (List.zipIdxLE_trans trans) (List.zipIdxLE_total total)
    eligible.zipIdx (zipIdx_antisymm le eligible), List.mergeSort_zipIdx] at erased
  have bounded := fold_prefix (fun a b => !le b a) k eligible []
  simp only [List.map_nil] at erased
  simp only [List.take_nil, ← erased] at bounded
  exact congrArg (fun xs : List α => xs.mergeSort sourceOrder) bounded

end Davinci.TemplateLint
#print axioms Davinci.TemplateLint.fold_prefix
#print axioms Davinci.TemplateLint.labels_exact
