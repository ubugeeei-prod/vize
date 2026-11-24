/- Composition of declarations, references and lexical scopes on one checked
   script unit. Each list retains the original table's payload and multiplicity. -/
namespace Davinci.ScriptPosition

inductive Unique (α : Type) where
  | empty
  | row (value : α)
  | ambiguous
  deriving DecidableEq

def unique : List α → Unique α
  | [] => .empty
  | [x] => .row x
  | _ :: _ :: _ => .ambiguous

def select : Unique D → Unique R → Unique (Sum D R)
  | .ambiguous, _ => .ambiguous
  | _, .ambiguous => .ambiguous
  | .empty, .empty => .empty
  | .row d, .empty => .row (.inl d)
  | .empty, .row r => .row (.inr r)
  | .row _, .row _ => .ambiguous

/-- Joining the two selected rows gives the same answer as inspecting every
    original tagged row, even when equal payloads occur in different tables. -/
theorem select_original (ds : List D) (rs : List R) :
    select (unique ds) (unique rs) = unique (ds.map Sum.inl ++ rs.map Sum.inr) := by
  match ds, rs with
  | [], [] | [], [_] | [], _ :: _ :: _ | [_], [] | [_], [_] | [_], _ :: _ :: _
    | _ :: _ :: _, _ => rfl

def project (site : Unique α) (read : Option α → Unique β) : Unique β :=
  match site with
  | .empty => read none
  | .row x => read (some x)
  | .ambiguous => .ambiguous

structure Answers (R B S : Type) where
  reference : Unique R
  binding : Unique B
  scope : Unique S

def composed (ds : List D) (rs : List R)
    (binding : Option (Sum D R) → Unique B) (scope : Option (Sum D R) → Unique S) :
    Answers R B S :=
  let reference := unique rs
  let site := select (unique ds) reference
  ⟨reference, project site binding, project site scope⟩

/-- Each projection may independently return an absent or ambiguous answer.
    In particular a containing-scope refusal cannot erase a reference. -/
theorem all_products (ds : List D) (rs : List R)
    (binding : Option (Sum D R) → Unique B) (scope : Option (Sum D R) → Unique S) :
    composed ds rs binding scope =
      ⟨unique rs, project (unique (ds.map Sum.inl ++ rs.map Sum.inr)) binding,
       project (unique (ds.map Sum.inl ++ rs.map Sum.inr)) scope⟩ := by
  exact congrArg (fun site => Answers.mk (unique rs) (project site binding) (project site scope))
    (select_original ds rs)

theorem collision_preserves_reference (d : D) (r : R)
    (binding : Option (Sum D R) → Unique B) (scope : Option (Sum D R) → Unique S) :
    composed [d] [r] binding scope = ⟨.row r, .ambiguous, .ambiguous⟩ := rfl

#print axioms select_original
#print axioms all_products
#print axioms collision_preserves_reference

end Davinci.ScriptPosition
