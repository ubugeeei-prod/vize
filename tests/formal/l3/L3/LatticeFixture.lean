import Impeto.Lattice

/-!
Differential between the Rust P3-2 evaluator and the Lean classifier. The
committed `[s3-reactivity-folio]` page is printed by Rust for an enumerated
input matrix; this module parses it independently, re-enumerates the same
matrix and recomputes every class. The matrix is all 256 effect sets for an
inert local binding, then every origin × escape pair with no effect and with
each single effect, which reaches every combination of rule floors.
-/

namespace Impeto.LatticeFixture
open Lattice

structure Row where
  id : Nat
  value : Class
  verdict : Verdict
  origin : Origin
  effects : List Effect
  escape : Escape
  span : Nat × Nat
deriving DecidableEq

def spelled {α : Type} (all : List α) (spelling : α -> String) (what text : String) :
    Except String α :=
  match all.filter (fun value => spelling value == text) with
  | [value] => pure value
  | _ => throw s!"unknown {what} `{text}`"

def field (name token : String) : Except String String :=
  if token.startsWith (name ++ "=") then pure (token.drop (name.length + 1)).toString
  else throw s!"expected `{name}=...`, got `{token}`"

def number (text : String) : Except String Nat :=
  match text.toNat? with
  | some value => pure value
  | none => throw s!"invalid number `{text}`"

def parseEffects (text : String) : Except String (List Effect) := do
  if text == "-" then return []
  let effects <- (text.splitOn ",").mapM (spelled Effect.all Effect.spelling "effect")
  if Effect.all.filter (effects.contains ·) != effects then
    throw s!"effects `{text}` must be distinct and in vocabulary order"
  pure effects

def parseRow (line : String) : Except String Row := do
  let [id, value, verdict, origin, effects, escape, span] := line.splitOn " "
    | throw s!"expected 7 fields in `{line}`"
  let [start, stop] := (<- field "span" span).splitOn ":"
    | throw s!"invalid span in `{line}`"
  pure {
    id := <- number (<- field "id" id)
    value := <- spelled Class.all Class.spelling "class" (<- field "class" value)
    verdict := <- spelled Verdict.all Verdict.spelling "verdict" (<- field "verdict" verdict)
    origin := <- spelled Origin.all Origin.spelling "origin" (<- field "origin" origin)
    effects := <- parseEffects (<- field "effects" effects)
    escape := <- spelled Escape.all Escape.spelling "escape" (<- field "escape" escape)
    span := (<- number start, <- number stop)
  }

def printEffects : List Effect -> String
  | [] => "-"
  | effects => ",".intercalate (effects.map Effect.spelling)

def printRow (row : Row) : String :=
  s!"id={row.id} class={row.value.spelling} verdict={row.verdict.spelling} " ++
    s!"origin={row.origin.spelling} effects={printEffects row.effects} " ++
    s!"escape={row.escape.spelling} span={row.span.1}:{row.span.2}\n"

def print (rows : List Row) : String :=
  "[s3-reactivity-folio]\n\n[s3-reactivity-folio.bindings]\n" ++ String.join (rows.map printRow)

def parse (text : String) : Except String (List Row) := do
  let lines := (text.splitOn "\n").filter (· != "")
  let "[s3-reactivity-folio]" :: "[s3-reactivity-folio.bindings]" :: rows := lines
    | throw "missing reactivity folio sections"
  let rows <- rows.mapM parseRow
  if print rows != text then throw "fact page is not in canonical print form"
  pure rows

def subset (mask : Nat) : List Effect :=
  (Effect.all.zipIdx.filter (fun pair => mask.testBit pair.2)).map (·.1)

def matrix : List (Origin × List Effect × Escape) :=
  (List.range 256).map (fun mask => (.local, subset mask, .none)) ++
    Origin.all.flatMap fun origin => Escape.all.flatMap fun escape =>
      ([] :: Effect.all.map (fun effect => [effect])).map fun effects => (origin, effects, escape)

def verdictAt (index : Nat) : Verdict :=
  match index % 3 with
  | 0 => .proven
  | 1 => .refuted
  | _ => .unknown

/-- The matrix must stay exhaustive where it claims to be. -/
def coverage : Except String Unit := do
  if ((List.range 256).map subset).eraseDups.length != 256 then
    throw "effect-set enumeration is not the full powerset"
  let floors := matrix.map fun (origin, effects, escape) =>
    (origin.floor, effectsFloor effects, escape.floor)
  if floors.eraseDups.length != 3 * 4 * 4 then
    throw "matrix misses a combination of origin, effect and escape floors"

def check (classifier : Input -> Class) (text : String) : Except String Nat := do
  coverage
  let rows <- parse text
  if rows.length != matrix.length then
    throw s!"expected {matrix.length} fact rows, found {rows.length}"
  for ((row, (origin, effects, escape)), index) in (rows.zip matrix).zipIdx do
    if row.id != index || row.span != (0, 0) then throw s!"row {index}: unexpected id or span"
    if row.origin != origin || row.effects != effects || row.escape != escape ||
        row.verdict != verdictAt index then
      throw s!"row {index}: input differs from the enumerated matrix"
    let expected := classifier { origin, effects, escape, verdict := row.verdict }
    if row.value != expected then
      throw s!"row {index}: Rust wrote {row.value.spelling}, Lean classifies {expected.spelling}"
  pure rows.length

def expectRejected (label : String) (result : Except String Nat) : Except String Unit :=
  match result with
  | .ok _ => throw s!"lattice differential accepted {label}"
  | .error _ => pure ()

/-- Damaged pages and wrong classifiers must both be rejected. -/
def negativeTests (text : String) : Except String Unit := do
  let rows <- parse text
  for (label, damaged) in [
    ("a promoted class", text.replace "id=0 class=static" "id=0 class=props-stable"),
    ("a dropped row", print rows.dropLast),
    ("a duplicated row", print (rows ++ rows.take 1)),
    ("non-canonical effect order", text.replace "effects=freeze,capture " "effects=capture,freeze "),
    ("a duplicated effect", text.replace "effects=freeze,capture " "effects=freeze,freeze "),
    ("an unknown origin", text.replace "origin=prop " "origin=props "),
    ("an extra field", text.replace "span=0:0\n" "span=0:0 extra=1\n"),
    ("a changed verdict", text.replace "id=1 class=props-stable verdict=refuted"
      "id=1 class=props-stable verdict=proven")
  ] do
    if damaged == text then throw s!"vacuous damage: {label}"
    expectRejected label (check classify damaged)
  let mutants : List (String × (Input -> Class)) := [
    ("ignoring escapes", fun i => i.origin.floor.join (effectsFloor i.effects)),
    ("ignoring effects", fun i => i.origin.floor.join i.escape.floor),
    ("ignoring origins", fun i => (effectsFloor i.effects).join i.escape.floor),
    ("allocation as static", fun i => classify { i with effects := i.effects.filter (· != .allocate) }),
    ("capture as props-stable", fun i =>
      if i.effects == [.capture] && i.origin == .local && i.escape == .none then .propsStable
      else classify i)
  ]
  for (label, mutant) in mutants do
    expectRejected s!"a classifier {label}" (check mutant text)

def checkFile (path : String) : IO UInt32 := do
  let text <- IO.FS.readFile path
  match check classify text, negativeTests text with
  | .ok count, .ok () =>
      IO.println s!"{path}: {count} Rust lattice facts agree with the Lean classifier"
      pure 0
  | .error message, _ | _, .error message =>
      IO.eprintln s!"{path}: {message}"
      pure 1

end Impeto.LatticeFixture
