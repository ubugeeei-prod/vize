import Impeto.View

namespace Impeto.Incremental
open Lean

inductive Node where
  | text (value : String)
  | element (address : String) (identity : Nat) (tag : String)
      (attributes : List (String × Json)) (disabled : Bool)
      (event : Option Json) (children : List Node)
deriving BEq

structure State where
  roots : List Node := []
  nextIdentity : Nat := 0

def address : Node -> Option String
  | .text _ => none
  | .element value _ _ _ _ _ _ => some value

mutual
/-- Forget retained identities: the view a materialized tree displays. -/
def erase : Node -> View
  | .text value => .text value
  | .element address _ tag attrs disabled event children =>
      .element address tag attrs disabled event (eraseAll children)

def eraseAll : List Node -> List View
  | [] => []
  | node :: rest => erase node :: eraseAll rest
end

/-- Reuse the first previous sibling with the same S3 address, or allocate. -/
def retain (previous : List Node) (owner tag : String) (next : Nat) :
    Except String (Nat × List Node × Nat) :=
  match previous.find? (fun node => address node == some owner) with
  | some (.element _ identity oldTag _ _ _ children) =>
      if tag = oldTag then .ok (identity, children, next)
      else .error "stable S3 identity changed element kind"
  | _ => .ok (next, [], next + 1)

-- This update machine owns retained identities and the previous materialized
-- tree. It receives a fresh view, matches within each parent, patches payloads,
-- inserts missing nodes, and retires absent nodes. No runtime output is input.
mutual
def reconcile : Nat -> List Node -> List View -> Nat -> Except String (List Node × Nat)
  | 0, _, _, _ => .error "incremental tree exceeds structural fuel"
  | fuel + 1, previous, desired, next => siblings fuel previous [] desired next
termination_by fuel _ _ _ => (fuel, 0)

def siblings (fuel : Nat) (previous : List Node) (seen : List String) :
    List View -> Nat -> Except String (List Node × Nat)
  | [], next => .ok ([], next)
  | .text value :: rest, next =>
      match siblings fuel previous seen rest next with
      | .error message => .error message
      | .ok (nodes, next) => .ok (.text value :: nodes, next)
  | .element owner tag attrs disabled event children :: rest, next =>
      if owner ∈ seen then .error "duplicate materialized address" else
      match retain previous owner tag next with
      | .error message => .error message
      | .ok (identity, oldChildren, next) =>
          match reconcile fuel oldChildren children next with
          | .error message => .error message
          | .ok (updated, next) =>
              match siblings fuel previous (owner :: seen) rest next with
              | .error message => .error message
              | .ok (nodes, next) =>
                  .ok (.element owner identity tag attrs disabled event updated :: nodes, next)
termination_by desired _ => (fuel, desired.length + 1)
end

def json (node : Node) : Json := View.json (erase node)

def tree (state : State) : Json := View.tree (eraseAll state.roots)

def update (fuel : Nat) (state : State) (view : List View) : Except String State :=
  match reconcile fuel state.roots view state.nextIdentity with
  | .error message => .error message
  | .ok (roots, nextIdentity) =>
      let result := { roots, nextIdentity : State }
      if tree result == View.tree view then .ok result
      else .error "incremental versus from-scratch tree drift"

structure Target where
  name : String
  identity : Nat
  tag : String
  disabled : Bool
  event : Option Json

partial def targets : Node -> List Target
  | .text _ => []
  | .element _ identity tag attrs disabled event children =>
      let own := match attrs.find? (fun field => field.1 == "data-id") with
        | some (_, .str name) => [{ name, identity, tag, disabled, event : Target }]
        | _ => []
      own ++ children.flatMap targets

def allTargets (state : State) : List Target := state.roots.flatMap targets

def identities (state : State) : Json :=
  .arr ((allTargets state).map (fun target => .arr #[.str target.name, toJson target.identity])).toArray

def click (state : State) (name : String) : Except String (Option Json) := do
  let [target] := (allTargets state).filter (fun target => target.name == name)
    | throw "expected one live interaction target"
  if target.tag != "button" then throw "expected a native button"
  pure (if target.disabled then none else target.event)

end Impeto.Incremental
