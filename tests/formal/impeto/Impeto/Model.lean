import Impeto.Iteration

/-!
Native `v-model` reference semantics for P3-4 (`ui.model` in S3). A model
binding is a `set-prop` whose binding kind is `model`. It carries identical
`model-read`/`model-write` paths and `model-attribute` rows for the element kind
and the `lazy`/`trim`/`number` modifiers. This module is written from Vue's
documented directive contract, not from runtime code. It covers text inputs
with IME composition, checkboxes (boolean and array membership), radios, and
single or multiple selects with static options. Live DOM state (value, checked,
selected, composing) belongs to the element and survives re-renders. Directive
hooks rewrite it only as the contract says. Anything else fails closed.
-/

namespace Impeto.Model
open Lean

inductive Kind where
  | text | checkbox | radio | select
deriving BEq, Repr

structure Control where
  kind : Kind
  path : String
  lazy : Bool := false
  trim : Bool := false
  number : Bool := false
  multiple : Bool := false
  domValue : String := "on"
  options : List String := []

/-- DOM-owned state of one control. `selected` is per static option. -/
structure Live where
  value : String := ""
  checked : Bool := false
  selected : List Bool := []
  composing : Bool := false
  lastModel : Json := .null
deriving BEq

def modelRow (rows : List Operand) (op : Op) : Bool :=
  op.kind == .setProp && (Values.forOp rows op.id).any (fun row =>
    row.role == "binding-kind" && row.kind == "literal" && row.text == "model")

def staticAttr (rows : List Operand) (id : Nat) (name : String) : Option Operand :=
  (Values.forOp rows id).find? (fun row => row.role == "attribute" && row.name == some name)

def optionValues (program : Program) (rows : List Operand) (select : Op) : Except String (List String) := do
  let [region] := program.regions.filter (fun r => r.owner == some select.id)
    | throw "select requires one option region"
  let mut values := []
  for op in program.ops.filter (fun op => op.region == region.id) do
    if op.kind != .insertNode || (<- Values.literal (<- Values.one rows op.id "tag")) != "option" then
      throw "select children must be static options"
    let some value := staticAttr rows op.id "value" | throw "option requires a static value"
    values := values ++ [<- Values.literal value]
  if values.isEmpty || values.eraseDups.length != values.length then
    throw "select options must be distinct"
  pure values

/-- Validate one model binding and extract its static control description. -/
def control (program : Program) (rows : List Operand) (binding : Op) : Except String Control := do
  let operands := Values.forOp rows binding.id
  if operands.any (fun row => !["name", "model-read", "model-write", "model-attribute",
      "binding-kind"].contains row.role) then throw "unsupported model operand"
  if (<- Values.one rows binding.id "name").kind != "absent" then throw "unsupported model argument"
  let read <- Values.one rows binding.id "model-read"
  let write <- Values.one rows binding.id "model-write"
  if read.kind != "js" || write.kind != "js" || read.text != write.text || !Iteration.path read.text then
    throw "model requires one assignable path"
  let some target := program.ops.find? (fun op => some op.id == read.target && op.kind == .insertNode)
    | throw "model target must be an element"
  if operands.any (fun row => row.target != some target.id) then throw "malformed model target"
  let attrs := operands.filter (·.role == "model-attribute")
  let flag := fun (name : String) => attrs.any (fun row => row.name == some name && row.kind == "absent")
  if attrs.any (fun row => !["element-kind", "lazy", "trim", "number"].contains (row.name.getD "")) ||
      (attrs.map (·.name)).eraseDups.length != attrs.length then
    throw "unsupported model modifier"
  let elementKind <- Values.literal (<- (attrs.find? (·.name == some "element-kind")).elim
    (throw "missing model element kind") pure)
  let tag <- Values.literal (<- Values.one rows target.id "tag")
  if elementKind != tag then throw "model element kind disagrees with its tag"
  let typeAttr <- (staticAttr rows target.id "type").elim (pure "text") Values.literal
  let domValue <- (staticAttr rows target.id "value").elim (pure "on") Values.literal
  let kind <- match tag, typeAttr with
    | "input", "text" => pure Kind.text
    | "input", "checkbox" => pure .checkbox
    | "input", "radio" => pure .radio
    | "select", _ => pure .select
    | _, _ => throw "unsupported model control"
  let modifiers := ["lazy", "trim", "number"].filter flag
  if kind != .text && !(kind == .select && (modifiers == [] || modifiers == ["number"])) &&
      !modifiers.isEmpty then throw "modifiers apply only to text and select controls"
  if kind == .text && (staticAttr rows target.id "value").isSome then
    throw "text models must not also carry a static value"
  let multiple := match staticAttr rows target.id "multiple" with
    | some row => row.kind == "absent"
    | none => false
  let options <- if kind == .select then optionValues program rows target else pure []
  let result : Control := {
    kind := kind
    path := read.text
    lazy := flag "lazy"
    trim := flag "trim"
    number := flag "number"
    multiple := multiple
    domValue := domValue
    options := options
  }
  pure result

