import Impeto.ModelBehavior
import Impeto.IvmMatrix
import Impeto.BehaviorTests

/-!
Fail-closed witnesses for the native `v-model` reference. Each damages one
committed model case (scenario, state or S3 operands) so that it leaves the
modelled contract, and requires a reference error rather than a guess.
-/

namespace Impeto.ModelTests
open Lean BehaviorTests

structure Case where
  program : Program
  rows : List Operand
  scenario : Json

def load (cases lowered : String) (name : String) : Except String Case := do
  let cases <- IvmMatrix.jsonLines cases
  let lowered <- IvmMatrix.jsonLines lowered
  let some case := cases.find? (fun case => (case.getObjValAs? String "name").toOption == some name)
    | throw s!"missing model case {name}"
  let some graph := lowered.find? (fun graph => (graph.getObjValAs? String "name").toOption == some name)
    | throw s!"missing model graph {name}"
  pure {
    program := <- Folio.parseProgram (<- IvmMatrix.field graph "graph")
    rows := <- Values.parse (<- IvmMatrix.field graph "values")
    scenario := <- case.getObjVal? "scenario"
  }

def withSteps (case : Case) (steps : List String) : Except String Json := do
  pure (case.scenario.setObjVal! "steps" (.arr (<- steps.mapM Json.parse).toArray))

def withContext (case : Case) (context : String) : Except String Json := do
  pure ((case.scenario.setObjVal! "context" (<- Json.parse context)).setObjVal! "steps" (.arr #[]))

def negativeTests (cases lowered : String) : Except String Unit := do
  let text <- load cases lowered "text-ime"
  let number <- load cases lowered "text-trim-number"
  let single <- load cases lowered "select-single"
  let multiple <- load cases lowered "select-multiple"
  let toggle <- load cases lowered "checkbox-boolean"
  let run := fun (case : Case) (script : Json) => ModelBehavior.run case.program case.rows script
  for (label, case, steps) in [
    ("checked on a text input", text, "{\"event\":\"input\",\"selector\":\"input\",\"checked\":true}"),
    ("unmodelled key event", text, "{\"event\":\"keydown\",\"selector\":\"input\"}"),
    ("missing target", text, "{\"event\":\"input\",\"selector\":\"textarea\"}"),
    ("unknown step field", text, "{\"event\":\"input\",\"selector\":\"input\",\"bubbles\":false}"),
    ("fractional number", number, "{\"event\":\"input\",\"selector\":\"input\",\"value\":\"4.5\"}"),
    ("exponent number", number, "{\"event\":\"input\",\"selector\":\"input\",\"value\":\"1e3\"}"),
    ("exotic whitespace", number, "{\"event\":\"input\",\"selector\":\"input\",\"value\":\"\\u00a042\"}"),
    ("selectedValues on a single select", single,
      "{\"event\":\"change\",\"selector\":\"select\",\"selectedValues\":[\"a\"]}"),
    ("single select without a selection", single,
      "{\"event\":\"change\",\"selector\":\"select\",\"value\":\"zzz\"}"),
    ("value on a multiple select", multiple, "{\"event\":\"change\",\"selector\":\"select\",\"value\":\"a\"}")
  ] do
    expectError label (run case (<- withSteps case [steps]))
  expectError "string checkbox model" (run toggle (<- withContext toggle "{\"on\":\"yes\"}"))
  expectError "scalar multiple-select model" (run multiple (<- withContext multiple "{\"picks\":\"a\"}"))
  expectError "structured text model" (run text (<- withContext text "{\"value\":[1]}"))
  let damage := fun (case : Case) (role : String) (f : Operand -> Operand) =>
    case.rows.map (fun row => if row.role == role then f row else row)
  for (label, rows) in [
    ("divergent read and write paths", damage text "model-write" (fun row => { row with text := "other" })),
    ("compound model path", damage text "model-read" (fun row => { row with text := "value || other" })),
    ("unmodelled model attribute", damage text "model-attribute" (fun row => { row with name := some "true-value" })),
    ("model argument", damage text "name" (fun row => { row with kind := "literal", text := "arg" })),
    ("element kind mismatch", damage text "model-attribute" (fun row => { row with text := "select" }))
  ] do
    expectError label (ModelBehavior.run text.program rows text.scenario)

/-- Looped controls: index aliases are not assignable, nested loops and
identity-free observation fail closed. -/
def loopTests (cases lowered : String) : Except String Unit := do
  let keyed <- load cases lowered "keyed-text-ime"
  expectError "looped control without identities" (ModelBehavior.run keyed.program keyed.rows keyed.scenario)
  let _ <- ModelBehavior.run keyed.program keyed.rows keyed.scenario true
  let indexWrite := keyed.rows.map (fun row =>
    if ["model-read", "model-write"].contains row.role then { row with text := "position" }
    else if row.role == "for-key" then { row with kind := "js", text := "position" } else row)
  expectError "index alias assignment" (ModelBehavior.run keyed.program indexWrite keyed.scenario true)
  let step := "{\"event\":\"input\",\"selector\":\"input[data-id=zzz]\",\"value\":\"x\"}"
  expectError "missing keyed target" (ModelBehavior.run keyed.program keyed.rows (<- withSteps keyed [step]) true)

def check : IO UInt32 := do
  let cases <- IO.FS.readFile "fixtures/model-reference.cases.jsonl"
  let lowered <- IO.FS.readFile "fixtures/model-reference.lowered.jsonl"
  let loopCases <- IO.FS.readFile "fixtures/model-loop-reference.cases.jsonl"
  let loopLowered <- IO.FS.readFile "fixtures/model-loop-reference.lowered.jsonl"
  match negativeTests cases lowered *> loopTests loopCases loopLowered with
  | .ok () => pure 0
  | .error message => IO.eprintln s!"model reference: {message}"; pure 1

end Impeto.ModelTests
