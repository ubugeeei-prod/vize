import Impeto.Behavior

namespace Impeto.BehaviorTests
open Lean

def expectError {α : Type} (label : String) (result : Except String α) : Except String Unit :=
  match result with
  | .error _ => pure ()
  | .ok _ => throw s!"negative witness accepted: {label}"

def parseRow (fields : Array Json) : Except String Operand :=
  Values.parseOperand ("operand=" ++ (Json.arr fields).compress)

def parserTests : Except String Unit := do
  for role in Values.roles do
    for kind in ["absent", "literal", "js", "opaque", "foreign", "vue.filter"] do
      let target : Json := if ["binding-kind", "value", "modifier", "model-read",
        "model-write", "model-attribute", "params"].contains role then 1 else .null
      let region : Json := if role == "condition" then 1 else .null
      let name : Json := if ["attribute", "model-attribute"].contains role then .str "class" else .null
      let text := if kind == "absent" then "" else "quoted\"\\\n\t" ++ String.singleton (Char.ofNat 0x96ea)
      let qualifier := if ["opaque", "foreign"].contains kind then "dialect" else ""
      let row <- parseRow #[0, .str role, target, region, name, .str kind, .str text, .str qualifier, 0, 20]
      if row.role != role || row.kind != kind || row.text != text || row.qualifier != qualifier then
        throw "S3 operand parsing lost authored payload"
  let valid : Array Json := #[0, .str "text", .null, .null, .null, .str "literal", .str "hello", .str "", 0, 5]
  for (index, value) in [
    (0, .num (-1)), (0, 4294967296), (1, .str "unknown"), (2, 1), (3, 1),
    (4, .str "extra"), (5, .str "unknown"), (5, .str "absent"), (5, .str "opaque"),
    (5, .str "foreign"), (7, .str "extra"), (8, 6)
  ] do
    expectError s!"malformed field {index}" (parseRow (valid.set! index value))
  expectError "short tuple" (parseRow #[])
  expectError "long tuple" (parseRow (valid.push 0))
  for role in ["value", "binding-kind", "model-attribute", "condition", "attribute"] do
    expectError "missing reference" (parseRow (valid.set! 1 (.str role)))
  for text in ["", "operand=[]", "[s3-values-folio]\n", "[s3-values-folio]\n[unknown]"] do
    expectError "malformed value page" (Values.parse text)

def referenceTests (program : Program) (rows : List Operand) (script : Json) : Except String Unit := do
  let _ <- Behavior.run program rows script
  for kind in ["opaque", "foreign", "vue.filter", "absent"] do
    expectError "unsupported expression kind" (Behavior.run program
      (rows.map (fun row => if row.role == "text" then { row with kind } else row)) script)
  expectError "compound expression" (Behavior.run program
    (rows.map (fun row => if row.role == "text" then { row with text := "label + '!'" } else row)) script)
  expectError "missing text" (Behavior.run program (rows.filter (fun row => row.role != "text")) script)
  expectError "duplicate text" (Behavior.run program (rows ++ rows.filter (fun row => row.role == "text")) script)
  expectError "dangling operand" (Behavior.run program (rows.map (fun row => { row with op := 999 })) script)
  expectError "unsupported op" (Behavior.run
    { program with ops := program.ops.map (fun op => if op.id == 4 then { op with kind := .loop } else op) }
    rows script)
  expectError "cyclic region" (Behavior.run
    { program with regions := program.regions.map (fun r => if r.id == 1 then { r with parent := some 2 } else r) }
    rows script)
  expectError "unsupported phase" (Behavior.run { program with phase := .scheduled } rows script)
  expectError "wrong binding target" (Behavior.run program
    (rows.map (fun row => if row.role == "value" then { row with target := some 999 } else row)) script)
  expectError "unsupported handler" (Behavior.run program
    (rows.map (fun row => if row.op == 3 && row.role == "value" then { row with text := "other" } else row)) script)
  for text in [
    "{\"context\":{},\"steps\":[]}",
    "{\"context\":{\"locked\":true,\"label\":{}},\"steps\":[]}",
    "{\"context\":{\"locked\":\"false\",\"label\":\"x\"},\"steps\":[]}",
    "{\"context\":{\"locked\":false,\"label\":1.5},\"steps\":[]}",
    "{\"context\":{\"locked\":false,\"label\":2147483648},\"steps\":[]}",
    "{\"context\":{\"locked\":false,\"label\":\"x\",\"save\":null},\"steps\":[]}"
  ] do
    expectError "unsupported state" (Behavior.run program rows (<- Json.parse text))
  for text in [
    "{}", "{\"patch\":{},\"ignored\":true}", "{\"patch\":{\"save\":null}}",
    "{\"activate\":\"missing\"}", "{\"activate\":false}",
    "{\"activate\":\"button\",\"event\":\"click\"}",
    "{\"activate\":\"button\",\"patch\":{}}",
    "{\"event\":\"change\",\"selector\":\"button\"}",
    "{\"event\":\"click\",\"selector\":\"missing\"}",
    "{\"event\":\"click\",\"selector\":\"button\"}"
  ] do
    -- Activation is supported when locked; synthetic dispatch remains excluded.
    expectError "unsupported step" (Behavior.run program rows
      (script.setObjVal! "steps" (.arr #[<- Json.parse text])))

def check : IO UInt32 := do
  let stem := "fixtures/rust-lowered-static-dynamic"
  let graph <- IO.FS.readFile s!"{stem}.s3.folio"
  let values <- IO.FS.readFile s!"{stem}.values.folio"
  let script <- IO.FS.readFile s!"{stem}.scenario.json"
  let result := do
    parserTests
    referenceTests (<- Folio.parseProgram graph) (<- Values.parse values) (<- Json.parse script)
  match result with
  | .ok () => pure 0
  | .error message =>
      IO.eprintln message
      pure 1

end Impeto.BehaviorTests
