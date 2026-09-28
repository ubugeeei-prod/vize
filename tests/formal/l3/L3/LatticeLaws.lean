import Impeto.Lattice

/-!
P3-15 lattice laws. The value axis is a four-point chain, `join` is its least
upper bound, the naive evaluator returns exactly the least class admitted by
the declarative rule spec, and classification is monotone: more effects, a
weaker origin or a wider escape never make a binding more stable.
-/

namespace Impeto.Lattice
namespace Class

theorem le_refl (a : Class) : a ≤ a := Nat.le_refl a.rank

theorem le_trans {a b c : Class} : a ≤ b -> b ≤ c -> a ≤ c := Nat.le_trans

theorem le_antisymm {a b : Class} : a ≤ b -> b ≤ a -> a = b := by
  cases a <;> cases b <;> decide

theorem le_total (a b : Class) : a ≤ b ∨ b ≤ a := Nat.le_total a.rank b.rank

theorem static_le (a : Class) : Class.static ≤ a := Nat.zero_le a.rank

theorem le_unstable (a : Class) : a ≤ Class.unstable := by cases a <;> decide

/-- The value axis is the strict chain static < props-stable < reactive < unstable. -/
theorem chain :
    Class.static.rank < Class.propsStable.rank ∧
    Class.propsStable.rank < Class.reactive.rank ∧
    Class.reactive.rank < Class.unstable.rank := by decide

theorem join_comm (a b : Class) : a.join b = b.join a := by
  cases a <;> cases b <;> rfl

theorem join_assoc (a b c : Class) : (a.join b).join c = a.join (b.join c) := by
  cases a <;> cases b <;> cases c <;> rfl

theorem join_idem (a : Class) : a.join a = a := by cases a <;> rfl

theorem static_join (a : Class) : Class.static.join a = a := by cases a <;> rfl

theorem join_static (a : Class) : a.join .static = a := by cases a <;> rfl

theorem unstable_join (a : Class) : Class.unstable.join a = .unstable := by cases a <;> rfl

theorem join_unstable (a : Class) : a.join .unstable = .unstable := by cases a <;> rfl

theorem le_join_left (a b : Class) : a ≤ a.join b := by cases a <;> cases b <;> decide

theorem le_join_right (a b : Class) : b ≤ a.join b := by cases a <;> cases b <;> decide

theorem join_le {a b c : Class} : a ≤ c -> b ≤ c -> a.join b ≤ c := by
  cases a <;> cases b <;> cases c <;> decide

/-- `join` is the least upper bound of the chain order. -/
theorem join_le_iff {a b c : Class} : a.join b ≤ c ↔ a ≤ c ∧ b ≤ c :=
  ⟨fun h => ⟨le_trans (le_join_left a b) h, le_trans (le_join_right a b) h⟩,
    fun ⟨ha, hb⟩ => join_le ha hb⟩

/-- Joins move only toward less stable values. -/
theorem join_eq_right_iff {a b : Class} : a.join b = b ↔ a ≤ b := by
  cases a <;> cases b <;> decide

/-- The table join is the rank maximum, the form a bit-rank implementation computes. -/
theorem rank_join (a b : Class) : (a.join b).rank = max a.rank b.rank := by
  cases a <;> cases b <;> rfl

theorem join_mono {a b c d : Class} (hac : a ≤ c) (hbd : b ≤ d) : a.join b ≤ c.join d :=
  join_le (le_trans hac (le_join_left c d)) (le_trans hbd (le_join_right c d))

end Class

theorem effectsFloor_cons (effect : Effect) (effects : List Effect) :
    effectsFloor (effect :: effects) = effect.floor.join (effectsFloor effects) := rfl

/-- The effect floor is a join homomorphism from set union. -/
theorem effectsFloor_append (s t : List Effect) :
    effectsFloor (s ++ t) = (effectsFloor s).join (effectsFloor t) := by
  induction s with
  | nil => exact (Class.static_join _).symm
  | cons effect s ih => rw [List.cons_append, effectsFloor_cons, effectsFloor_cons, ih, Class.join_assoc]

