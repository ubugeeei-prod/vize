import Std

namespace Davinci.StyleFormat

structure Attribute where
  name : String
  value : Option String
  deriving DecidableEq

def plain (name : String) : Bool :=
  name == "lang" || name == "scoped" || name == "module"

-- The Rust scan consults the preceding attributes on the original block.
def scan (after : List Attribute) (previous : List String) : List Attribute → Bool
  | [] => true
  | attr :: rest =>
    plain attr.name && !previous.contains attr.name && after.contains attr &&
      scan after (attr.name :: previous) rest

def accepts (before after : List Attribute) : Bool :=
  before.length == after.length && (before == after || scan after [] before)

def permitted (attrs : List Attribute) : Prop :=
  attrs.Pairwise (fun a b => a.name ≠ b.name) ∧ ∀ a ∈ attrs, plain a.name = true

private theorem scan_iff (before after : List Attribute) (previous : List String) :
    scan after previous before = true ↔
      permitted before ∧ before ⊆ after ∧ ∀ a ∈ before, a.name ∉ previous := by
  induction before generalizing previous with
  | nil => simp [scan, permitted]
  | cons a rest ih =>
    simp [scan, ih, permitted, List.cons_subset]
    grind

-- Distinct source pairs found in an equal-size target exhaust that target.
private theorem perm_of_membership {before after : List Attribute}
    (distinct : before.Nodup) (found : before ⊆ after)
    (size : before.length = after.length) : before.Perm after := by
  induction before generalizing after with
  | nil => simpa using size.symm
  | cons a rest ih =>
    obtain ⟨fresh, tail⟩ := List.nodup_cons.mp distinct
    have present : a ∈ after := found (by simp)
    have remaining : rest ⊆ after.erase a := by
      intro b hb
      have other : b ≠ a := fun h => fresh (h ▸ hb)
      exact (List.mem_erase_of_ne other).mpr (found (List.mem_cons_of_mem _ hb))
    have length : rest.length = (after.erase a).length := by
      rw [List.length_erase_of_mem present]
      simp at size
      omega
    exact ((ih tail remaining length).cons a).trans (List.perm_cons_erase present).symm

theorem accepted_iff (before after : List Attribute) :
    accepts before after = true ↔ before = after ∨ (permitted before ∧ before.Perm after) := by
  simp [accepts, scan_iff]
  constructor
  · rintro ⟨size, identical | ⟨flags, found⟩⟩
    · exact Or.inl identical
    · have distinct : before.Nodup :=
        flags.1.imp (fun different h => different (congrArg Attribute.name h))
      exact Or.inr ⟨flags, perm_of_membership distinct found size⟩
  · rintro (rfl | ⟨flags, reordered⟩)
    · exact ⟨rfl, Or.inl rfl⟩
    · exact ⟨reordered.length_eq, Or.inr ⟨flags, reordered.subset⟩⟩

#print axioms accepted_iff
end Davinci.StyleFormat
