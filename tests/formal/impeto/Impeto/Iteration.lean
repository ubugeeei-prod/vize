import Impeto.Values
import Impeto.Expression

namespace Impeto.Iteration
open Lean

def identifier (text : String) : Bool :=
  let start := fun c => c.isAlpha && c.toNat < 128 || c == '_' || c == '$'
  match text.toList with
  | [] => false
  | first :: rest => start first && rest.all (fun c => start c || c.isDigit)

def path (text : String) : Bool :=
  (text.splitOn ".").all (fun part => identifier part &&
    !["__proto__", "constructor", "prototype"].contains part)

/-- `toDisplayString` over the reference value domain. -/
def display (value : Json) : Except String String :=
  match value with
  | .null => pure ""
  | .str text => pure text
  | .bool true => pure "true"
  | .bool false => pure "false"
  | .num _ => do
      let n <- value.getInt?
      if n < -2147483648 || n > 2147483647 then throw "unsupported display number"
      pure (toString n)
  | _ => throw "unsupported display value"

/-- Literal text whose DOM spelling needs no entity decoding or whitespace condensing. -/
def plainText (text : String) : Bool :=
  !text.toList.any (fun c => c == '&' || c == '<' || c == '\n' || c == '\t' || c == '\r') &&
    (text.splitOn "  ").length == 1

/-- Compound text as S3 preserves it: literal runs and `{{ expression }}` holes. -/
def compoundParts (text : String) : Except String (List (Bool × String)) := do
  let pieces := text.splitOn "{{"
  let some first := pieces.head? | throw "empty compound text"
  if !plainText first || first.contains '}' then throw "unsupported compound literal"
  let mut parts := if first.isEmpty then [] else [(false, first)]
  for piece in pieces.drop 1 do
    let [expr, literal] := piece.splitOn "}}" | throw "unbalanced compound interpolation"
    if !plainText literal || literal.contains '}' then throw "unsupported compound literal"
    parts := parts ++ [(true, expr)] ++ (if literal.isEmpty then [] else [(false, literal)])
  if !parts.any (·.1) then throw "compound text without interpolation"
  pure parts

def validateExpression (row : Operand) : Except String Unit := do
  if row.kind == "literal" then return
  if row.kind == "opaque" && row.qualifier == "compound" then
    for (isExpr, text) in <- compoundParts row.text do
      if isExpr then
        if let .error message := Expression.parse text then
          throw s!"unsupported interpolation {text}: {message}"
    return
  if row.kind != "js" then throw s!"unsupported expression {row.kind}: {row.text}"
  match Expression.parse row.text with
  | .ok _ => pure ()
  | .error message => throw s!"unsupported expression {row.text}: {message}"

def lookup (context : Json) (text : String) : Except String Json := do
  if !path text then throw "unsupported member path"
  let mut value := context
  for field in text.splitOn "." do
    value <- value.getObjVal? field
  pure value

def evaluate (context : Json) (row : Operand) : Except String Json := do
  validateExpression row
  if row.kind == "literal" then return .str row.text
  if row.kind == "opaque" then
    let mut text := ""
    for (isExpr, part) in <- compoundParts row.text do
      let piece <- if isExpr then display (<- Expression.evaluate context part) else pure part
      text := text ++ piece
    return .str text
  Expression.evaluate context row.text

def recordArgument (text : String) : Option String :=
  if text.startsWith "record(" && text.endsWith ")" then
    let argument := String.ofList (text.toList.drop 7 |>.dropLast)
    if path argument then some argument else none
  else none

def event (context : Json) (row : Operand) : Except String Json := do
  if row.kind != "js" then throw "unsupported event value"
  if row.text == "save" then pure (.str "save")
  else match recordArgument row.text with
    | none => throw "unsupported event expression"
    | some argument => do
        let value <- lookup context argument
        pure (.str (<- value.getStr?))

def alias (row : Operand) : Except String String := do
  if !(row.kind == "js" || (row.kind == "opaque" && row.qualifier == "for-value")) ||
      !identifier row.text ||
      ["save", "record", "$event", "$slots", "__proto__", "constructor", "prototype"].contains row.text then
    throw "unsupported loop alias"
  pure row.text

