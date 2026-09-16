import Impeto.Values

namespace Impeto.Observation
open Lean

def identifier (text : String) : Bool :=
  let start := fun c => c.isAlpha && c.toNat < 128 || c == '_' || c == '$'
  match text.toList with
  | [] => false
  | first :: rest => start first && rest.all (fun c => start c || c.isDigit)

def evaluate (context : Json) (row : Operand) : Except String Json := do
  if row.kind == "literal" then pure (.str row.text)
  else if row.kind == "js" && identifier row.text then context.getObjVal? row.text
  else throw s!"unsupported expression {row.kind}: {row.text}"

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

def validate (program : Program) (rows : List Operand) : Except String Unit := do
  if program.phase != .built then throw "stateful reference requires built S3"
  if (program.ops.map (·.id)).eraseDups.length != program.ops.length ||
      (program.regions.map (·.id)).eraseDups.length != program.regions.length then
    throw "duplicate S3 identity"
  if (program.regions.filter (fun r => r.parent.isNone)).map (·.id) != [0] then
    throw "expected one root region #0"
  for region in program.regions do
    match region.parent, region.owner with
    | none, none => pure ()
    | some parent, some owner =>
        if parent >= region.id || !program.regions.any (fun r => r.id == parent) ||
            !program.ops.any (fun op => op.id == owner && op.region == parent && op.kind == .insertNode) ||
            (program.regions.filter (fun r => r.owner == some owner)).length != 1 then
          throw "unsupported region ownership"
    | _, _ => throw "malformed region ownership"
  for row in rows do
    let some op := program.ops.find? (fun op => op.id == row.op)
      | throw "operand references unknown op"
    if row.span.start < op.span.start || row.span.stop > op.span.stop then
      throw "operand span escapes op"
  for op in program.ops do
    if !program.regions.any (fun r => r.id == op.region) then throw "unknown op region"
    let operands := Values.forOp rows op.id
    let allowed <- match op.kind with
      | .insertNode => pure ["tag", "namespace", "attribute"]
      | .setProp | .setEvent => pure ["name", "value", "binding-kind"]
      | .setText => pure ["text"]
      | _ => throw s!"unsupported stateful op#{op.id}"
    if operands.any (fun row => !allowed.contains row.role) then throw "unsupported operand role"
    match op.kind with
    | .setProp | .setEvent =>
        if operands.length != 3 then throw "expected three binding operands"
        let name <- Values.one rows op.id "name"
        let value <- Values.one rows op.id "value"
        let kind <- Values.one rows op.id "binding-kind"
        if name.target != value.target || value.target != kind.target ||
            !program.ops.any (fun target => some target.id == value.target &&
              target.region == op.region && target.kind == .insertNode) then
          throw "malformed binding target"
        let expected := if op.kind == .setProp then "bind" else "on"
        if (<- Values.literal kind) != expected then throw "unsupported binding kind"
    | _ =>
        if operands.any (fun row => row.target.isSome) then throw "unexpected structural target"

def attached (program : Program) (rows : List Operand) (id : Nat) : List Op :=
  program.ops.filter (fun op => rows.any (fun row => row.op == op.id &&
    row.role == "binding-kind" && row.target == some id))

def appendNode (nodes : List Json) (node : Json) : List Json :=
  match nodes, node with
  | _, .str "" => nodes
  | .str previous :: rest, .str text => .str (previous ++ text) :: rest
  | _, _ => node :: nodes

def renderRegion (program : Program) (rows : List Operand) (context : Json)
    (fuel : Nat) (regionId : Nat) : Except String (List Json) := do
  match fuel with
  | 0 => throw "cyclic S3 region graph"
  | fuel + 1 =>
      let mut nodes := []
      for op in program.ops.filter (fun op => op.region == regionId) do
        match op.kind with
        | .insertNode =>
            let tag <- Values.literal (<- Values.one rows op.id "tag")
            if !["main", "section", "div", "span", "p", "button"].contains tag then
              throw s!"unsupported HTML tag {tag}"
            if (<- Values.literal (<- Values.one rows op.id "namespace")) != "html" then
              throw "unsupported namespace"
            let mut attrs := []
            for row in Values.forOp rows op.id do
              if row.role == "attribute" then
                let some name := row.name | throw "missing attribute name"
                if !["class", "id", "title"].contains name || attrs.any (fun pair => pair.1 == name) then
                  throw "unsupported or duplicate static attribute"
                attrs := attrs ++ [(name, .str (<- Values.literal row))]
            let mut disabled := false
            let bindings := attached program rows op.id
            if (bindings.filter (fun binding => binding.kind == .setProp)).length > 1 then
              throw "unsupported multiple prop bindings"
            if (bindings.filter (fun binding => binding.kind == .setEvent)).length > 1 then
              throw "unsupported multiple event bindings"
            for binding in bindings do
              let name <- Values.literal (<- Values.one rows binding.id "name")
              let value <- Values.one rows binding.id "value"
              if binding.kind == .setProp then
                if tag != "button" || name != "disabled" then throw "unsupported property"
                disabled <- (<- evaluate context value).getBool?
              else
                if tag != "button" || name != "click" || value.kind != "js" || value.text != "save" then
                  throw "unsupported event binding"
            if disabled then attrs := attrs ++ [("disabled", .str "")]
            let children <- match program.regions.find? (fun r => r.owner == some op.id) with
              | none => pure []
              | some child => renderRegion program rows context fuel child.id
            let fields := [("tag", .str tag), ("attributes", Json.mkObj attrs),
              ("children", .arr children.toArray)]
            let fields := if tag == "button" then fields ++ [("disabled", .bool disabled)] else fields
            nodes := appendNode nodes (Json.mkObj fields)
        | .setText =>
            let text <- display (<- evaluate context (<- Values.one rows op.id "text"))
            nodes := appendNode nodes (.str text)
        | .setProp | .setEvent => pure ()
        | _ => throw "unsupported stateful op"
      pure nodes.reverse

def render (program : Program) (rows : List Operand) (context : Json) : Except String Json := do
  validate program rows
  pure (.arr (<- renderRegion program rows context (program.regions.length + 1) 0).toArray)

end Impeto.Observation
