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

/-- A monotone cursor list has one prefix below a given boundary. -/
private theorem cursor_prefix (limit : Nat) (points : List Nat)
    (ordered : points.Pairwise (· ≤ ·)) :
    points.takeWhile (fun p => decide (p < limit)) = points.filter (fun p => decide (p < limit)) := by
  induction points with
  | nil => rfl
  | cons x xs ih =>
    obtain ⟨least, tail⟩ := List.pairwise_cons.mp ordered
    by_cases inside : x < limit
    · simp [inside, ih tail]
    · have empty : xs.filter (fun p => decide (p < limit)) = [] := by
        apply List.filter_eq_nil_iff.mpr
        intro y mem
        have := least y mem
        simpa using (show ¬y < limit by omega)
      simp [inside, empty]

private theorem cursor_suffix (start : Nat) (points : List Nat)
    (ordered : points.Pairwise (· ≤ ·)) :
    points.dropWhile (fun p => decide (p < start)) = points.filter (fun p => decide (start ≤ p)) := by
  induction points with
  | nil => rfl
  | cons x xs ih =>
    obtain ⟨least, tail⟩ := List.pairwise_cons.mp ordered
    by_cases below : x < start
    · simp [below, Nat.not_le_of_lt below, ih tail]
    · have fromStart := Nat.le_of_not_lt below
      have all : xs.filter (fun p => decide (start ≤ p)) = xs := by
        apply List.filter_eq_self.mpr
        intro y mem
        exact decide_eq_true (Nat.le_trans fromStart (least y mem))
      simp [below, fromStart, all]

/-- Select authored half-open ranges, and retain every request at the insertion
    point of a zero-width diagnostic. List equality includes duplicate cursors. -/
theorem recovery_cursor_window (points : List Nat) (lo hi : Nat)
    (ordered : points.Pairwise (· ≤ ·)) :
    ((points.dropWhile fun p => decide (p < lo)).takeWhile fun p =>
      if lo = hi then decide (p ≤ hi) else decide (p < hi)) =
    points.filter (fun p => decide ((lo ≤ p ∧ p < hi) ∨ (lo = hi ∧ p = lo))) := by
  rw [cursor_suffix lo _ ordered]
  have sorted := ordered.filter (fun p => decide (lo ≤ p))
  by_cases anchor : lo = hi
  · subst hi
    simp only [if_true, true_and, eq_self]
    have upper : (fun p => decide (p ≤ lo)) = (fun p => decide (p < lo + 1)) := by
      funext p
      simp only [decide_eq_decide]
      omega
    rw [upper, cursor_prefix _ _ sorted, List.filter_filter]
    apply List.filter_congr
    intro p _
    by_cases eq : p = lo
    · simp [eq]
    · by_cases lt : p < lo
      · simp [lt, Nat.not_le_of_lt lt, eq]
      · have after : ¬p < lo + 1 := by omega
        simp [after, lt, eq]
  · simp only [if_neg anchor]
    rw [cursor_prefix _ _ sorted, List.filter_filter]
    apply List.filter_congr
    intro p _
    simp [anchor, Bool.and_comm]

/-- info: 'Davinci.recovery_cursor_window' depends on axioms: [propext, Classical.choice, Quot.sound] -/
#guard_msgs in
#print axioms recovery_cursor_window

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


namespace BoundedTrail

def allOrdinalLists (ordinals : List E) : Nat → List (List E)
  | 0 => [[]]
  | n+1 => [] :: ordinals.flatMap fun e => (allOrdinalLists ordinals n).map (e :: ·)

/-- Test a complete candidate list against the original directed relationships.
    The prefix may already own some of the physical ordinals. -/
def admittedTrail [DecidableEq E] (next : V → E → Option V)
    (seen : List E) (v : V) : List E → Bool
  | [] => true
  | e :: tail => match next v e with
    | none => false
    | some w => decide (e ∉ seen) && admittedTrail next (e :: seen) w tail

/-- Preorder of every prefix, before applying the requested minimum length. -/
def indexedPrefixes [DecidableEq E] (incident : V → List E) (next : V → E → Option V) :
    Nat → List E → V → List (List E)
  | 0, _, _ => [[]]
  | n+1, seen, v => [] :: (incident v).flatMap fun e =>
      match next v e with
      | none => []
      | some w =>
        if e ∈ seen then [] else (indexedPrefixes incident next n (e :: seen) w).map (e :: ·)