def validate (program : Program) (rows : List Operand) (op : Op) : Except String Unit := do
  let operands := Values.forOp rows op.id
  if operands.length != 4 then throw "expected four loop operands"
  let source <- Values.one rows op.id "for-source"
  if source.kind != "js" then throw "unsupported loop source"
  validateExpression source
  let mut aliases := [<- alias (<- Values.one rows op.id "for-value")]
  for role in ["for-key", "for-index"] do
    let row <- Values.one rows op.id role
    if row.kind != "absent" then
      let name <- alias row
      if aliases.contains name then throw "duplicate loop alias"
      aliases := aliases ++ [name]
  if (<- Values.one rows op.id "for-key").kind == "absent" &&
      (<- Values.one rows op.id "for-index").kind != "absent" then
    throw "third loop alias without a second alias"
  let [region] := program.regions.filter (fun r => r.owner == some op.id)
    | throw "expected one loop body"
  let [root] := program.ops.filter (fun child => child.region == region.id && child.kind == .insertNode)
    | throw "expected one native loop root"
  if program.ops.any (fun child => child.region == region.id &&
      ![.insertNode, .setProp, .setEvent, .setText].contains child.kind) then
    throw "unsupported loop root shape"
  if program.ops.any (fun child => child.region == region.id && child.kind == .setText &&
      !(Values.forOp rows child.id).any (fun row => row.target == some root.id)) then
    throw "loop root text must belong to its element"

def keyValue (value : Json) : Except String Json := do
  match value with
  | .str _ => pure value
  | .num _ =>
      let n <- value.getInt?
      if n < -2147483648 || n > 2147483647 then throw "unsupported loop key integer"
      pure value
  | _ => throw "loop keys must be strings or signed 32-bit integers"

/-- JavaScript enumerates canonical array-index keys before other keys, so
objects with such keys have no order shared with the JSON reference. -/
def arrayIndexKey (key : String) : Bool :=
  match key.toNat? with
  | some n => toString n == key && n < 4294967295
  | none => false

/-- Iteration entries as `renderList` produces them: value, second alias and
optional third alias. Objects iterate in the canonical key order that the
Rust scenario loader also serializes, and ranges yield `1..n`. -/
def entries (source : Json) (third : Bool) : Except String (List (Json × Json × Json)) := do
  match source with
  | .arr items =>
      if third then throw "third loop alias requires an object source"
      pure (items.toList.zipIdx.map fun (item, index) => (item, toJson index, .null))
  | .num _ =>
      if third then throw "third loop alias requires an object source"
      let n <- Expression.integer source
      if n < 0 || n > 1000 then throw "unsupported range source"
      pure ((List.range n.toNat).map fun index => (toJson (index + 1), toJson index, .null))
  | .obj fields =>
      let pairs := fields.toList.map fun ⟨key, value⟩ => (key, value)
      if pairs.any (arrayIndexKey ·.1) then throw "unsupported integer-like object key"
      pure (pairs.zipIdx.map fun ((key, value), index) => (value, .str key, toJson index))
  | _ => throw "unsupported loop source value"

-- `for-key` is the second alias (index or object key). Vue's :key belongs to the repeated root.
def contexts (program : Program) (rows : List Operand) (op : Op)
    (context : Json) : Except String (List (Json × Json)) := do
  validate program rows op
  let [region] := program.regions.filter (fun r => r.owner == some op.id)
    | throw "missing loop body"
  let some root := program.ops.find? (fun child => child.region == region.id && child.kind == .insertNode)
    | throw "missing loop root"
  let keys := rows.filter (fun row => row.role == "name" && row.target == some root.id &&
    row.kind == "literal" && row.text == "key")
  if keys.length > 1 then throw "duplicate loop key binding"
  let valueAlias <- alias (<- Values.one rows op.id "for-value")
  let secondRow <- Values.one rows op.id "for-key"
  let secondAlias <- if secondRow.kind == "absent" then pure none else some <$> alias secondRow
  let thirdRow <- Values.one rows op.id "for-index"
  let thirdAlias <- if thirdRow.kind == "absent" then pure none else some <$> alias thirdRow
  let source <- evaluate context (<- Values.one rows op.id "for-source")
  let mut result := []
  let mut seen := []
  for ((item, second, third), index) in (<- entries source thirdAlias.isSome).zipIdx do
    let mut scope := context.setObjVal! valueAlias item
    if let some name := secondAlias then scope := scope.setObjVal! name second
    if let some name := thirdAlias then scope := scope.setObjVal! name third
    let identity <- match keys with
      | [] => pure (Json.arr #[.str "position", toJson index])
      | [key] => do
          let value <- keyValue (<- evaluate scope (<- Values.one rows key.op "value"))
          if seen.contains value then throw "duplicate loop key"
          seen := value :: seen
          pure (Json.arr #[.str "key", value])
      | _ => throw "ambiguous loop key"
    result := result ++ [(Json.arr #[toJson op.id, identity], scope)]
  pure result

end Impeto.Iteration
