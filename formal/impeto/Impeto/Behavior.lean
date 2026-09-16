import Impeto.Observation

namespace Impeto.Behavior
open Lean

def keys (value : Json) : Except String (List String) := do
  pure ((<- value.getObj?).toList.map (·.1))

def validateState (context : Json) : Except String Unit := do
  for name in (<- keys context) do
    if !Observation.identifier name ||
        ["save", "$slots", "__proto__", "constructor", "prototype"].contains name then
      throw "unsupported state key"

def snapshot (tree : Json) (events : List String) : Json :=
  Json.mkObj [("tree", tree), ("events", .arr (events.map Json.str).toArray)]

def buttonState (program : Program) (rows : List Operand) (context : Json) : Except String (Bool × Bool) := do
  let buttons := program.ops.filter (fun op => op.kind == .insertNode &&
    (Values.one rows op.id "tag").toOption.any (fun row => row.text == "button"))
  let [button] := buttons | throw "expected one interaction target"
  let bindings := Observation.attached program rows button.id
  let handlers := bindings.filter (fun op => op.kind == .setEvent)
  if handlers.length > 1 then throw "unsupported multiple click handlers"
  let mut disabled := false
  for op in bindings do
    if op.kind == .setProp then
      disabled <- (<- Observation.evaluate context (<- Values.one rows op.id "value")).getBool?
  pure (disabled, !handlers.isEmpty)

def run (program : Program) (rows : List Operand) (script : Json) : Except String Json := do
  if (<- keys script) != ["context", "steps"] then throw "unsupported scenario fields"
  let mut context <- script.getObjVal? "context"
  validateState context
  let steps <- (<- script.getObjVal? "steps").getArr?
  let mut events := []
  let mut trace := [snapshot (<- Observation.render program rows context) events]
  for step in steps do
    let fields <- keys step
    if fields == ["patch"] then
      validateState (<- step.getObjVal? "patch")
      let patch <- (<- step.getObjVal? "patch").getObj?
      for (name, value) in patch.toList do
        context := context.setObjVal! name value
    else if fields == ["activate"] then
      if (<- step.getObjVal? "activate") != .str "button" then
        throw "unsupported activation target"
      let (disabled, hasHandler) <- buttonState program rows context
      if !disabled && hasHandler then
        events := events ++ ["save"]
    else if fields == ["event", "selector"] then
      if (<- step.getObjVal? "event") != .str "click" ||
          (<- step.getObjVal? "selector") != .str "button" then
        throw "unsupported interaction"
      let (disabled, hasHandler) <- buttonState program rows context
      if disabled then
        throw "disabled synthetic dispatch is outside this reference subset"
      if hasHandler then events := events ++ ["save"]
    else throw "unsupported interaction step"
    trace := trace ++ [snapshot (<- Observation.render program rows context) events]
  pure (.arr (trace ++ [snapshot (.arr #[]) events]).toArray)

def readInputs (stem : String) : IO (Except String Json) := do
  let graph <- IO.FS.readFile s!"{stem}.s3.folio"
  let values <- IO.FS.readFile s!"{stem}.values.folio"
  let script <- IO.FS.readFile s!"{stem}.scenario.json"
  pure do run (<- Folio.parseProgram graph) (<- Values.parse values) (<- Json.parse script)

def check (stem : String) : IO UInt32 := do
  let expected <- IO.FS.readFile s!"{stem}.behavior.json"
  match (<- readInputs stem), Json.parse expected with
  | .ok actual, .ok expected =>
      if actual == expected then pure 0
      else
        IO.eprintln s!"{stem}: stateful observation drift\nactual: {actual.compress}"
        pure 1
  | .error message, _ | _, .error message =>
      IO.eprintln s!"{stem}: {message}"
      pure 2

end Impeto.Behavior
