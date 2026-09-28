import Impeto.Schedule

/-!
P3-15 effect-grouping theorem. Acceptance by the scheduled-phase validator
contract implies that the P3-4 semantics executes every state edge's source
before its target under both backends, and every effect-scoped edge stays in
its scope. Any regrouping of ops that the validator accepts therefore
preserves the whole dependency edge set of the original program.
-/

namespace Impeto.Schedule

theorem getElem?_of_indexOf? : ∀ {ids : List Nat} {id i : Nat},
    indexOf? ids id = some i -> ids[i]? = some id
  | [], _, _, h => by simp [indexOf?] at h
  | x :: xs, id, i, h => by
      unfold indexOf? at h
      split at h
      · next hx => cases h; simp [hx]
      · cases hi : indexOf? xs id with
        | none => simp [hi] at h
        | some k =>
            simp only [hi, Option.map_some, Option.some.injEq] at h
            subst h
            simpa using getElem?_of_indexOf? hi

theorem indexOf?_of_getElem? : ∀ {ids : List Nat} {id k : Nat},
    ids.Nodup -> ids[k]? = some id -> indexOf? ids id = some k
  | [], _, _, _, h => by simp at h
  | x :: xs, id, 0, _, h => by
      simp only [List.getElem?_cons_zero, Option.some.injEq] at h
      simp [indexOf?, h]
  | x :: xs, id, k + 1, nd, h => by
      simp only [List.getElem?_cons_succ] at h
      have fresh : x ≠ id := fun same =>
        (List.nodup_cons.mp nd).1 (same ▸ List.mem_of_getElem? h)
      simp [indexOf?, fresh, indexOf?_of_getElem? (List.nodup_cons.mp nd).2 h]

theorem nodup_of_duplicateCodes : ∀ {seen ids : List Nat},
    duplicateCodes seen ids = [] -> ids.Nodup ∧ ∀ id, id ∈ ids -> id ∉ seen
  | _, [], _ => ⟨List.nodup_nil, fun _ mem => nomatch mem⟩
  | seen, id :: rest, h => by
      simp only [duplicateCodes, List.append_eq_nil_iff] at h
      obtain ⟨here, later⟩ := h
      have unseen : id ∉ seen := fun mem => by simp [mem] at here
      obtain ⟨nodup, fresh⟩ := nodup_of_duplicateCodes later
      refine ⟨List.nodup_cons.mpr ⟨fun mem => fresh id mem (.head _), nodup⟩, ?_⟩
      intro other mem
      cases mem with
      | head => exact unseen
      | tail _ mem => exact fun seenMem => fresh other mem (.tail _ seenMem)

theorem runOps_eq (backend : Backend) :
    ∀ (ops : List Op) (trace : List TraceEvent),
      runOps backend ops trace = trace ++ ops.map (interpret backend)
  | [], trace => by simp [runOps]
  | op :: rest, trace => by simp [runOps, runOps_eq backend rest]

theorem interpret_opId (backend : Backend) (op : Op) : (interpret backend op).opId = op.id := rfl

/-- Trace positions are op positions: the semantics executes ops in list order. -/
theorem run_opId (backend : Backend) (program : Program) (i : Nat) :
    ((run backend program)[i]?).map TraceEvent.opId = (program.ops.map (·.id))[i]? := by
  simp [run, runOps_eq, Option.map_map, Function.comp_def, interpret_opId]

theorem edgeCodes_nil {program : Program} {edge : StateEdge} (h : edgeCodes program edge = [])
    (scheduled : program.phase = .scheduled) :
    ∃ i j, indexOf? (program.ops.map (·.id)) edge.source = some i ∧
      indexOf? (program.ops.map (·.id)) edge.target = some j ∧ i < j := by
  unfold edgeCodes at h
  simp only [List.append_eq_nil_iff] at h
  obtain ⟨⟨⟨source, target⟩, _⟩, order⟩ := h
  cases hs : indexOf? (program.ops.map (·.id)) edge.source with
  | none => simp [hs] at source
  | some i =>
      cases ht : indexOf? (program.ops.map (·.id)) edge.target with
      | none => simp [ht] at target
      | some j =>
          rw [hs, ht, scheduled] at order
          refine ⟨i, j, rfl, rfl, ?_⟩
          by_cases lt : i < j
          · exact lt
          · simp [orderCode, lt] at order

