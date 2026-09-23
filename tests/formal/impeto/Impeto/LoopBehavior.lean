import Impeto.Behavior
import Impeto.Incremental

namespace Impeto.LoopBehavior
open Lean

def snapshot (state : Incremental.State) (events : List Json) : Json :=
  Json.mkObj [("tree", Incremental.tree state), ("events", .arr events.toArray),
    ("identities", Incremental.identities state)]

def update (program : Program) (rows : List Operand) (context : Json)
    (state : Incremental.State) (slots : Json := .null) : Except String Incremental.State := do
  let fresh <- Observation.observe program rows context slots
  Incremental.update (program.regions.length + 2) state fresh.nodes

/-- Supplied slots as a caller passes them: static text or one displayed prop. -/
def validateSlots (slots : Json) : Except String Unit := do
  for (name, spec) in (<- slots.getObj?).toList do
    if name.isEmpty || !name.toList.all (fun c => c.isAlphanum || c == '-') then
      throw "unsupported supplied slot name"
    match (<- Behavior.keys spec) with
    | ["text"] => let _ <- (<- spec.getObjVal? "text").getStr?
    | ["prop"] =>
        if !Observation.identifier (<- (<- spec.getObjVal? "prop").getStr?) then
          throw "unsupported supplied slot prop"
    | _ => throw "supplied slots render static text or one prop"

def run (program : Program) (rows : List Operand) (script : Json) : Except String Json := do
  let fields <- Behavior.keys script
  if fields != ["context", "steps"] && fields != ["context", "slots", "steps"] then
    throw "unsupported loop scenario fields"
  let slots := (script.getObjVal? "slots").toOption.getD .null
  if slots != .null then validateSlots slots
  let mut context <- script.getObjVal? "context"
  Behavior.validateState context
  let mut state <- update program rows context {} slots
  let mut events := []
  let mut trace := [snapshot state events]
  for step in (<- (<- script.getObjVal? "steps").getArr?).toList do
    let fields <- Behavior.keys step
    if fields == ["patch"] then
      let patch <- step.getObjVal? "patch"
      Behavior.validateState patch
      for (name, value) in (<- patch.getObj?).toList do
        context := context.setObjVal! name value
      state <- update program rows context state slots
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
