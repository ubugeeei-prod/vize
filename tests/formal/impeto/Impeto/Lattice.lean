namespace Impeto.Lattice

/-!
Independent model of the P3-2 reactivity lattice. It is written from the
declarative rule table in `docs/davinci/plan/phase-3.md` and the P3-2 record,
never from Rust source: every binding fact demands a lower bound on the class,
and the naive evaluator must return the least class meeting every demand.
`LatticeLaws` proves that contract; `LatticeFixture` compares this evaluator
with the Rust evaluator's committed fact page.
-/

/-- The value axis, ordered from most to least stable. -/
inductive Class where
  | static
  | propsStable
  | reactive
  | unstable
deriving Repr, DecidableEq

namespace Class

def rank : Class -> Nat
  | .static => 0
  | .propsStable => 1
  | .reactive => 2
  | .unstable => 3

instance : LE Class := ⟨fun a b => a.rank ≤ b.rank⟩

instance (a b : Class) : Decidable (a ≤ b) :=
  inferInstanceAs (Decidable (a.rank ≤ b.rank))

/-- "Less stable wins", stated as a table so the least-upper-bound theorem is
not true by definition. -/
def join : Class -> Class -> Class
  | .unstable, _ => .unstable
  | _, .unstable => .unstable
  | .reactive, _ => .reactive
  | _, .reactive => .reactive
  | .propsStable, _ => .propsStable
  | _, .propsStable => .propsStable
  | .static, .static => .static

def spelling : Class -> String
  | .static => "static"
  | .propsStable => "props-stable"
  | .reactive => "reactive"
  | .unstable => "unstable"

def all : List Class := [.static, .propsStable, .reactive, .unstable]

end Class

/-- React-Compiler-style effect vocabulary, in the fact page's print order. -/
inductive Effect where
  | freeze
  | capture
  | readProp
  | readReactive
  | mutateLocal
  | mutateGlobal
  | callUnknown
  | allocate
deriving Repr, DecidableEq

namespace Effect

/-- Declarative rule: a binding carrying this effect is at least this unstable. -/
def floor : Effect -> Class
  | .freeze | .readProp | .allocate => .propsStable
  | .capture | .readReactive | .mutateLocal => .reactive
  | .mutateGlobal | .callUnknown => .unstable

def spelling : Effect -> String
  | .freeze => "freeze"
  | .capture => "capture"
  | .readProp => "read-prop"
  | .readReactive => "read-reactive"
  | .mutateLocal => "mutate-local"
  | .mutateGlobal => "mutate-global"
  | .callUnknown => "call-unknown"
  | .allocate => "allocate"

def all : List Effect :=
  [.freeze, .capture, .readProp, .readReactive, .mutateLocal, .mutateGlobal, .callUnknown, .allocate]

end Effect

/-- Where the binding value enters the component. -/
inductive Origin where
  | «local»
  | prop
  | provideInject
  | templateRef
deriving Repr, DecidableEq

namespace Origin

def floor : Origin -> Class
  | .local => .static
  | .prop => .propsStable
  | .provideInject | .templateRef => .reactive

def spelling : Origin -> String
  | .local => "local"
  | .prop => "prop"
  | .provideInject => "provide-inject"
  | .templateRef => "template-ref"

def all : List Origin := [.local, .prop, .provideInject, .templateRef]

end Origin

/-- Escape-analysis result; each escape demotes toward `unstable`. -/
inductive Escape where
  | none
  | returned
  | stored
  | global
deriving Repr, DecidableEq

namespace Escape

def floor : Escape -> Class
  | .none => .static
  | .returned => .propsStable
  | .stored => .reactive
  | .global => .unstable

def spelling : Escape -> String
  | .none => "none"
  | .returned => "returned"
  | .stored => "stored"
  | .global => "global"

def all : List Escape := [.none, .returned, .stored, .global]

end Escape

/-- Epistemic axis, orthogonal to the value axis. -/
inductive Verdict where
  | proven
  | refuted
  | unknown
deriving Repr, DecidableEq

namespace Verdict

def spelling : Verdict -> String
  | .proven => "proven"
  | .refuted => "refuted"
  | .unknown => "unknown"

def all : List Verdict := [.proven, .refuted, .unknown]

end Verdict

/-- One binding summary. Effects are a set: only membership is observable. -/
structure Input where
  origin : Origin
  effects : List Effect
  escape : Escape
  verdict : Verdict
deriving Repr, DecidableEq

def effectsFloor (effects : List Effect) : Class :=
  effects.foldr (fun effect floor => effect.floor.join floor) .static

/-- Naive evaluator: join every floor, then apply the provide/inject cap. -/
def classify (input : Input) : Class :=
  let value := (input.origin.floor.join (effectsFloor input.effects)).join input.escape.floor
  if input.origin = .provideInject then value.join .reactive else value

/-- The declarative rule spec: every premise demands a lower bound. -/
inductive Demand (input : Input) : Class -> Prop where
  | origin : Demand input input.origin.floor
  | effect {effect : Effect} : effect ∈ input.effects -> Demand input effect.floor
  | escape : Demand input input.escape.floor
  | provideInject : input.origin = .provideInject -> Demand input .reactive

/-- A class satisfies the spec when it meets every demanded lower bound. -/
def Admissible (input : Input) (value : Class) : Prop :=
  ∀ demand, Demand input demand -> demand ≤ value

/-- `j` carries at least the instability evidence of `i`. -/
structure Weaker (i j : Input) : Prop where
  origin : i.origin.floor ≤ j.origin.floor
  effects : ∀ effect, effect ∈ i.effects -> effect ∈ j.effects
  escape : i.escape.floor ≤ j.escape.floor

/-- Consumers fire only on proven values. -/
def fires (input : Input) (value : Class) : Bool :=
  input.verdict == .proven && classify input == value

end Impeto.Lattice
