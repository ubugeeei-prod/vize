import Impeto.Model
import Impeto.View

namespace Impeto.Observation
open Lean

abbrev identifier := Iteration.identifier
abbrev validateExpression := Iteration.validateExpression
abbrev evaluate := Iteration.evaluate

abbrev display := Iteration.display

def textBinding (rows : List Operand) (op : Op) : Bool :=
  op.kind == .setText && (Values.forOp rows op.id).any (fun row => row.role == "binding-kind")

def attached (program : Program) (rows : List Operand) (id : Nat) : List Op :=
  program.ops.filter (fun op => rows.any (fun row => row.op == op.id &&
    row.role == "binding-kind" && row.target == some id))

def staticElement (rows : List Operand) (id : Nat) : Except String (String × List (String × Json)) := do
  let tag <- Values.literal (<- Values.one rows id "tag")
  if !["main", "section", "div", "span", "p", "button", "input", "select", "option"].contains tag then
    throw s!"unsupported HTML tag {tag}"
  if (<- Values.literal (<- Values.one rows id "namespace")) != "html" then
    throw "unsupported namespace"
  let mut attrs := []
  for row in Values.forOp rows id do
    if row.role == "attribute" then
      let some name := row.name | throw "missing attribute name"
      if !["class", "id", "title", "type", "value", "multiple"].contains name ||
          attrs.any (fun pair => pair.1 == name) then
        throw "unsupported or duplicate static attribute"
      -- Form attributes are admitted only where the model reference defines them.
      if (name == "type" && tag != "input") || (name == "value" && !["input", "option"].contains tag) ||
          (name == "multiple" && tag != "select") || ((name == "multiple") != (row.kind == "absent")) then
        throw "unsupported form attribute"
      let value <- if row.kind == "absent" then pure "" else Values.literal row
      attrs := attrs ++ [(name, .str value)]
  pure (tag, attrs)

def validateElementBindings (program : Program) (rows : List Operand) (op : Op)
    (tag : String) : Except String Unit := do
  let bindings := attached program rows op.id
  let models := bindings.filter (Model.modelRow rows)
  if ["input", "select"].contains tag then
    if models.length != 1 || bindings.length != 1 then
      throw "form controls require exactly one model binding"
    return
  if !models.isEmpty then throw "unsupported model target"
  if tag == "option" then
    let owner := (program.regions.find? (·.id == op.region)).bind (·.owner)
    let parent := program.ops.find? (fun parent => some parent.id == owner && parent.kind == .insertNode)
    let parentTag <- parent.elim (pure "") (fun parent => do
      Values.literal (<- Values.one rows parent.id "tag"))
    if parentTag != "select" || !bindings.isEmpty then throw "options must be static select children"
    for region in program.regions.filter (fun r => r.owner == some op.id) do
      for child in program.ops.filter (fun child => child.region == region.id) do
        if child.kind != .setText || (<- Values.one rows child.id "text").kind != "literal" then
          throw "option labels must be static text"
  for kind in [OpKind.setEvent, .setText] do
    if (bindings.filter (fun binding => binding.kind == kind)).length > 1 then
      throw "unsupported multiple bindings of the same kind"
  let mut names := (<- staticElement rows op.id).2.map (·.1)
  for binding in bindings do
    if textBinding rows binding then
      if program.regions.any (fun r => r.owner == some op.id &&
          program.ops.any (fun child => child.region == r.id)) then
        throw "unsupported v-text with authored children"
    else
      let name <- Values.literal (<- Values.one rows binding.id "name")
      if binding.kind == .setProp then
        if (name == "disabled" && tag != "button") ||
            !["disabled", "title", "data-id", "key"].contains name then throw "unsupported property"
        if names.contains name then throw "duplicate property binding"
        names := name :: names
        if name == "key" && !program.regions.any (fun region => region.id == op.region &&
            program.ops.any (fun owner => some owner.id == region.owner && owner.kind == .loop)) then
          throw "key binding requires a loop root"
        if (<- Values.one rows binding.id "value").kind != "js" then
          throw "unsupported property expression"
      else if binding.kind == .setEvent then
        let value <- Values.one rows binding.id "value"
        if tag != "button" || name != "click" || value.kind != "js" ||
            (value.text != "save" && (Iteration.recordArgument value.text).isNone) then
          throw "unsupported event binding"

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
              [.insertNode, .branch, .slotOutlet, .loop].contains op.kind) then
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
    let model := Model.modelRow rows op
    if model then
      let _ <- Model.control program rows op
    let allowed <- match op.kind with
      | .insertNode => pure ["tag", "namespace", "attribute"]
      | .setProp => pure (if model then ["name", "model-read", "model-write", "model-attribute",
          "binding-kind"] else ["name", "value", "binding-kind"])
      | .setEvent => pure ["name", "value", "binding-kind"]
      | .setText => pure (if isTextBinding then ["value", "binding-kind"] else ["text"])
      | .branch => pure ["condition"]
      | .slotOutlet => pure ["name"]
      | .loop => pure ["for-source", "for-value", "for-key", "for-index"]
      | _ => throw s!"unsupported stateful op#{op.id}"
    if operands.any (fun row => !allowed.contains row.role) then throw "unsupported operand role"
    if op.kind == .insertNode then
      let (tag, _) <- staticElement rows op.id
      validateElementBindings program rows op tag
    if op.kind == .setText && !isTextBinding then
      validateExpression (<- Values.one rows op.id "text")
    if model then pure ()
    else if [.setProp, .setEvent].contains op.kind || isTextBinding then
      if operands.length != (if isTextBinding then 2 else 3) then
        throw "unexpected binding operand count"
      let value <- Values.one rows op.id "value"
      if op.kind != .setEvent then validateExpression value
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
        else if row.kind != "js" then throw "unsupported condition"
        else validateExpression row
    if op.kind == .slotOutlet then
      if children.length != 1 || operands.length != 1 then throw "unsupported slot shape"
      if (<- Values.literal (<- Values.one rows op.id "name")).isEmpty then
        throw "unsupported slot name"
    if op.kind == .loop then Iteration.validate program rows op

