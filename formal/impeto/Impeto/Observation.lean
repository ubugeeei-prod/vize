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

def textBinding (rows : List Operand) (op : Op) : Bool :=
  op.kind == .setText && (Values.forOp rows op.id).any (fun row => row.role == "binding-kind")

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
            !program.ops.any (fun op => op.id == owner && op.region == parent &&
              [.insertNode, .branch, .slotOutlet].contains op.kind) then
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
    let children := program.regions.filter (fun r => r.owner == some op.id)
    if op.kind != .branch && children.length > 1 then throw "duplicate child region"
    let isTextBinding := textBinding rows op
    let allowed <- match op.kind with
      | .insertNode => pure ["tag", "namespace", "attribute"]
      | .setProp | .setEvent => pure ["name", "value", "binding-kind"]
      | .setText => pure (if isTextBinding then ["value", "binding-kind"] else ["text"])
      | .branch => pure ["condition"]
      | .slotOutlet => pure ["name"]
      | _ => throw s!"unsupported stateful op#{op.id}"
    if operands.any (fun row => !allowed.contains row.role) then throw "unsupported operand role"
    if op.kind == .insertNode then
      let _ <- Values.one rows op.id "tag"
      let _ <- Values.one rows op.id "namespace"
    if op.kind == .setText && !isTextBinding then
      let _ <- Values.one rows op.id "text"
    if [.setProp, .setEvent].contains op.kind || isTextBinding then
      if operands.length != (if isTextBinding then 2 else 3) then
        throw "unexpected binding operand count"
      let value <- Values.one rows op.id "value"
      let kind <- Values.one rows op.id "binding-kind"
      if value.target != kind.target ||
          !program.ops.any (fun target => some target.id == value.target &&
            target.region == op.region && target.kind == .insertNode) then
        throw "malformed binding target"
      if !isTextBinding then
        if (<- Values.one rows op.id "name").target != value.target then
          throw "malformed binding name target"
      let expected := if isTextBinding then "vue.text" else if op.kind == .setProp then "bind" else "on"
      if (<- Values.literal kind) != expected then throw "unsupported binding kind"
    else if operands.any (fun row => row.target.isSome) then
      throw "unexpected structural target"
    if op.kind == .branch then
      if children.isEmpty || operands.map (·.region) != children.map (fun r => some r.id) then
        throw "conditions must name every owned branch region in authored order"
      for (row, index) in operands.zipIdx do
        if row.kind == "absent" then
          if index == 0 || index + 1 != operands.length then throw "else must be the final branch"
        else if row.kind != "js" || !identifier row.text then throw "unsupported condition"
    if op.kind == .slotOutlet then
      if children.length != 1 || operands.length != 1 then throw "unsupported slot shape"
      if (<- Values.literal (<- Values.one rows op.id "name")).isEmpty then
        throw "unsupported slot name"

def attached (program : Program) (rows : List Operand) (id : Nat) : List Op :=
  program.ops.filter (fun op => rows.any (fun row => row.op == op.id &&
    row.role == "binding-kind" && row.target == some id))

def appendNode (nodes : List Json) (node : Json) : List Json :=
  match nodes, node with
  | _, .str "" => nodes
  | .str previous :: rest, .str text => .str (previous ++ text) :: rest
  | _, _ => node :: nodes

def selectedBranch (rows : List Operand) (context : Json) (id : Nat) : Except String (Option Nat) := do
  for row in Values.forOp rows id do
    let selected <- if row.kind == "absent" then pure true
      else (<- evaluate context row).getBool?
    if selected then
      return row.region
  pure none

structure Rendered where
  nodes : List Json := []
  buttons : List (Bool × Bool) := []

def renderRegion (program : Program) (rows : List Operand) (context : Json)
    (fuel : Nat) (regionId : Nat) : Except String Rendered := do
  match fuel with
  | 0 => throw "cyclic S3 region graph"
  | fuel + 1 =>
      let mut nodes := []
      let mut buttons := []
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
            for binding in bindings.filter (fun binding => !textBinding rows binding) do
              let name <- Values.literal (<- Values.one rows binding.id "name")
              let value <- Values.one rows binding.id "value"
              if binding.kind == .setProp then
                if tag != "button" || name != "disabled" then throw "unsupported property"
                disabled <- (<- evaluate context value).getBool?
              else if binding.kind == .setEvent then
                if tag != "button" || name != "click" || value.kind != "js" || value.text != "save" then
                  throw "unsupported event binding"
            if disabled then attrs := attrs ++ [("disabled", .str "")]
            let child := program.regions.find? (fun r => r.owner == some op.id)
            let rendered <- match bindings.filter (textBinding rows) with
              | [] => match child with
                | none => pure ({} : Rendered)
                | some child => renderRegion program rows context fuel child.id
              | [binding] => do
                if child.any (fun r => program.ops.any (fun op => op.region == r.id)) then
                  throw "unsupported v-text with authored children"
                let text <- display (<- evaluate context (<- Values.one rows binding.id "value"))
                pure { nodes := appendNode [] (.str text) }
              | _ => throw "duplicate v-text binding"
            let fields := [("tag", .str tag), ("attributes", Json.mkObj attrs),
              ("children", .arr rendered.nodes.toArray)]
            let fields := if tag == "button" then fields ++ [("disabled", .bool disabled)] else fields
            nodes := appendNode nodes (Json.mkObj fields)
            if tag == "button" then
              buttons := buttons ++ [(disabled, bindings.any (fun binding => binding.kind == .setEvent))]
            buttons := buttons ++ rendered.buttons
        | .setText =>
            if !textBinding rows op then
              let text <- display (<- evaluate context (<- Values.one rows op.id "text"))
              nodes := appendNode nodes (.str text)
        | .branch | .slotOutlet =>
            let region <- if op.kind == .branch then selectedBranch rows context op.id
              else pure ((program.regions.find? (fun r => r.owner == some op.id)).map (·.id))
            if let some region := region then
              let rendered <- renderRegion program rows context fuel region
              for node in rendered.nodes do
                nodes := appendNode nodes node
              buttons := buttons ++ rendered.buttons
        | .setProp | .setEvent => pure ()
        | _ => throw "unsupported stateful op"
      pure { nodes := nodes.reverse, buttons }

def observe (program : Program) (rows : List Operand) (context : Json) : Except String Rendered := do
  validate program rows
  renderRegion program rows context (program.regions.length + 1) 0

def render (program : Program) (rows : List Operand) (context : Json) : Except String Json := do
  pure (.arr (<- observe program rows context).nodes.toArray)

end Impeto.Observation
