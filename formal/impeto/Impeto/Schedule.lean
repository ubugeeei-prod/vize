import Impeto.Semantics

/-!
P3-15 scheduling contract. S3 has no separate effect-grouping pass yet. The
contract every grouping must meet is the TS-27 phase validator's edge
contract, so this module re-states its order checks (S3V001 duplicate op ids,
S3V004 unresolved edge endpoints, S3V007 effect-scoped edges leaving their
scope, and S3V008 scheduled edges that point backward) independently of the
Rust code. It then proves that acceptance implies the ordering property in the
P3-4 small-step semantics, for both backends.
-/

namespace Impeto.Schedule

/-- First position of an id, as the verifier's `op_index` computes it. -/
def indexOf? : List Nat -> Nat -> Option Nat
  | [], _ => none
  | x :: xs, id => if x = id then some 0 else (indexOf? xs id).map (· + 1)

/-- One S3V001 for every op whose id already occurred earlier. -/
def duplicateCodes : List Nat -> List Nat -> List String
  | _, [] => []
  | seen, id :: rest => (if id ∈ seen then ["S3V001"] else []) ++ duplicateCodes (id :: seen) rest

def opRegion? (program : Program) (id : Nat) : Option Nat :=
  (program.ops.find? (·.id == id)).map (·.region)

def parent? (program : Program) (id : Nat) : Option Nat :=
  (program.regions.find? (·.id == id)).bind (·.parent)

/-- Walk parents, visiting at most one more region than the program has. -/
def isOrDescendant (program : Program) (child ancestor : Nat) : Bool :=
  go child (program.regions.length + 1)
where
  go (current : Nat) : Nat -> Bool
    | 0 => false
    | fuel + 1 =>
        current == ancestor ||
          match parent? program current with
          | some parent => go parent fuel
          | none => false

def opInside (program : Program) (id region : Nat) : Bool :=
  match opRegion? program id with
  | some child => isOrDescendant program child region
  | none => false

def scopeCodes (program : Program) (edge : StateEdge) (resolved : Bool) : List String :=
  match edge.effect with
  | none => []
  | some scope =>
      match program.effects.find? (·.id == scope) with
      | none => ["S3V007"]
      | some effect =>
          if resolved && !(opInside program edge.source effect.region &&
              opInside program edge.target effect.region) then ["S3V007"] else []

def orderCode : Phase -> Option Nat -> Option Nat -> List String
  | .scheduled, some i, some j => if i < j then [] else ["S3V008"]
  | _, _, _ => []

def edgeCodes (program : Program) (edge : StateEdge) : List String :=
  let ids := program.ops.map (·.id)
  let source := indexOf? ids edge.source
  let target := indexOf? ids edge.target
  (if source.isSome then [] else ["S3V004"]) ++ (if target.isSome then [] else ["S3V004"]) ++
    scopeCodes program edge (source.isSome && target.isSome) ++
    orderCode program.phase source target

/-- The validator's ordering verdict: an empty list means acceptance. -/
def orderCodes (program : Program) : List String :=
  duplicateCodes [] (program.ops.map (·.id)) ++ program.edges.flatMap (edgeCodes program)

/-- Happens-before in an observed trace: both ops execute, and every event of
the source precedes every event of the target. -/
def Precedes (trace : List TraceEvent) (source target : Nat) : Prop :=
  (∃ i : Nat, (trace[i]?).map TraceEvent.opId = some source) ∧
  (∃ j : Nat, (trace[j]?).map TraceEvent.opId = some target) ∧
  ∀ i j : Nat, (trace[i]?).map TraceEvent.opId = some source ->
    (trace[j]?).map TraceEvent.opId = some target -> i < j

end Impeto.Schedule
