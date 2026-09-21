import Impeto.BehaviorTests

namespace Impeto.ControlTests
open Lean BehaviorTests

def expectValue {α : Type} [BEq α] (label : String) (expected : α)
    (result : Except String α) : Except String Unit := do
  if (<- result) != expected then throw s!"incorrect control observation: {label}"

def referenceTests (program : Program) (rows : List Operand) (script : Json) : Except String Unit := do
  let _ <- Behavior.run program rows script
  let mutate := fun (id : Nat) (role : String) (f : Operand -> Operand) =>
    rows.map (fun row => if row.op == id && row.role == role then f row else row)
  let condition <- Values.one rows 1 "condition"
  let slot <- Values.one rows 4 "name"
  let value <- Values.one rows 6 "value"
  let hidden := Json.mkObj [("context", Json.mkObj [("ready", .bool false),
    ("fallback", .str "fallback")]), ("steps", .arr #[])]
  let _ <- Behavior.run program rows hidden
  let text <- Values.one rows 3 "text"
  let tag <- Values.one rows 2 "tag"
  let namespaceRow <- Values.one rows 2 "namespace"
  for (label, damaged) in [
    ("inactive branch missing text", rows.filter (fun row => row.op != 3)),
    ("inactive branch duplicate text", rows ++ [text]),
    ("inactive branch targeted text", mutate 3 "text" (fun row => { row with target := some 2 })),
    ("inactive branch missing tag", rows.filter (fun row => row.op != 2 || row.role != "tag")),
    ("inactive branch duplicate tag", rows ++ [tag]),
    ("inactive branch missing namespace", rows.filter (fun row => row.op != 2 || row.role != "namespace")),
    ("inactive branch duplicate namespace", rows ++ [namespaceRow])
  ] do
    expectError label (Behavior.run program damaged hidden)
  for (label, damaged) in [
    ("missing condition", rows.filter (fun row => row.role != "condition")),
    ("duplicate condition", rows ++ [condition]),
    ("foreign branch region", mutate 1 "condition" (fun row => { row with region := some 4 })),
    ("missing branch region", mutate 1 "condition" (fun row => { row with region := some 99 })),
    ("loose-equality condition", mutate 1 "condition" (fun row => { row with text := "ready == other" })),
    ("literal condition", mutate 1 "condition" (fun row => { row with kind := "literal" })),
    ("sole else branch", mutate 1 "condition" (fun row => { row with kind := "absent", text := "" })),
    ("missing slot name", rows.filter (fun row => row.op != 4)),
    ("duplicate slot name", rows ++ [slot]),
    ("dynamic slot", mutate 4 "name" (fun row => { row with kind := "js" })),
    ("empty slot name", mutate 4 "name" (fun row => { row with text := "" })),
    ("slot props", rows ++ [{ slot with role := "attribute", name := some "title" }]),
    ("missing text value", rows.filter (fun row => row.op != 6 || row.role != "value")),
    ("duplicate text value", rows ++ [value]),
    ("wrong text target", mutate 6 "value" (fun row => { row with target := some 2 })),
    ("non-element text target", mutate 6 "value" (fun row => { row with target := some 4 })),
    ("unsupported text binding", mutate 6 "binding-kind" (fun row => { row with text := "vue.html" })),
    ("untargeted text binding", rows.filter (fun row => row.op != 6 || row.role != "binding-kind"))
  ] do
    expectError label (Behavior.run program damaged script)
  for (label, damaged) in [
    ("wrong branch owner", { program with regions := (program.regions.map
      (fun r => if r.id == 2 then { r with owner := some 3 } else r)) }),
    ("wrong branch parent", { program with regions := (program.regions.map
      (fun r => if r.id == 2 then { r with parent := some 0 } else r)) }),
    ("missing fallback region", { program with regions := program.regions.filter (fun r => r.id != 4) }),
    ("extra branch region", { program with regions := program.regions ++
      [{ id := 6, parent := some 1, owner := some 1, span := condition.span }] }),
    ("extra fallback region", { program with regions := program.regions ++
      [{ id := 6, parent := some 1, owner := some 4, span := slot.span }] })
  ] do
    expectError label (Behavior.run damaged rows script)
  -- Conditions follow JavaScript truthiness, exactly like both mounted runtimes.
  for (truthyValue, same) in [("\"false\"", "true"), ("null", "false"), ("1", "true"), ("\"\"", "false")] do
    let run := fun (ready : String) => do
      Behavior.run program rows (script.setObjVal! "context"
        (<- Json.parse s!"\{\"ready\":{ready},\"fallback\":\"x\"}"))
    if (<- run truthyValue) != (<- run same) then
      throw s!"condition {truthyValue} must render like {same}"
  for context in [
    "{\"ready\":true,\"fallback\":{}}",
    "{\"ready\":true,\"fallback\":[]}",
    "{\"ready\":true,\"fallback\":1.5}",
    "{\"ready\":true,\"fallback\":\"x\",\"$slots\":{}}"
  ] do
    expectError "unsupported control context" (Behavior.run program rows
      (script.setObjVal! "context" (<- Json.parse context)))
  let textOp : Op := { id := 99, kind := .setText, region := 5, effect := none, span := value.span }
  let textRow := { value with op := 99, role := "text", target := none }
  expectError "v-text with children" (Behavior.run { program with ops := program.ops ++ [textOp] }
    (rows ++ [textRow]) script)
  let some original := program.ops.find? (fun op => op.id == 6) | throw "missing text binding fixture"
  let duplicate := { original with id := 99 }
  let duplicateRows := (Values.forOp rows 6).map (fun row => { row with op := 99 })
  expectError "duplicate v-text binding" (Behavior.run { program with ops := program.ops ++ [duplicate] }
    (rows ++ duplicateRows) script)

def inactiveValueTests (program : Program) (rows : List Operand) : Except String Unit := do
  let script := Json.mkObj [("context", Json.mkObj [("ready", .bool false),
    ("fallback", .str "fallback")]), ("steps", .arr #[])]
  let tag <- Values.one rows 2 "tag"
  let attr := { tag with role := "attribute", name := some "title", text := "hidden" }
  let mutate := fun (id : Nat) (role : String) (f : Operand -> Operand) =>
    rows.map (fun row => if row.op == id && row.role == role then f row else row)
  let _ <- Behavior.run program (rows ++ [attr]) script
  for (label, damaged) in [
    ("inactive dynamic tag", mutate 2 "tag" (fun row => { row with kind := "js" })),
    ("inactive unsupported tag", mutate 2 "tag" (fun row => { row with text := "script" })),
    ("inactive dynamic namespace", mutate 2 "namespace" (fun row => { row with kind := "js" })),
    ("inactive unsupported namespace", mutate 2 "namespace" (fun row => { row with text := "svg" })),
    ("inactive dynamic attribute", rows ++ [{ attr with kind := "js" }]),
    ("inactive unnamed attribute", rows ++ [{ attr with name := none }]),
    ("inactive unsupported attribute", rows ++ [{ attr with name := some "onclick" }]),
    ("inactive duplicate attribute", rows ++ [attr, attr]),
    ("inactive unsupported text", mutate 3 "text" (fun row => { row with kind := "opaque" })),
    ("inactive compound text", mutate 3 "text" (fun row => { row with kind := "js", text := "missing()" }))
  ] do
    expectError label (Behavior.run program damaged script)
  let _ <- Behavior.run program
    (mutate 3 "text" (fun row => { row with kind := "js", text := "missing" })) script
  let some element := program.ops.find? (fun op => op.id == 2) | throw "missing branch element"
  let binding := { element with id := 99, kind := OpKind.setProp }
  let name := { tag with op := 99, role := "name", target := some 2, text := "disabled" }
  let value := { name with role := "value", kind := "js", text := "missing" }
  let kind := { name with role := "binding-kind", text := "bind" }
  let buttonRows := mutate 2 "tag" (fun row => { row with text := "button" })
  let _ <- Behavior.run { program with ops := program.ops ++ [binding] }
    (buttonRows ++ [name, value, kind]) script
  expectError "inactive property on non-button" (Behavior.run
    { program with ops := program.ops ++ [binding] } (rows ++ [name, value, kind]) script)
  let eventRows := [{ name with text := "click" }, { value with text := "save" },
    { kind with text := "on" }]
  for (label, ops, operands) in [
    ("inactive unsupported property", [binding], [{ name with text := "style" }, value, kind]),
    ("inactive unsupported prop expression", [binding], [name, { value with text := "missing()" }, kind]),
    ("inactive literal property", [binding], [name, { value with kind := "literal" }, kind]),
    ("inactive duplicate properties", [binding, { binding with id := 100 }],
      [name, value, kind] ++ [name, value, kind].map (fun row => { row with op := 100 })),
    ("inactive duplicate events", [{ binding with kind := .setEvent }, { binding with id := 100, kind := .setEvent }],
      eventRows ++ eventRows.map (fun row => { row with op := 100 })),
    ("inactive unsupported event", [{ binding with kind := .setEvent }],
      [{ name with text := "focus" }, { value with text := "save" }, { kind with text := "on" }]),
    ("inactive unsupported handler", [{ binding with kind := .setEvent }],
      [{ name with text := "click" }, value, { kind with text := "on" }]),
    ("inactive v-text with children", [{ binding with kind := .setText }],
      [value, { kind with text := "vue.text" }])
  ] do
    expectError label (Behavior.run { program with ops := program.ops ++ ops }
      (buttonRows ++ operands) script)
  let textBinding := { binding with kind := OpKind.setText }
  let textRows := [value, { kind with text := "vue.text" }]
  let withoutText := { program with ops := program.ops.filter (fun op => op.id != 3) }
  let withoutTextRows := rows.filter (fun row => row.op != 3)
  let _ <- Behavior.run { withoutText with ops := withoutText.ops ++ [textBinding] }
    (withoutTextRows ++ textRows) script
  expectError "inactive duplicate v-text" (Behavior.run
    { withoutText with ops := withoutText.ops ++ [textBinding, { textBinding with id := 100 }] }
    (withoutTextRows ++ textRows ++ textRows.map (fun row => { row with op := 100 })) script)

def branchTests (program : Program) (rows : List Operand) : Except String Unit := do
  let first <- Values.one rows 1 "condition"
  let second := { first with region := some 6, text := "other" }
  let fallback := { first with region := some 7, kind := "absent", text := "" }
  let branches := [first, second, fallback]
  let program := { program with regions := program.regions ++ [
    { id := 6, parent := some 1, owner := some 1, span := first.span },
    { id := 7, parent := some 1, owner := some 1, span := first.span }
  ] }
  let rows := rows.filter (fun row => row.op != 1) ++ branches
  Observation.validate program rows
  for (context, expected) in [
    ("{\"ready\":true}", some 2),
    ("{\"ready\":true,\"other\":true}", some 2),
    ("{\"ready\":false,\"other\":true}", some 6),
    ("{\"ready\":false,\"other\":false}", some 7)
  ] do
    expectValue "first matching branch" expected
      (Observation.selectedBranch rows (<- Json.parse context) 1)
  expectValue "no selected branch" none
    (Observation.selectedBranch [first] (Json.mkObj [("ready", .bool false)]) 1)
  for conditions in [[fallback, second, first], [first, fallback, second], [first, second, second]] do
    expectError "misordered or duplicate branches"
      (Observation.validate program (rows.filter (fun row => row.op != 1) ++ conditions))
  for conditions in [
    [{ first with kind := "absent", text := "" }, second, fallback],
    [first, { second with kind := "absent", text := "" }, fallback]
  ] do
    expectError "else before final region"
      (Observation.validate program (rows.filter (fun row => row.op != 1) ++ conditions))
  for (context, expected) in [
    ("{\"ready\":0,\"other\":\"false\"}", some 6),
    ("{\"ready\":\"\",\"other\":[]}", some 6),
    ("{\"ready\":null,\"other\":0}", some 7),
    ("{\"ready\":{},\"other\":0}", some 2)
  ] do
    expectValue "JavaScript truthiness" expected
      (Observation.selectedBranch branches (<- Json.parse context) 1)
  expectError "unsupported reached condition"
    (Observation.selectedBranch branches (<- Json.parse "{\"ready\":false}") 1)

def activationTests (program : Program) (rows : List Operand) : Except String Unit := do
  let context := Json.mkObj [("ready", .bool true), ("fallback", .str "fallback")]
  let hidden := context.setObjVal! "ready" (.bool false)
  let rows := rows.map (fun row => if row.op == 2 && row.role == "tag" then
    { row with text := "button" } else row)
  let handler : Op := { id := 99, kind := .setEvent, region := 2, effect := none, span := { start := 9, stop := 34 } }
  let name : Operand := {
    op := 99, role := "name", target := some 2, region := none,
    name := none, kind := "literal", text := "click", qualifier := "", span := handler.span
  }
  let program := { program with ops := program.ops ++ [handler] }
  let rows := rows ++ [name, { name with role := "value", kind := "js", text := "save" },
    { name with role := "binding-kind", text := "on" }]
  expectValue "visible branch button" (false, true) (Behavior.buttonState program rows context)
  expectError "hidden branch button" (Behavior.buttonState program rows hidden)
  let script := Json.mkObj [("context", context),
    ("steps", .arr #[Json.mkObj [("activate", .str "button")]])]
  let trace <- Behavior.run program rows script
  let snapshot <- trace.getArrVal? 1
  expectValue "visible branch delivery" (.arr #[.str "save"]) (snapshot.getObjVal? "events")
  let twoButtons := rows.map (fun row => if row.op == 5 && row.role == "tag" then
    { row with text := "button" } else row)
  expectError "ambiguous active buttons" (Behavior.buttonState program twoButtons context)
  expectValue "only fallback button is active" (false, false)
    (Behavior.buttonState program twoButtons hidden)

def check : IO UInt32 := do
  let stem := "fixtures/rust-lowered-control-slots"
  let graph <- IO.FS.readFile s!"{stem}.s3.folio"
  let values <- IO.FS.readFile s!"{stem}.values.folio"
  let script <- IO.FS.readFile s!"{stem}.scenario.json"
  let result := do
    let program <- Folio.parseProgram graph
    let rows <- Values.parse values
    referenceTests program rows (<- Json.parse script)
    inactiveValueTests program rows
    branchTests program rows
    activationTests program rows
  match result with
  | .ok () => pure 0
  | .error message =>
      IO.eprintln message
      pure 1

end Impeto.ControlTests
