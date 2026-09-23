import Impeto.Checker

/-!
Exactness of the C-23 checker: zero false positives and zero false negatives
against the declarative `WellFormed` specification. Because the checker is total
(no `partial` definition), this also shows the check always terminates with a
verdict.
-/

namespace Impeto.Checker

theorem code_nil {holds : Bool} {name : String} : code holds name = [] ↔ holds = true := by
  cases holds <;> simp [code]

/-- The checker reports nothing exactly when the program is well formed. -/
theorem violations_nil_iff (p : Program) : violations p = [] ↔ WellFormed p := by
  simp only [violations, List.append_eq_nil_iff, code_nil, decide_eq_true_eq, WellFormed]
  constructor
  · intro ⟨⟨⟨⟨⟨⟨⟨a, b⟩, c⟩, d⟩, e⟩, f⟩, g⟩, h⟩
    exact ⟨a, b, c, d, e, f, g, h⟩
  · intro ⟨a, b, c, d, e, f, g, h⟩
    exact ⟨⟨⟨⟨⟨⟨⟨a, b⟩, c⟩, d⟩, e⟩, f⟩, g⟩, h⟩

/-- A reported code is always backed by a violated invariant (no false positive). -/
theorem violation_sound (p : Program) (h : violations p ≠ []) : ¬ WellFormed p :=
  fun wf => h ((violations_nil_iff p).mpr wf)

/-- Well-formedness is exactly the conjunction the TS-27 contract states, and in
the scheduled phase it yields the edge-ordering theorem's hypothesis. -/
theorem wellFormed_ordered {p : Program} (wf : WellFormed p) (scheduled : p.phase = .scheduled)
    {edge : StateEdge} (mem : edge ∈ p.edges) : forward p edge = true :=
  wf.2.2.2.2.2.2.2 scheduled edge mem

end Impeto.Checker