private theorem skip_empty_rows (rows : List E) (keep : E → Bool) (f : E → List A)
    (empty : ∀ e, keep e = false → f e = []) :
    (rows.filter keep).flatMap f = rows.flatMap f := by
  induction rows with
  | nil => rfl
  | cons e rest ih => cases h : keep e <;> simp [h, empty, ih]

/-- For any finite fuel, the ordered index emits exactly the retained, admissible
    original lists. This equality includes both multiplicity and prefix order. -/
private theorem prefix_rows_exact [DecidableEq E] (ordinals : List E) (incident : V → List E)
    (next : V → E → Option V)
    (ordered : ∀ v, incident v = ordinals.filter fun e => (next v e).isSome)
    (fuel : Nat) (seen : List E) (v : V) :
    indexedPrefixes incident next fuel seen v =
      (allOrdinalLists ordinals fuel).filter (admittedTrail next seen v) := by
  induction fuel generalizing seen v with
  | zero => rfl
  | succ n ih =>
    simp only [indexedPrefixes, allOrdinalLists, List.filter_cons, admittedTrail, ite_true,
      List.filter_flatMap, ordered v]
    congr 1
    rw [skip_empty_rows]
    · apply congrArg (fun step : E → List (List E) => ordinals.flatMap step)
      funext e
      rw [List.filter_map]
      cases step : next v e with
      | none => simp [Function.comp_def, admittedTrail, step]
      | some w =>
        by_cases used : e ∈ seen <;> simp [Function.comp_def, admittedTrail, step, used, ih]
    · intro e absent
      cases found : next v e <;> simp_all

def nextOn [DecidableEq V] (original : E → Option (V × V)) (v : V) (e : E) : Option V :=
  match original e with
  | none => none
  | some (src, dst) => if src = v then some dst else none

private theorem admitted_exact [DecidableEq E] [DecidableEq V]
    (original : E → Option (V × V)) (rows seen : List E) (v : V) :
    admittedTrail (nextOn original) seen v rows = true ↔
      (∀ e ∈ rows, e ∉ seen) ∧ rows.Nodup ∧ (replay original v rows).isSome = true := by
  induction rows generalizing v seen with
  | nil => simp [admittedTrail, replay]
  | cons e rest ih =>
    cases cell : original e with
    | none => simp [admittedTrail, nextOn, cell, replay]
    | some pair =>
      obtain ⟨src, dst⟩ := pair
      by_cases start : src = v
      · subst src
        simp [admittedTrail, nextOn, cell, replay, ih]
        constructor
        · rintro ⟨fresh, avoid, distinct, ran⟩
          exact ⟨⟨fresh, fun x hx => (avoid x hx).2⟩,
            ⟨fun used => (avoid e used).1 rfl, distinct⟩, ran⟩
        · rintro ⟨⟨fresh, avoid⟩, ⟨unused, distinct⟩, ran⟩
          exact ⟨fresh, fun x hx => ⟨fun same => unused (same ▸ hx), avoid x hx⟩, distinct, ran⟩
      · simp [admittedTrail, nextOn, cell, start, replay]

/-- Every reported prefix agrees, in ordinal order, with filtering all words
    through the original relation and the independent witness conditions. -/
theorem enumeration_exact [DecidableEq E] [DecidableEq V]
    (ordinals : List E) (original : E → Option (V × V)) (incident : V → List E)
    (ordered : ∀ v, incident v = ordinals.filter fun e => (nextOn original v e).isSome)
    (lo hi : Nat) (start : V) :
    ((indexedPrefixes incident (nextOn original) hi [] start).filter fun xs =>
        decide (lo ≤ xs.length)) =
      (allOrdinalLists ordinals hi).filter fun xs =>
        decide (lo ≤ xs.length ∧ xs.Nodup) && (replay original start xs).isSome := by
  rw [prefix_rows_exact ordinals incident _ ordered, List.filter_filter]
  apply congrArg (fun test : List E → Bool => (allOrdinalLists ordinals hi).filter test)
  funext xs
  apply Bool.eq_iff_iff.mpr
  simp [admitted_exact, and_comm, and_left_comm, and_assoc]

/-- info: 'Davinci.BoundedTrail.enumeration_exact' depends on axioms: [propext, Classical.choice, Quot.sound] -/
#guard_msgs in
#print axioms enumeration_exact

end BoundedTrail

end Davinci
