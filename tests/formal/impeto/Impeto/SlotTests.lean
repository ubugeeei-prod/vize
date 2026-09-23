import Impeto.ModelTests

/-!
Fail-closed witnesses for supplied slots. A caller may supply a named slot
as static text or as one displayed slot prop. Any other supplied shape, name
or missing prop must be rejected instead of approximated.
-/

namespace Impeto.SlotTests
open Lean BehaviorTests ModelTests

def negativeTests (cases lowered : String) : Except String Unit := do
  let named <- load cases lowered "named-prop"
  for slots in [
    "{\"header\":{\"text\":1}}", "{\"header\":{\"prop\":\"missing\"}}",
    "{\"header\":{\"text\":\"x\",\"prop\":\"item\"}}", "{\"header\":{\"html\":\"<b>\"}}",
    "{\"header\":{\"prop\":\"item.label\"}}", "{\"bad name\":{\"text\":\"x\"}}", "[]"
  ] do
    let script := named.scenario.setObjVal! "slots" (<- Json.parse slots)
    expectError s!"supplied slots {slots}" (LoopBehavior.run named.program named.rows script)
  let dynamic := named.rows.map (fun row =>
    if row.role == "name" && row.target.isNone then { row with kind := "js" } else row)
  expectError "dynamic slot name" (LoopBehavior.run named.program dynamic named.scenario)

def check : IO UInt32 := do
  let cases <- IO.FS.readFile "fixtures/slot-reference.cases.jsonl"
  let lowered <- IO.FS.readFile "fixtures/slot-reference.lowered.jsonl"
  match negativeTests cases lowered with
  | .ok () => pure 0
  | .error message => IO.eprintln s!"slot reference: {message}"; pure 1

end Impeto.SlotTests