def selectedBranch (rows : List Operand) (context : Json) (id : Nat) : Except String (Option Nat) := do
  for row in Values.forOp rows id do
    let selected <- if row.kind == "absent" then pure true
      else Expression.truthy <$> evaluate context row
    if selected then
      return row.region
  pure none

structure Rendered where
  nodes : List View := []
  buttons : List (Bool × Bool) := []

def renderRegion (program : Program) (rows : List Operand) (context : Json)
    (fuel : Nat) (regionId : Nat) (scope : List Json := []) : Except String Rendered := do
  match fuel with
  | 0 => throw "cyclic S3 region graph"
  | fuel + 1 =>
      let mut nodes := []
      let mut buttons := []
      for op in program.ops.filter (fun op => op.region == regionId) do
        match op.kind with
        | .insertNode =>
            let (tag, staticAttrs) <- staticElement rows op.id
            let mut attrs := staticAttrs
            let mut disabled := false
            let bindings := attached program rows op.id
            for binding in bindings.filter (fun binding => binding.kind == .setProp &&
                !Model.modelRow rows binding) do
              let name <- Values.literal (<- Values.one rows binding.id "name")
              if name == "disabled" then
                disabled <- (<- evaluate context (<- Values.one rows binding.id "value")).getBool?
              else if name != "key" then
                let value <- evaluate context (<- Values.one rows binding.id "value")
                if value != .null then attrs := attrs ++ [(name, .str (<- display value))]
            if disabled then attrs := attrs ++ [("disabled", .str "")]
            let child := program.regions.find? (fun r => r.owner == some op.id)
            let rendered <- match bindings.filter (textBinding rows) with
              | [] => match child with
                | none => pure ({} : Rendered)
                | some child => renderRegion program rows context fuel child.id scope
              | [binding] => do
                if child.any (fun r => program.ops.any (fun op => op.region == r.id)) then
                  throw "unsupported v-text with authored children"
                let text <- display (<- evaluate context (<- Values.one rows binding.id "value"))
                pure { nodes := View.append [] (.text text) }
              | _ => throw "duplicate v-text binding"
            let event <- match bindings.filter (fun binding => binding.kind == .setEvent) with
              | [] => pure none
              | [binding] => some <$> Iteration.event context (<- Values.one rows binding.id "value")
              | _ => throw "duplicate event binding"
            let address := (Json.arr (scope ++ [toJson op.id]).toArray).compress
            nodes := View.append nodes (.element address tag attrs disabled event rendered.nodes)
            if tag == "button" then
              buttons := buttons ++ [(disabled, bindings.any (fun binding => binding.kind == .setEvent))]
            buttons := buttons ++ rendered.buttons
        | .setText =>
            if !textBinding rows op then
              let text <- display (<- evaluate context (<- Values.one rows op.id "text"))
              nodes := View.append nodes (.text text)
        | .branch | .slotOutlet =>
            let region <- if op.kind == .branch then selectedBranch rows context op.id
              else pure ((program.regions.find? (fun r => r.owner == some op.id)).map (·.id))
            if let some region := region then
              let rendered <- renderRegion program rows context fuel region scope
              for node in rendered.nodes do
                nodes := View.append nodes node
              buttons := buttons ++ rendered.buttons
        | .loop =>
            let some region := program.regions.find? (fun r => r.owner == some op.id)
              | throw "missing loop body"
            for (identity, context) in (<- Iteration.contexts program rows op context) do
              let rendered <- renderRegion program rows context fuel region.id (scope ++ [identity])
              for node in rendered.nodes do
                nodes := View.append nodes node
              buttons := buttons ++ rendered.buttons
        | .setProp | .setEvent => pure ()
        | _ => throw "unsupported stateful op"
      pure { nodes := nodes.reverse, buttons }

def observe (program : Program) (rows : List Operand) (context : Json) : Except String Rendered := do
  validate program rows
  renderRegion program rows context (program.regions.length + 1) 0

def render (program : Program) (rows : List Operand) (context : Json) : Except String Json := do
  pure (View.tree (<- observe program rows context).nodes)

end Impeto.Observation