def jsWhitespace (c : Char) : Bool := c == ' ' || c == '\t' || c == '\n' || c == '\r'

/-- Characters JavaScript treats as whitespace but this reference does not model. -/
def exoticSpace (c : Char) : Bool :=
  c.toNat == 0x0B || c.toNat == 0x0C || c.toNat == 0xA0 || c.toNat == 0x1680 ||
    (c.toNat >= 0x2000 && c.toNat <= 0x200A) || [0x2028, 0x2029, 0x202F, 0x205F, 0x3000,
      0xFEFF].contains c.toNat

def trimJs (text : String) : Except String String := do
  if text.toList.any exoticSpace then throw "unsupported whitespace in model value"
  pure (String.ofList ((text.toList.dropWhile jsWhitespace).reverse.dropWhile jsWhitespace).reverse)

/-- `looseToNumber`: `parseFloat`, keeping the string when it is `NaN`. -/
def looseToNumber (text : String) : Except String Json := do
  let chars := (<- trimJs text).toList
  let (negative, rest) := match chars with
    | '-' :: rest => (true, rest)
    | '+' :: rest => (false, rest)
    | _ => (false, chars)
  if rest.take 8 == "Infinity".toList then throw "unsupported infinite model number"
  let digits := rest.takeWhile Char.isDigit
  let after := rest.dropWhile Char.isDigit
  let (fraction, after) := match after with
    | '.' :: tail => (tail.takeWhile Char.isDigit, tail.dropWhile Char.isDigit)
    | _ => ([], after)
  if digits.isEmpty && fraction.isEmpty then return .str text
  if fraction.any (· != '0') then throw "unsupported fractional model number"
  if let c :: tail := after then
    if (c == 'e' || c == 'E') && (tail.dropWhile (fun d => d == '+' || d == '-')).head?.any Char.isDigit then
      throw "unsupported exponent in model number"
  let magnitude : Int := (String.ofList digits).toNat?.getD 0
  Expression.number (if negative then -magnitude else magnitude)

/-- `looseEqual` over primitives; structured values fail closed. -/
def looseEqual (a b : Json) : Except String Bool := do
  pure ((<- Expression.primitiveText a) == (<- Expression.primitiveText b))

/-- The DOM spelling of a text model value (`value == null ? '' : value`). -/
def textOf (model : Json) : Except String String :=
  if model == .null then pure "" else Expression.primitiveText model

def checkedFor (control : Control) (model : Json) : Except String Bool := do
  match control.kind, model with
  | .checkbox, .arr items => items.toList.anyM (looseEqual · (.str control.domValue))
  | .checkbox, .bool b => pure b
  | .checkbox, _ => throw "checkbox models must be booleans or arrays"
  | .radio, _ => looseEqual model (.str control.domValue)
  | _, _ => pure false

def selection (control : Control) (model : Json) : Except String (List Bool) := do
  if control.multiple then
    let .arr items := model | throw "multiple selects require array models"
    let texts <- items.toList.mapM Expression.primitiveText
    pure (control.options.map texts.contains)
  else
    let mut found := none
    for (option, index) in control.options.zipIdx do
      if found.isNone && (<- looseEqual (.str option) model) then found := some index
    pure (control.options.zipIdx.map (fun (_, index) => found == some index))

/-- Mounted hook: initialize DOM state from the model. -/
def mount (control : Control) (model : Json) : Except String Live := do
  match control.kind with
  | .text => pure { value := <- textOf model, lastModel := model }
  | .checkbox | .radio => pure { checked := <- checkedFor control model, lastModel := model }
  | .select => pure { selected := <- selection control model, lastModel := model }

/-- Update hooks after a re-render; `assigning` suppresses select resync. -/
def update (control : Control) (live : Live) (model : Json) (assigning : Bool) : Except String Live := do
  let previous := live.lastModel
  let live := { live with lastModel := model }
  match control.kind with
  | .text =>
      if live.composing then return live
      let current <- if control.number && !(live.value.startsWith "0" &&
          (live.value.drop 1).toString.front.isDigit) then looseToNumber live.value else pure (.str live.value)
      let desired := if model == .null then .str "" else model
      if (← Expression.strictEqual current desired) then return live
      pure { live with value := <- textOf model }
  | .checkbox => pure { live with checked := <- checkedFor control model }
  | .radio =>
      if (← Expression.strictEqual previous model) then return live
      pure { live with checked := <- checkedFor control model }
  | .select => if assigning then pure live else pure { live with selected := <- selection control model }

def displayValue (control : Control) (live : Live) : String :=
  match control.kind with
  | .text => live.value
  | .checkbox | .radio => control.domValue
  | .select => ((control.options.zip live.selected).find? (·.2)).elim "" (·.1)

end Impeto.Model
