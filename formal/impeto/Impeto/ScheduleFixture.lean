import Impeto.Folio
import Impeto.Schedule

/-!
Differential for the scheduling contract. Rust lowers each reference
template, applies a named ordering scenario and records the TS-27 validator's
exact codes in `schedule-contract.txt`. This module applies the same scenarios
to the committed graph Folios and recomputes the codes with `orderCodes`.
Accepted scheduled programs are also executed to observe the ordering that
`accepted_schedule_orders_edges` proves.
-/

namespace Impeto.ScheduleFixture
open Schedule

def stems : List String := [
  "rust-lowered-static-dynamic", "rust-lowered-control-slots", "rust-lowered-loop-keyed",
  "rust-lowered-loop-unkeyed", "rust-lowered-loop-nested"
]

def scenarios : List String := [
  "built", "scheduled", "scheduled-reversed-ops", "scheduled-reversed-first-edge",
  "scheduled-self-edge", "dangling-target", "duplicate-last-op", "foreign-scope-edge",
  "missing-scope-edge"
]

def edge (source target : Nat) (kind : EdgeKind) (effect : Option Nat := none) : StateEdge :=
  { source, target, kind, effect }

def mutate (program : Program) (scenario : String) : Option Program := do
  let first <- program.ops.head?
  let last <- program.ops.getLast?
  let nextOp := (program.ops.map (·.id + 1)).foldl max 0
  let nextEffect := (program.effects.map (·.id + 1)).foldl max 0
  let scheduled := { program with phase := .scheduled }
  match scenario with
  | "built" => pure program
  | "scheduled" => pure scheduled
  | "scheduled-reversed-ops" => pure { scheduled with ops := program.ops.reverse }
  | "scheduled-reversed-first-edge" =>
      match program.edges with
      | [] => none
      | head :: rest =>
          let reversed := edge head.target head.source head.kind head.effect
          pure { scheduled with edges := reversed :: rest }
  | "scheduled-self-edge" => pure { scheduled with edges := program.edges ++ [edge first.id first.id .domOrder] }
  | "dangling-target" => pure { program with edges := program.edges ++ [edge first.id nextOp .dataDependency] }
  | "duplicate-last-op" => pure { program with ops := program.ops ++ [last] }
  | "foreign-scope-edge" =>
      let scope <- program.effects.head?
      pure { program with edges := program.edges ++ [edge first.id last.id .effectOrder (some scope.id)] }
  | "missing-scope-edge" =>
      pure { program with edges := program.edges ++ [edge first.id last.id .effectOrder (some nextEffect)] }
  | _ => none

def codesText : List String -> String
  | [] => "-"
  | codes => ",".intercalate codes

/-- Executable reading of `Precedes` for one trace. -/
def precedes (trace : List TraceEvent) (source target : Nat) : Bool :=
  let ids := trace.map (·.opId)
  let sources := (ids.zipIdx.filter (·.1 == source)).map (·.2)
  let targets := (ids.zipIdx.filter (·.1 == target)).map (·.2)
  !sources.isEmpty && !targets.isEmpty && sources.all (fun i => targets.all (i < ·))

def observe (stem scenario : String) (program : Program) : Except String (Option String) := do
  let some mutated := mutate program scenario | return none
  let codes := orderCodes mutated
  if scenario ∉ ["built", "scheduled"] && codes.isEmpty then
    throw s!"{stem} {scenario}: the scenario must be rejected"
  if codes.isEmpty && mutated.phase == .scheduled then
    for backend in [Backend.vdom, .vapor] do
      for stateEdge in mutated.edges do
        if !precedes (run backend mutated) stateEdge.source stateEdge.target then
          throw s!"{stem}: accepted schedule executes op#{stateEdge.target} too early"
  pure (some s!"{stem} {scenario} {codesText codes}\n")

def check : IO UInt32 := do
  let mut actual := ""
  for stem in stems do
    let text <- IO.FS.readFile s!"fixtures/{stem}.s3.folio"
    match Folio.parseProgram text with
    | .error message => IO.eprintln s!"{stem}: {message}"; return 2
    | .ok program =>
        for scenario in scenarios do
          match observe stem scenario program with
          | .error message => IO.eprintln message; return 1
          | .ok line => actual := actual ++ line.getD ""
  let expected <- IO.FS.readFile "fixtures/schedule-contract.txt"
  if actual != expected then
    IO.eprintln "schedule-contract.txt: Rust validator and Lean contract disagree"
    IO.eprintln "lean:"
    IO.eprint actual
    return 1
  IO.println s!"fixtures/schedule-contract.txt: {(actual.splitOn "\n").length - 1} scheduling verdicts agree"
  pure 0

end Impeto.ScheduleFixture