theorem orderCodes_nil {program : Program} (h : orderCodes program = []) :
    (program.ops.map (·.id)).Nodup ∧ ∀ edge, edge ∈ program.edges -> edgeCodes program edge = [] := by
  simp only [orderCodes, List.append_eq_nil_iff, List.flatMap_eq_nil_iff] at h
  exact ⟨(nodup_of_duplicateCodes h.1).1, h.2⟩

/-- Validator acceptance implies the P3-4 semantics orders every state edge. -/
theorem accepted_schedule_orders_edges {program : Program} (h : orderCodes program = [])
    (scheduled : program.phase = .scheduled) (backend : Backend) {edge : StateEdge}
    (mem : edge ∈ program.edges) : Precedes (run backend program) edge.source edge.target := by
  obtain ⟨nodup, edges⟩ := orderCodes_nil h
  obtain ⟨i, j, hi, hj, lt⟩ := edgeCodes_nil (edges edge mem) scheduled
  refine ⟨⟨i, ?_⟩, ⟨j, ?_⟩, ?_⟩
  · rw [run_opId]; exact getElem?_of_indexOf? hi
  · rw [run_opId]; exact getElem?_of_indexOf? hj
  · intro a b ha hb
    rw [run_opId] at ha hb
    rw [indexOf?_of_getElem? nodup ha] at hi
    rw [indexOf?_of_getElem? nodup hb] at hj
    cases hi; cases hj; exact lt

/-- Validator acceptance keeps every effect-scoped edge inside its scope. -/
theorem accepted_edges_stay_in_scope {program : Program} (h : orderCodes program = [])
    {edge : StateEdge} (mem : edge ∈ program.edges) {scope : Nat} (hasScope : edge.effect = some scope) :
    ∃ effect, program.effects.find? (·.id == scope) = some effect ∧
      opInside program edge.source effect.region = true ∧
      opInside program edge.target effect.region = true := by
  have codes := (orderCodes_nil h).2 edge mem
  unfold edgeCodes at codes
  simp only [List.append_eq_nil_iff] at codes
  obtain ⟨⟨⟨source, target⟩, scopeNil⟩, _⟩ := codes
  have resolved : (indexOf? (program.ops.map (·.id)) edge.source).isSome = true ∧
      (indexOf? (program.ops.map (·.id)) edge.target).isSome = true := by
    constructor
    · cases hs : indexOf? (program.ops.map (·.id)) edge.source <;> simp_all
    · cases ht : indexOf? (program.ops.map (·.id)) edge.target <;> simp_all
  rw [resolved.1, resolved.2] at scopeNil
  unfold scopeCodes at scopeNil
  rw [hasScope] at scopeNil
  cases found : program.effects.find? (·.id == scope) with
  | none => simp [found] at scopeNil
  | some effect =>
      simp only [found] at scopeNil
      refine ⟨effect, rfl, ?_⟩
      by_cases inside : (opInside program edge.source effect.region &&
          opInside program edge.target effect.region) = true
      · simpa using inside
      · simp [inside] at scopeNil

/-- Effect grouping preserves dependency edges: a regrouping of the same ops
with the same edge set that the validator accepts executes the same events and
orders every original edge, whatever its kind. -/
theorem accepted_regrouping_preserves_edges {original grouped : Program}
    (sameOps : grouped.ops.Perm original.ops) (sameEdges : grouped.edges = original.edges)
    (accepted : orderCodes grouped = []) (scheduled : grouped.phase = .scheduled)
    (backend : Backend) :
    (run backend grouped).Perm (run backend original) ∧
      ∀ edge, edge ∈ original.edges -> Precedes (run backend grouped) edge.source edge.target := by
  refine ⟨?_, fun edge mem => ?_⟩
  · simp only [run, runOps_eq, List.nil_append]
    exact sameOps.map _
  · exact accepted_schedule_orders_edges accepted scheduled backend (sameEdges ▸ mem)

end Impeto.Schedule
