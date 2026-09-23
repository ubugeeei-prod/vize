import Impeto.LoopBehavior
import Impeto.BehaviorTests

namespace Impeto.LoopTests
open Lean BehaviorTests

def negativeTests (program : Program) (rows : List Operand) (script : Json) : Except String Unit := do
  let some loopOp := program.ops.find? (fun op => op.kind == .loop) | throw "missing loop"
  let source <- Values.one rows loopOp.id "for-source"
  let value <- Values.one rows loopOp.id "for-value"
  let index <- Values.one rows loopOp.id "for-key"
  let some key := rows.find? (fun row => row.role == "name" && row.text == "key")
    | throw "missing identity binding"
  let change := fun (id : Nat) (role : String) (f : Operand -> Operand) =>
    rows.map (fun row => if row.op == id && row.role == role then f row else row)
  let empty := script.setObjVal! "context"
    (Json.mkObj [("items", .arr #[]), ("item", .str "outer"), ("position", .str "outer")])
  let empty := empty.setObjVal! "steps" (.arr #[])
  for (label, damaged) in [
    ("missing source", rows.filter (fun row => row != source)),
    ("duplicate source", rows ++ [source]),
    ("missing value alias", rows.filter (fun row => row != value)),
    ("duplicate value alias", rows ++ [value]),
    ("same aliases", change loopOp.id "for-key" (fun row => { row with text := value.text })),
    ("destructured alias", change loopOp.id "for-value" (fun row => { row with text := "{id}" })),
    ("reserved alias", change loopOp.id "for-value" (fun row => { row with text := "record" })),
    ("event alias", change loopOp.id "for-value" (fun row => { row with text := "$event" })),
    ("missing index slot", rows.filter (fun row => row != index)),
    ("third array alias", change loopOp.id "for-index" (fun row => { row with kind := "js", text := "third" })),
    ("compound source", change loopOp.id "for-source" (fun row => { row with text := "items || []" })),
    ("prototype source", change loopOp.id "for-source" (fun row => { row with text := "items.__proto__" })),
    ("dynamic key name", change key.op "name" (fun row => { row with kind := "js" })),
    ("unsupported inactive expression", change key.op "value" (fun row => { row with text := "item.id()" }))
  ] do
    expectError label (LoopBehavior.run program damaged empty)
  for bad in ["null", "{}", "true", "1.5", "2147483648"] do
    let item := Json.mkObj [("id", <- Json.parse bad), ("label", .str "bad")]
    let context := (<- script.getObjVal? "context").setObjVal! "items" (.arr #[item])
    expectError "unsupported key" (LoopBehavior.run program rows
      (script.setObjVal! "context" context |>.setObjVal! "steps" (.arr #[])))
  let item := Json.mkObj [("id", .str "same"), ("label", .str "duplicate")]
  let context <- script.getObjVal? "context"
  let typedItems := [Json.num 1, Json.str "1"].map (fun id =>
    Json.mkObj [("id", id), ("label", .str "typed")])
  let scopes <- Iteration.contexts program rows loopOp (context.setObjVal! "items" (.arr typedItems.toArray))
  if (scopes.map (·.1)).eraseDups.length != 2 then throw "string and numeric keys collided"
  expectError "duplicate key" (LoopBehavior.run program rows (script
    |>.setObjVal! "context" (context.setObjVal! "items" (.arr #[item, item]))
    |>.setObjVal! "steps" (.arr #[])))
  for source in [.null, .bool false, .str "abc", .num (-1), 1001,
      Json.mkObj [("0", .str "index-like")]] do
    expectError "non-array source" (LoopBehavior.run program rows (script
      |>.setObjVal! "context" (context.setObjVal! "items" source)
      |>.setObjVal! "steps" (.arr #[])))
  for step in ["{}", "{\"click\":\"missing\"}", "{\"click\":1}",
    "{\"click\":\"a\",\"patch\":{}}", "{\"patch\":{\"record\":null}}", "{\"patch\":{\"$event\":{}}}"] do
    expectError "invalid interaction" (LoopBehavior.run program rows
      (script.setObjVal! "steps" (.arr #[<- Json.parse step])))
  let some body := program.regions.find? (fun r => r.owner == some loopOp.id)
    | throw "missing body"
  expectError "missing loop body" (LoopBehavior.run
    { program with regions := program.regions.filter (fun r => r.id != body.id) } rows empty)
  expectError "multiple loop bodies" (LoopBehavior.run
    { program with regions := program.regions ++ [{ body with id := 999 }] } rows empty)
  let some root := program.ops.find? (fun op => op.region == body.id && op.kind == .insertNode)
    | throw "missing loop root"
  let duplicateTitle := { source with
    op := root.id
    role := "attribute"
    kind := "literal"
    text := "static"
    name := some "title"
    target := none
    span := root.span }
  expectError "static and dynamic property collision" (LoopBehavior.run program (rows ++ [duplicateTitle]) empty)
  let some outer := program.ops.find? (fun op => op.region == 0 && op.kind == .insertNode)
    | throw "missing outer root"
  let outsideKey := rows.map (fun row => if row.op == key.op then
    { row with target := some outer.id } else row)
  let outsideProgram := { program with ops := program.ops.map (fun op =>
    if op.id == key.op then { op with region := 0 } else op) }
  expectError "key outside loop root" (LoopBehavior.run outsideProgram outsideKey empty)

-- Enumerate every ordered subset of three keys. This covers all insertion,
-- deletion, and permutation pairs, including empty lists, without borrowing
-- either runtime's update algorithm or any expected output it generated.
def arrangements : List (List String) :=
  [[], ["a"], ["b"], ["c"], ["a", "b"], ["b", "a"], ["a", "c"], ["c", "a"],
    ["b", "c"], ["c", "b"], ["a", "b", "c"], ["a", "c", "b"],
    ["b", "a", "c"], ["b", "c", "a"], ["c", "a", "b"], ["c", "b", "a"]]

def views (keyed : Bool) (keys : List String) (suffix : String) : List View :=
  keys.zipIdx.map fun (key, index) =>
    .element (if keyed then key else toString index) "button" [("data-id", .str key)]
      false (some (.str (key ++ suffix))) [.text (key ++ suffix)]

def reconciliationTests : Except String Unit := do
  for keyed in [true, false] do
    for before in arrangements do
      for after in arrangements do
        let state <- Incremental.update 4 {} (views keyed before "old")
        let updated <- Incremental.update 4 state (views keyed after "new")
        let actual := Incremental.allTargets updated
        let mut next := before.length
        for (key, index) in after.zipIdx do
          let retained := if keyed then (before.zipIdx.find? (fun pair => pair.1 == key)).map (·.2)
            else if index < before.length then some index else none
          let expected <- match retained with
            | some identity => pure identity
            | none => let identity := next; next := next + 1; pure identity
          let some target := actual[index]? | throw "missing reconciled target"
          if target.name != key || target.identity != expected || target.event != some (.str (key ++ "new")) then
            throw "incorrect retained identity or stale event payload"
        if actual.length != after.length || updated.nextIdentity != next then
          throw "incorrect allocation or removal count"
        let cleared <- Incremental.update 4 updated []
        let reinserted <- Incremental.update 4 cleared (views keyed ["a"] "again")
        if (Incremental.allTargets reinserted).map (·.identity) != [next] then
          throw "retired identity was resurrected"
  let duplicate := views true ["a", "a"] ""
  expectError "duplicate materialized address" (Incremental.update 4 {} duplicate)

def check : IO UInt32 := do
  for name in ["keyed", "unkeyed", "nested"] do
    let code <- LoopBehavior.check s!"fixtures/rust-lowered-loop-{name}"
    if code != 0 then return code
  let graph <- IO.FS.readFile "fixtures/rust-lowered-loop-keyed.s3.folio"
  let values <- IO.FS.readFile "fixtures/rust-lowered-loop-keyed.values.folio"
  let script <- IO.FS.readFile "fixtures/rust-lowered-loop-keyed.scenario.json"
  let result := do
    reconciliationTests
    negativeTests (<- Folio.parseProgram graph) (<- Values.parse values) (<- Json.parse script)
  match result with
  | .ok () => pure 0
  | .error message => IO.eprintln message; pure 1

end Impeto.LoopTests
