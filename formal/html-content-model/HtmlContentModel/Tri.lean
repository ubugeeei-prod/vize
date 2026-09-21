/-
Three-valued facts (`crates/vize_patina/src/html_content_model/tri.rs`):
`maybe` is "depends on what the template does not show". A connective is
sound when, for every pair of concrete truth values its arguments allow,
its result allows the concrete result (`admits`).
-/
namespace HtmlContentModel

inductive Tri where
  | yes
  | maybe
  | no
  deriving DecidableEq, Repr

namespace Tri

/-- `t` is consistent with the concrete truth value `b`. -/
def admits : Tri → Bool → Prop
  | yes, b => b = true
  | no, b => b = false
  | maybe, _ => True

def not : Tri → Tri
  | yes => no
  | no => yes
  | maybe => maybe

def and : Tri → Tri → Tri
  | no, _ => no
  | _, no => no
  | yes, yes => yes
  | _, _ => maybe

def or : Tri → Tri → Tri
  | yes, _ => yes
  | _, yes => yes
  | no, no => no
  | _, _ => maybe

theorem not_sound {t : Tri} {b : Bool} (h : admits t b) : admits (not t) (!b) := by
  cases t <;> cases b <;> simp_all [admits, not]

theorem and_sound {s t : Tri} {a b : Bool} (ha : admits s a) (hb : admits t b) :
    admits (and s t) (a && b) := by
  cases s <;> cases t <;> cases a <;> cases b <;> simp_all [admits, and]

theorem or_sound {s t : Tri} {a b : Bool} (ha : admits s a) (hb : admits t b) :
    admits (or s t) (a || b) := by
  cases s <;> cases t <;> cases a <;> cases b <;> simp_all [admits, or]

/-- Kleene disjunction is the De Morgan dual of conjunction. -/
theorem or_eq_not_and_not (s t : Tri) : or s t = not (and (not s) (not t)) := by
  cases s <;> cases t <;> rfl

/-- A definite fact admits exactly one concrete value. -/
theorem admits_unique {t : Tri} {a b : Bool} (ht : t ≠ maybe) (ha : admits t a)
    (hb : admits t b) : a = b := by
  cases t <;> simp_all [admits]

end Tri
end HtmlContentModel
