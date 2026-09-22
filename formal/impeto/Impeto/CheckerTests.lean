import Impeto.Checker
import Impeto.IvmMatrix

/-!
Runs the C-23 checker over every committed S3 graph: the hand-written and
Rust-lowered ladder Folios, the TS-29 matrix, and the model and slot
references. Every graph must be well formed. Seven targeted damages of a
Rust-lowered graph must each produce exactly the expected invariant codes.
-/

namespace Impeto.CheckerTests
open Lean Checker

def graphFiles : List String := [
  "static-text", "dynamic-button", "control-flow", "rust-lowered-static-dynamic",
  "rust-lowered-control-slots", "rust-lowered-loop-keyed", "rust-lowered-loop-unkeyed",
  "rust-lowered-loop-nested"
]

def loweredFiles : List String := ["ivm-matrix", "model-reference", "slot-reference"]

def damages (p : Program) : Except String (List (String × Program × List String)) := do
  let some last := p.ops.getLast? | throw "empty program"
  let some child := p.regions.find? (·.id != 0) | throw "no child region"
  let some grand := p.regions.find? (fun r => r.parent == some child.id) | throw "no nested region"
  let region := fun (f : Region -> Region) => p.regions.map fun r =>
    if r.id == child.id then f r else if r.id == grand.id then { r with parent := some child.id } else r
  let dangling : StateEdge := { source := last.id, target := 999, kind := .dataDependency, effect := none }
  pure [
    ("duplicate op id", { p with ops := p.ops ++ [last] }, ["S3V001"]),
    ("op span escapes its region", { p with ops := p.ops.map fun o =>
      if o.id == last.id then { o with span := { o.span with stop := o.span.stop + 10000 } } else o },
      ["S3V006"]),
    ("op in a missing region", { p with ops := p.ops.map fun o =>
      if o.id == last.id then { o with region := 999 } else o }, ["S3V003", "S3V007"]),
    ("op names a missing effect", { p with ops := p.ops.map fun o =>
      if o.id == last.id then { o with effect := some 999 } else o }, ["S3V007"]),
    ("dangling edge target", { p with edges := p.edges ++ [dangling] }, ["S3V004"]),
    ("parent cycle", { p with regions := region fun r => { r with parent := some grand.id } },
      ["S3V005", "S3V006"]),
    ("backward scheduled edges", { p with phase := .scheduled, ops := p.ops.reverse }, ["S3V008"])
  ]

def run : IO (Except String Nat) := do
  let mut programs := []
  for stem in graphFiles do
    programs := programs ++ [(stem, <- IO.FS.readFile s!"fixtures/{stem}.s3.folio")]
  for stem in loweredFiles do
    match IvmMatrix.jsonLines (<- IO.FS.readFile s!"fixtures/{stem}.lowered.jsonl") with
    | .error message => return .error message
    | .ok rows =>
        for row in rows do
          match IvmMatrix.field row "name", IvmMatrix.field row "graph" with
          | .ok name, .ok graph => programs := programs ++ [(s!"{stem}/{name}", graph)]
          | _, _ => return .error s!"{stem}: malformed lowered row"
  pure do
    let mut checked := 0
    for (name, text) in programs do
      let program <- Folio.parseProgram text
      let codes := violations program
      if !codes.isEmpty then throw s!"{name}: {codes}"
      checked := checked + 1
    let nested <- Folio.parseProgram (programs.lookup "rust-lowered-loop-nested" |>.getD "")
    for (label, damaged, expected) in <- damages nested do
      let actual := violations damaged
      if actual != expected then throw s!"{label}: expected {expected}, got {actual}"
    if checked < 50 then throw "folio checker corpus is too small"
    pure checked

def check : IO UInt32 := do
  match (<- run) with
  | .ok count =>
      IO.println s!"folio checker: {count} committed S3 graphs are well formed"
      pure 0
  | .error message => IO.eprintln s!"folio checker: {message}"; pure 1

end Impeto.CheckerTests
