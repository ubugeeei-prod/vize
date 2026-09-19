import Impeto.Values

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

def validateExpression (row : Operand) : Except String Unit := do
  if row.kind != "literal" && !(row.kind == "js" && path row.text) then
    throw s!"unsupported expression {row.kind}: {row.text}"

def lookup (context : Json) (text : String) : Except String Json := do
  if !path text then throw "unsupported member path"
  let mut value := context
  for field in text.splitOn "." do
    value <- value.getObjVal? field
  pure value

def evaluate (context : Json) (row : Operand) : Except String Json := do
  validateExpression row
  if row.kind == "literal" then pure (.str row.text) else lookup context row.text

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
  if source.kind != "js" || !path source.text then throw "unsupported loop source"
  let value <- alias (<- Values.one rows op.id "for-value")
  let index <- Values.one rows op.id "for-key"
  if index.kind != "absent" then
    if (<- alias index) == value then throw "duplicate loop alias"
  if (<- Values.one rows op.id "for-index").kind != "absent" then
    throw "third array alias is outside the reference subset"
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

-- `for-key` is the second array alias. Vue's :key belongs to the repeated root.
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
  let indexRow <- Values.one rows op.id "for-key"
  let indexAlias <- if indexRow.kind == "absent" then pure none else some <$> alias indexRow
  let items <- (<- evaluate context (<- Values.one rows op.id "for-source")).getArr?
  let mut result := []
  let mut seen := []
  for (item, index) in items.toList.zipIdx do
    let mut scope := context.setObjVal! valueAlias item
    if let some name := indexAlias then scope := scope.setObjVal! name (toJson index)
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
