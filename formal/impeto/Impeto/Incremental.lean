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

-- This update machine owns retained identities and the previous materialized
-- tree. It receives a fresh view, matches within each parent, patches payloads,
-- inserts missing nodes, and retires absent nodes. No runtime output is input.
def reconcile (fuel : Nat) (previous : List Node) (desired : List View)
    (next : Nat) : Except String (List Node × Nat) := do
  match fuel with
  | 0 => throw "incremental tree exceeds structural fuel"
  | fuel + 1 =>
      let mut result := []
      let mut next := next
      let mut seen := []
      for view in desired do
        match view with
        | .text value => result := result ++ [.text value]
        | .element owner tag attrs disabled event children =>
            if seen.contains owner then throw "duplicate materialized address"
            seen := owner :: seen
            let retained := previous.find? (fun node => address node == some owner)
            let (identity, oldChildren) <- match retained with
              | some (.element _ identity oldTag _ _ _ children) =>
                  if tag != oldTag then throw "stable S3 identity changed element kind"
                  pure (identity, children)
              | _ =>
                  let identity := next
                  next := next + 1
                  pure (identity, [])
            let (updated, after) <- reconcile fuel oldChildren children next
            next := after
            result := result ++ [.element owner identity tag attrs disabled event updated]
      pure (result, next)

partial def json : Node -> Json
  | .text value => .str value
  | .element _ _ tag attrs disabled _ children =>
      let fields := [("tag", .str tag), ("attributes", Json.mkObj attrs),
        ("children", .arr (children.map json).toArray)]
      Json.mkObj (if tag == "button" then fields ++ [("disabled", .bool disabled)] else fields)

def tree (state : State) : Json := .arr (state.roots.map json).toArray

def update (fuel : Nat) (state : State) (view : List View) : Except String State := do
  let (roots, nextIdentity) <- reconcile fuel state.roots view state.nextIdentity
  let result := { roots, nextIdentity : State }
  if tree result != View.tree view then throw "incremental versus from-scratch tree drift"
  pure result

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
