/-
The stack-of-open-elements walk (`Chain::walk` in
`crates/vize_patina/src/html_content_model/chain.rs`). A scan classifies
each open element, innermost first, as the `target` it looks for, a `stop`
(a scope boundary, a special element, a marker), or `pass`. A frame whose
namespace is ambiguous (a component root mounted in HTML or SVG) is
classified once per possible namespace; if the classifications differ the
scan is `maybe`. Below the outermost frame the chain ends either at the
document (`document`: nothing further is open) or at an unknown mount point
(`truncated`: anything may be open).
-/
import HtmlContentModel.Tri

namespace HtmlContentModel

inductive Role where
  | target
  | stop
  | pass
  deriving DecidableEq, Repr

inductive Base where
  | document
  | truncated
  deriving DecidableEq, Repr

variable {Frame Ns : Type}

/-- The checker's walk over an abstract chain (frames innermost first). -/
def walk (role : Frame → Ns → Role) (nss : Frame → List Ns) : List Frame → Base → Tri
  | [], .document => .no
  | [], .truncated => .maybe
  | f :: fs, base =>
    match nss f with
    | [] => walk role nss fs base
    | n :: rest =>
      if rest.all (fun m => role f m == role f n) then
        match role f n with
        | .target => .yes
        | .stop => .no
        | .pass => walk role nss fs base
      else .maybe

/-- The same walk over a concrete stack: every element in one namespace, the
document at the bottom. -/
def run (role : Frame → Ns → Role) : List (Frame × Ns) → Bool
  | [] => false
  | (f, n) :: s =>
    match role f n with
    | .target => true
    | .stop => false
    | .pass => run role s

/-- `s` is a concrete stack the abstract chain describes: the chain's frames,
each in one of its possible namespaces, over nothing (a `document` chain) or
over any ancestors at all (a `truncated` chain). -/
inductive Describes (nss : Frame → List Ns) : List Frame → Base → List (Frame × Ns) → Prop
  | document : Describes nss [] .document []
  | truncated (s : List (Frame × Ns)) : Describes nss [] .truncated s
  | cons {f : Frame} {fs : List Frame} {base : Base} {s : List (Frame × Ns)} (n : Ns) :
      n ∈ nss f → Describes nss fs base s → Describes nss (f :: fs) base ((f, n) :: s)

/-- Soundness: whatever the walk decides holds on every concrete stack the
chain describes, in particular whatever the unknown mount point turns out
to be. This is the walk-level form of what
`truncated_verdicts_hold_in_every_faithful_context` checks for whole verdicts
by enumeration in `crates/vize_patina/tests/html_content_model_differential.rs`. -/
theorem walk_sound {role : Frame → Ns → Role} {nss : Frame → List Ns} :
    ∀ {fs : List Frame} {base : Base} {s : List (Frame × Ns)},
      Describes nss fs base s → (walk role nss fs base).admits (run role s) := by
  intro fs base s h
  induction h with
  | document => simp [walk, run, Tri.admits]
  | truncated s => simp [walk, Tri.admits]
  | @cons f fs base s n hn _ ih =>
    unfold walk
    split
    · rename_i hnil
      rw [hnil] at hn
      cases hn
    · rename_i m rest hcons
      rw [hcons] at hn
      split
      · rename_i hall
        have hsame : role f n = role f m := by
          cases hn with
          | head => rfl
          | tail _ hmem =>
            have := List.all_eq_true.mp hall n hmem
            simpa using this
        cases hrole : role f m with
        | target => simp [run, hsame, hrole, Tri.admits]
        | stop => simp [run, hsame, hrole, Tri.admits]
        | pass => simpa [run, hsame, hrole] using ih
      · simp [Tri.admits]

/-- Stability: a verdict the walk reaches on a truncated chain survives any
ancestors the context later reveals. -/
theorem walk_stable {role : Frame → Ns → Role} {nss : Frame → List Ns} :
    ∀ (fs more : List Frame) (base : Base) (v : Tri),
      walk role nss fs .truncated = v → v ≠ .maybe → walk role nss (fs ++ more) base = v := by
  intro fs
  induction fs with
  | nil => intro more base v h hv; simp [walk] at h; exact absurd h.symm hv
  | cons f fs ih =>
    intro more base v h hv
    cases hn : nss f with
    | nil =>
      simp only [List.cons_append, walk, hn] at h ⊢
      exact ih more base v h hv
    | cons m rest =>
      simp only [List.cons_append, walk, hn] at h ⊢
      by_cases hall : (rest.all fun x => role f x == role f m) = true
      · rw [if_pos hall] at h ⊢
        cases hrole : role f m with
        | target => simp only [hrole] at h ⊢; exact h
        | stop => simp only [hrole] at h ⊢; exact h
        | pass => simp only [hrole] at h ⊢; exact ih more base v h hv
      · rw [if_neg hall] at h
        exact absurd h.symm hv

/-- A violation is proven when every possible parent dispatch diverges. -/
def proven {D : Type} (ds : List D) (diverges : D → Tri) : Bool :=
  ds.all (fun d => diverges d == .yes)

/-- A proven violation holds for whichever dispatch the page really takes. -/
theorem proven_sound {D : Type} {ds : List D} {diverges : D → Tri} {concrete : D → Bool}
    (hsound : ∀ d ∈ ds, (diverges d).admits (concrete d)) (h : proven ds diverges = true) :
    ∀ d ∈ ds, concrete d = true := by
  intro d hd
  have hyes : diverges d = .yes := by
    simpa [proven] using List.all_eq_true.mp h d hd
  simpa [hyes, Tri.admits] using hsound d hd

/-- Every verdict is decided by evaluation. -/
instance {D : Type} (ds : List D) (diverges : D → Tri) : Decidable (proven ds diverges = true) :=
  inferInstance

end HtmlContentModel
