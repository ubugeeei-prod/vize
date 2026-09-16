import Impeto.Folio
import Lean.Data.Json

namespace Impeto
open Lean

structure Operand where
  op : Nat
  role : String
  target : Option Nat
  region : Option Nat
  name : Option String
  kind : String
  text : String
  qualifier : String
  span : Span
deriving Repr, DecidableEq

namespace Values

def roles : List String := [
  "tag", "namespace", "attribute", "text", "comment", "binding-kind", "name",
  "value", "modifier", "condition", "for-source", "for-value", "for-key",
  "for-index", "model-read", "model-write", "model-attribute", "params"
]

def u32 (value : Json) : Except String Nat := do
  let n <- value.getNat?
  if n > 4294967295 then throw "S3 integer exceeds u32"
  pure n

def optional {α : Type} (parse : Json -> Except String α) (value : Json) : Except String (Option α) :=
  if value.isNull then pure none else some <$> parse value

def parseOperand (text : String) : Except String Operand := do
  let row <- Json.parse (<- Folio.parseField 0 "operand" text)
  let fields <- row.getArr?
  if fields.size != 10 then throw "expected 10 S3 operand fields"
  let op <- u32 (<- row.getArrVal? 0)
  let role <- (<- row.getArrVal? 1).getStr?
  let target <- optional u32 (<- row.getArrVal? 2)
  let region <- optional u32 (<- row.getArrVal? 3)
  let name <- optional Json.getStr? (<- row.getArrVal? 4)
  let kind <- (<- row.getArrVal? 5).getStr?
  let value <- (<- row.getArrVal? 6).getStr?
  let qualifier <- (<- row.getArrVal? 7).getStr?
  let start <- u32 (<- row.getArrVal? 8)
  let stop <- u32 (<- row.getArrVal? 9)
  if !roles.contains role then throw "unknown S3 operand role"
  if !["absent", "literal", "js", "opaque", "foreign", "vue.filter"].contains kind then
    throw "unknown S3 value kind"
  if start > stop || (kind == "absent" && !value.isEmpty) ||
      (["opaque", "foreign"].contains kind == qualifier.isEmpty) then
    throw "malformed S3 operand value"
  let requiresTarget := ["binding-kind", "value", "modifier", "model-read",
    "model-write", "model-attribute", "params"].contains role
  if (!target.isSome && requiresTarget) ||
      (target.isSome && !requiresTarget && !["tag", "name"].contains role) ||
      (name.isSome != ["attribute", "model-attribute"].contains role) ||
      (region.isSome != (role == "condition")) then
    throw "malformed S3 operand references"
  pure { op, role, target, region, name, kind, text := value, qualifier, span := { start, stop } }

def parse (text : String) : Except String (List Operand) := do
  let mut current := 0
  let mut rows := []
  for (raw, index) in (text.splitOn "\n").zipIdx do
    let line := raw.trimAscii.toString
    if line.isEmpty then continue
    if line == "[s3-values-folio]" && current == 0 then current := 1
    else if line == "[s3-values-folio.operands]" && current == 1 then current := 2
    else if current == 2 then
      match parseOperand line with
      | .ok row => rows := row :: rows
      | .error message => throw s!"line {index + 1}: {message}"
    else throw s!"line {index + 1}: expected S3 value section"
  if current != 2 then throw "missing S3 value sections"
  pure rows.reverse

def forOp (rows : List Operand) (id : Nat) : List Operand :=
  rows.filter (fun row => row.op == id)

def one (rows : List Operand) (id : Nat) (role : String) : Except String Operand :=
  match (forOp rows id).filter (fun row => row.role == role) with
  | [row] => pure row
  | _ => throw s!"op#{id}: expected exactly one {role} operand"

def literal (row : Operand) : Except String String :=
  if row.kind == "literal" then pure row.text else throw s!"expected literal {row.role}"

end Values
end Impeto
