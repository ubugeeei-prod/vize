import Impeto.Behavior
import Impeto.Incremental

namespace Impeto.LoopBehavior
open Lean

def snapshot (state : Incremental.State) (events : List Json) : Json :=
  Json.mkObj [("tree", Incremental.tree state), ("events", .arr events.toArray),
    ("identities", Incremental.identities state)]

def update (program : Program) (rows : List Operand) (context : Json)
    (state : Incremental.State) : Except String Incremental.State := do
  let fresh <- Observation.observe program rows context
  Incremental.update (program.regions.length + 2) state fresh.nodes

def run (program : Program) (rows : List Operand) (script : Json) : Except String Json := do
  if (<- Behavior.keys script) != ["context", "steps"] then throw "unsupported loop scenario fields"
  let mut context <- script.getObjVal? "context"
  Behavior.validateState context
  let mut state <- update program rows context {}
  let mut events := []
  let mut trace := [snapshot state events]
  for step in (<- (<- script.getObjVal? "steps").getArr?).toList do
    let fields <- Behavior.keys step
    if fields == ["patch"] then
      let patch <- step.getObjVal? "patch"
      Behavior.validateState patch
      for (name, value) in (<- patch.getObj?).toList do
        context := context.setObjVal! name value
      state <- update program rows context state
    else if fields == ["click"] then
      if let some event <- Incremental.click state (<- (<- step.getObjVal? "click").getStr?) then
        events := events ++ [event]
    else throw "unsupported loop interaction step"
    trace := trace ++ [snapshot state events]
  state := { state with roots := [] }
  pure (.arr (trace ++ [snapshot state events]).toArray)

def check (stem : String) : IO UInt32 := do
  let graph <- IO.FS.readFile s!"{stem}.s3.folio"
  let values <- IO.FS.readFile s!"{stem}.values.folio"
  let script <- IO.FS.readFile s!"{stem}.scenario.json"
  let expected <- IO.FS.readFile s!"{stem}.behavior.json"
  let result := do
    let actual <- run (<- Folio.parseProgram graph) (<- Values.parse values) (<- Json.parse script)
    if actual != (<- Json.parse expected) then throw s!"loop observation drift: {actual.compress}"
  match result with
  | .ok () => pure 0
  | .error message => IO.eprintln s!"{stem}: {message}"; pure 1

end Impeto.LoopBehavior