theorem floor_le_effectsFloor : ∀ {s : List Effect} {effect : Effect},
    effect ∈ s -> effect.floor ≤ effectsFloor s
  | _ :: _, _, .head _ => Class.le_join_left _ _
  | _ :: _, _, .tail _ h => Class.le_trans (floor_le_effectsFloor h) (Class.le_join_right _ _)

theorem effectsFloor_le : ∀ {s : List Effect} {bound : Class},
    (∀ effect, effect ∈ s -> effect.floor ≤ bound) -> effectsFloor s ≤ bound
  | [], bound, _ => Class.static_le bound
  | effect :: _, _, h =>
      Class.join_le (h effect (.head _)) (effectsFloor_le fun other mem => h other (.tail _ mem))

/-- Only membership matters: adding effects never makes a binding more stable. -/
theorem effectsFloor_mono {s t : List Effect} (h : ∀ effect, effect ∈ s -> effect ∈ t) :
    effectsFloor s ≤ effectsFloor t :=
  effectsFloor_le fun effect mem => floor_le_effectsFloor (h effect mem)

/-- The same rule table read as tiers: the first matching tier wins. -/
def effectsFloorByTier (s : List Effect) : Class :=
  if Effect.mutateGlobal ∈ s ∨ Effect.callUnknown ∈ s then .unstable
  else if Effect.readReactive ∈ s ∨ Effect.mutateLocal ∈ s ∨ Effect.capture ∈ s then .reactive
  else if Effect.readProp ∈ s ∨ Effect.freeze ∈ s ∨ Effect.allocate ∈ s then .propsStable
  else .static

theorem effectsFloor_eq_tier (s : List Effect) : effectsFloor s = effectsFloorByTier s := by
  induction s with
  | nil => rfl
  | cons effect s ih =>
      rw [effectsFloor_cons, ih]
      cases effect <;>
      by_cases hu : (Effect.mutateGlobal ∈ s ∨ Effect.callUnknown ∈ s) <;>
      by_cases hr : (Effect.readReactive ∈ s ∨ Effect.mutateLocal ∈ s ∨ Effect.capture ∈ s) <;>
      by_cases hp : (Effect.readProp ∈ s ∨ Effect.freeze ∈ s ∨ Effect.allocate ∈ s) <;>
      simp_all [effectsFloorByTier, Effect.floor, Class.join]

def base (input : Input) : Class :=
  (input.origin.floor.join (effectsFloor input.effects)).join input.escape.floor

theorem classify_eq (input : Input) : classify input =
    if input.origin = .provideInject then (base input).join .reactive else base input := rfl

theorem base_le_classify (input : Input) : base input ≤ classify input := by
  rw [classify_eq]
  split
  · exact Class.le_join_left _ _
  · exact Class.le_refl _

/-- Provide/inject-derived bindings are never more stable than `reactive`. -/
theorem provideInject_capped {input : Input} (h : input.origin = .provideInject) :
    Class.reactive ≤ classify input := by
  rw [classify_eq, if_pos h]
  exact Class.le_join_right _ _

/-- With the current origin floors the cap is already implied by the origin rule. -/
theorem provideInject_cap_redundant (input : Input) : classify input = base input := by
  rw [classify_eq]
  split
  · next h =>
      have : Class.reactive ≤ base input := by
        have origin : input.origin.floor ≤ base input :=
          Class.le_trans (Class.le_join_left _ _) (Class.le_join_left _ _)
        rw [h] at origin
        exact origin
      rw [Class.join_comm]
      exact Class.join_eq_right_iff.mpr this
  · rfl

/-- Soundness: the naive evaluator satisfies every declarative demand. -/
theorem classify_admissible (input : Input) : Admissible input (classify input) := by
  intro demand h
  have hb := base_le_classify input
  cases h with
  | origin =>
      exact Class.le_trans (Class.le_trans (Class.le_join_left _ _) (Class.le_join_left _ _)) hb
  | effect mem =>
      exact Class.le_trans (Class.le_trans (floor_le_effectsFloor mem)
        (Class.le_trans (Class.le_join_right _ _) (Class.le_join_left _ _))) hb
  | escape => exact Class.le_trans (Class.le_join_right _ _) hb
  | provideInject h => exact provideInject_capped h

