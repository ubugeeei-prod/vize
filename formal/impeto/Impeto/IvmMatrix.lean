import Impeto.LoopBehavior

/-!
TS-29 IVM matrix. `ivm-matrix.cases.jsonl` holds generated templates and
scenarios; `ivm-matrix.lowered.jsonl` holds their Rust-checked graph and
value Folios. For every case this module runs the reference update machine,
whose accepted updates are proved equal to fresh rendering, and the result must
match `ivm-matrix.behavior.jsonl` exactly. Both mounted Vue runtimes are gated
on the same file. Regenerate it with `lake exe impetoRef --write-ivm-matrix`
only after a deliberate semantics change.
-/

namespace Impeto.IvmMatrix
open Lean

def jsonLines (text : String) : Except String (List Json) :=
  ((text.splitOn "\n").filter (· != "")).mapM Json.parse

def field (value : Json) (name : String) : Except String String := do
  (<- value.getObjVal? name).getStr?

def compute (runner : Program -> List Operand -> Json -> Except String Json)
    (minimum : Nat) (cases lowered : String) : Except String String := do
  let cases <- jsonLines cases
  let lowered <- jsonLines lowered
  if cases.length != lowered.length || cases.length < minimum then
    throw "matrix case and Folio files are out of step"
  let mut out := ""
  let mut names := []
  for (case, graph) in cases.zip lowered do
    let name <- field case "name"
    if names.contains name || (<- field graph "name") != name then
      throw s!"{name}: duplicate or misaligned matrix case"
    names := name :: names
    let program <- Folio.parseProgram (<- field graph "graph")
    let rows <- Values.parse (<- field graph "values")
    let trace <- match runner program rows (<- case.getObjVal? "scenario") with
      | .ok trace => pure trace
      | .error message => throw s!"{name}: {message}"
    out := out ++ (Json.mkObj [("name", .str name), ("trace", trace)]).compress ++ "\n"
  pure out

/-- Check (or, deliberately, rewrite) `fixtures/<stem>.behavior.jsonl`. -/
def runMatrix (stem : String) (runner : Program -> List Operand -> Json -> Except String Json)
    (minimum : Nat) (write : Bool) : IO UInt32 := do
  let behavior := s!"fixtures/{stem}.behavior.jsonl"
  match compute runner minimum (<- IO.FS.readFile s!"fixtures/{stem}.cases.jsonl")
      (<- IO.FS.readFile s!"fixtures/{stem}.lowered.jsonl") with
  | .error message => IO.eprintln s!"{stem}: {message}"; pure 1
  | .ok actual =>
      if write then
        IO.FS.writeFile behavior actual
        pure 0
      else if actual == (<- IO.FS.readFile behavior) then pure 0
      else
        IO.eprintln s!"{behavior}: reference observations drifted"
        pure 1

def run (write : Bool) : IO UInt32 := runMatrix "ivm-matrix" LoopBehavior.run 30 write

end Impeto.IvmMatrix
