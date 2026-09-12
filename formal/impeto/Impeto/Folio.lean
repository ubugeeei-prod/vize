import Impeto.Syntax

namespace Impeto
namespace Folio

inductive Section where
  | none
  | root
  | regions
  | ops
  | edges
  | effects
deriving Repr, DecidableEq

structure State where
  current : Section
  program : Program
deriving Repr

def State.empty : State :=
  { current := .none, program := Program.empty }

def fail {α : Type} (line : Nat) (message : String) : Except String α :=
  throw s!"line {line}: {message}"

def parseField (line : Nat) (name : String) (token : String) : Except String String := do
  let tag := name ++ "="
  if token.startsWith tag then
    pure ((token.drop tag.length).toString)
  else
    fail line s!"expected {tag}..., got {token}"

def parseNat (line : Nat) (name : String) (text : String) : Except String Nat := do
  match text.toNat? with
  | some value => pure value
  | none => fail line s!"invalid {name} integer {text}"

def parseOptionNat (line : Nat) (name : String) (text : String) : Except String (Option Nat) := do
  if text == "-" then
    pure none
  else
    pure (some (<- parseNat line name text))

def parseSpan (line : Nat) (text : String) : Except String Span := do
  match text.splitOn ":" with
  | [start, stop] => pure { start := <- parseNat line "span.start" start, stop := <- parseNat line "span.stop" stop }
  | _ => fail line s!"invalid span {text}"

def parseRegion (line : Nat) : List String -> Except String Region
  | [idToken, parentToken, ownerToken, spanToken] => do
      let id := <- parseNat line "id" (<- parseField line "id" idToken)
      let parent := <- parseOptionNat line "parent" (<- parseField line "parent" parentToken)
      let owner := <- parseOptionNat line "owner" (<- parseField line "owner" ownerToken)
      let span := <- parseSpan line (<- parseField line "span" spanToken)
      pure { id, parent, owner, span }
  | tokens => fail line s!"expected 4 fields, got {tokens.length}"

def parseOp (line : Nat) : List String -> Except String Op
  | [idToken, kindToken, regionToken, effectToken, spanToken] => do
      let id := <- parseNat line "id" (<- parseField line "id" idToken)
      let rawKind := <- parseField line "kind" kindToken
      let kind := <- match OpKind.parse rawKind with
        | some kind => pure kind
        | none => fail line s!"unknown op kind {rawKind}"
      let region := <- parseNat line "region" (<- parseField line "region" regionToken)
      let effect := <- parseOptionNat line "effect" (<- parseField line "effect" effectToken)
      let span := <- parseSpan line (<- parseField line "span" spanToken)
      pure { id, kind, region, effect, span }
  | tokens => fail line s!"expected 5 fields, got {tokens.length}"

def parseEdge (line : Nat) : List String -> Except String StateEdge
  | [fromToken, toToken, kindToken, effectToken] => do
      let source := <- parseNat line "from" (<- parseField line "from" fromToken)
      let target := <- parseNat line "to" (<- parseField line "to" toToken)
      let rawKind := <- parseField line "kind" kindToken
      let kind := <- match EdgeKind.parse rawKind with
        | some kind => pure kind
        | none => fail line s!"unknown edge kind {rawKind}"
      let effect := <- parseOptionNat line "effect" (<- parseField line "effect" effectToken)
      pure { source, target, kind, effect }
  | tokens => fail line s!"expected 4 fields, got {tokens.length}"

def parseEffect (line : Nat) : List String -> Except String EffectScope
  | [idToken, ownerToken, regionToken, spanToken] => do
      let id := <- parseNat line "id" (<- parseField line "id" idToken)
      let owner := <- parseNat line "owner" (<- parseField line "owner" ownerToken)
      let region := <- parseNat line "region" (<- parseField line "region" regionToken)
      let span := <- parseSpan line (<- parseField line "span" spanToken)
      pure { id, owner, region, span }
  | tokens => fail line s!"expected 4 fields, got {tokens.length}"

def parsePhaseLine (line : Nat) (text : String) (state : State) : Except String State := do
  let rawPhase := <- parseField line "phase" text
  let phase := <- match Phase.parse rawPhase with
    | some phase => pure phase
    | none => fail line s!"unknown phase {rawPhase}"
  pure { state with program := { state.program with phase } }

def parseRecordLine (line : Nat) (text : String) (state : State) : Except String State := do
  let tokens := text.splitOn " "
  match state.current with
  | .none => fail line "record before [s3-folio]"
  | .root => parsePhaseLine line text state
  | .regions =>
      let region := <- parseRegion line tokens
      let program := state.program
      pure { state with program := { program with regions := program.regions ++ [region] } }
  | .ops =>
      let op := <- parseOp line tokens
      let program := state.program
      pure { state with program := { program with ops := program.ops ++ [op] } }
  | .edges =>
      let edge := <- parseEdge line tokens
      let program := state.program
      pure { state with program := { program with edges := program.edges ++ [edge] } }
  | .effects =>
      let effect := <- parseEffect line tokens
      let program := state.program
      pure { state with program := { program with effects := program.effects ++ [effect] } }

def parseLine (line : Nat) (raw : String) (state : State) : Except String State := do
  let text := raw.trimAscii.toString
  if text.isEmpty then
    pure state
  else if text == "[s3-folio]" then
    pure { state with current := .root }
  else if text == "[s3-folio.regions]" then
    pure { state with current := .regions }
  else if text == "[s3-folio.ops]" then
    pure { state with current := .ops }
  else if text == "[s3-folio.edges]" then
    pure { state with current := .edges }
  else if text == "[s3-folio.effects]" then
    pure { state with current := .effects }
  else
    parseRecordLine line text state

def parseLines : List String -> Nat -> State -> Except String State
  | [], _, state => pure state
  | line :: rest, number, state => do
      let state := <- parseLine number line state
      parseLines rest (number + 1) state

def parseProgram (text : String) : Except String Program := do
  let state := <- parseLines (text.splitOn "\n") 1 State.empty
  pure state.program

end Folio
end Impeto