/-- Minimality: every admissible class is at least the evaluated class. -/
theorem classify_least {input : Input} {bound : Class} (h : Admissible input bound) :
    classify input ≤ bound := by
  have hb : base input ≤ bound :=
    Class.join_le (Class.join_le (h _ .origin)
      (effectsFloor_le fun _ mem => h _ (.effect mem))) (h _ .escape)
  rw [classify_eq]
  split
  · next pi => exact Class.join_le hb (h _ (.provideInject pi))
  · exact hb

/-- The evaluator computes exactly the least admissible class. -/
theorem classify_spec {input : Input} {bound : Class} :
    classify input ≤ bound ↔ Admissible input bound :=
  ⟨fun h demand mem => Class.le_trans (classify_admissible input demand mem) h, classify_least⟩

/-- Classification monotonicity. -/
theorem classify_monotone {i j : Input} (h : Weaker i j) : classify i ≤ classify j := by
  apply classify_least
  intro demand mem
  have hj := classify_admissible j
  cases mem with
  | origin => exact Class.le_trans h.origin (hj _ .origin)
  | effect mem => exact hj _ (.effect (h.effects _ mem))
  | escape => exact Class.le_trans h.escape (hj _ .escape)
  | provideInject pi =>
      have origin := h.origin
      rw [pi] at origin
      exact Class.le_trans origin (hj _ .origin)

theorem classify_add_effect (input : Input) (effect : Effect) :
    classify input ≤ classify { input with effects := effect :: input.effects } :=
  classify_monotone ⟨Class.le_refl _, fun _ mem => .tail _ mem, Class.le_refl _⟩

/-- Escape analysis only demotes. -/
theorem classify_escape_demotes (input : Input) (escape : Escape)
    (h : input.escape.floor ≤ escape.floor) :
    classify input ≤ classify { input with escape := escape } :=
  classify_monotone ⟨Class.le_refl _, fun _ mem => mem, h⟩

/-- Effect sets are sets: order and duplicates are unobservable. -/
theorem classify_effects_extensional {i j : Input} (origin : i.origin = j.origin)
    (escape : i.escape = j.escape) (effects : ∀ effect, effect ∈ i.effects ↔ effect ∈ j.effects) :
    classify i = classify j :=
  Class.le_antisymm
    (classify_monotone ⟨origin ▸ Class.le_refl _, fun e m => (effects e).mp m,
      escape ▸ Class.le_refl _⟩)
    (classify_monotone ⟨origin ▸ Class.le_refl _, fun e m => (effects e).mpr m,
      escape ▸ Class.le_refl _⟩)

/-- The verdict axis is orthogonal to the value axis. -/
theorem classify_verdict_orthogonal (input : Input) (verdict : Verdict) :
    classify { input with verdict } = classify input := rfl

theorem fires_iff {input : Input} {value : Class} :
    fires input value = true ↔ input.verdict = .proven ∧ classify input = value := by
  simp [fires]

theorem fires_only_proven {input : Input} {value : Class} (h : fires input value = true) :
    input.verdict = .proven := (fires_iff.mp h).1

/-- `static` is reachable only from an inert local binding. -/
theorem classify_static_iff (input : Input) : classify input = .static ↔
    input.origin = .local ∧ input.effects = [] ∧ input.escape = .none := by
  constructor
  · intro h
    have adm : Admissible input .static := classify_spec.mp (h ▸ Class.le_refl _)
    refine ⟨?_, ?_, ?_⟩
    · have := adm _ .origin
      revert this; cases input.origin <;> decide
    · match hs : input.effects with
      | [] => rfl
      | effect :: _ =>
          have := adm _ (.effect (hs ▸ List.Mem.head _ : effect ∈ input.effects))
          exact absurd this (by cases effect <;> decide)
    · have := adm _ .escape
      revert this; cases input.escape <;> decide
  · intro ⟨origin, effects, escape⟩
    rw [provideInject_cap_redundant, base, origin, effects, escape]
    rfl

end Impeto.Lattice
