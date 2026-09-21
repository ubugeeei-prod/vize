import Impeto.LoopBehavior
import Impeto.Model

/-!
Stateful runner for native `v-model` scenarios. Rendering and element identity
come from the proved update machine. Each form control's live DOM state is keyed
by that identity: it is created by the mounted hook, patched by update hooks
only on a reactive change, and mutated by the same event steps the mounted Vue
runner performs. Observations add the live `value`, `checked` and `selected`
properties exactly where the mounted observer reads them.
-/

namespace Impeto.ModelBehavior
open Lean

structure Runner where
  program : Program
  rows : List Operand
  controls : List (Nat × Model.Control)
  context : Json
  state : Incremental.State := {}
  forms : List (Nat × Model.Live) := []

def opOf (address : String) : Except String Nat := do
  let parts <- (<- Json.parse address).getArr?
  let some last := parts.back? | throw "empty S3 address"
  last.getNat?

partial def controlNodes (runner : Runner) : Incremental.Node -> Except String (List (Nat × Model.Control))
  | .text _ => pure []
  | .element address identity _ _ _ _ children => do
      let own := match runner.controls.lookup (<- opOf address) with
        | some control => [(identity, control)]
        | none => []
      pure (own ++ (<- children.flatMapM (controlNodes runner)))

def liveControls (runner : Runner) : Except String (List (Nat × Model.Control)) :=
  runner.state.roots.flatMapM (controlNodes runner)

/-- Re-render through the incremental machine, then run mount/update hooks. -/
def rerender (runner : Runner) (assigning : Option Nat := none) : Except String Runner := do
  let fresh <- Observation.observe runner.program runner.rows runner.context
  let state <- Incremental.update (runner.program.regions.length + 2) runner.state fresh.nodes
  let runner := { runner with state }
  let mut forms := []
  for (identity, control) in <- liveControls runner do
    let model <- Iteration.lookup runner.context control.path
    let live <- match runner.forms.lookup identity with
      | some live => Model.update control live model (assigning == some identity)
      | none => Model.mount control model
    forms := forms ++ [(identity, live)]
  pure { runner with forms }

/-- Reactive assignments trigger only when `Object.is` would see a change. -/
def changed (old new : Option Json) : Bool :=
  match old, new with
  | some a, some b =>
      match Expression.strictEqual a b with
      | .ok same => !same
      | .error _ => true
  | _, _ => true

partial def setPath (value : Json) (fields : List String) (next : Json) : Except String Json := do
  match fields with
  | [] => pure next
  | [field] => do let _ <- value.getObj?; pure (value.setObjVal! field next)
  | field :: rest => do
      let inner <- value.getObjVal? field
      pure (value.setObjVal! field (<- setPath inner rest next))

def assign (runner : Runner) (control : Model.Control) (next : Json) : Except String (Runner × Bool) := do
  let old := (Iteration.lookup runner.context control.path).toOption
  let context <- setPath runner.context (control.path.splitOn ".") next
  pure ({ runner with context }, changed old (some next))

def transform (control : Model.Control) (text : String) : Except String Json := do
  let text <- if control.trim then Model.trimJs text else pure text
  if control.number then Model.looseToNumber text else pure (.str text)

/-- Selector subset: `tag` or `tag[value=...]`, first match in document order. -/
def selects (selector : String) (node : Incremental.Node) : Bool :=
  match node with
  | .text _ => false
  | .element _ _ tag attrs _ _ _ =>
      match selector.splitOn "[value=" with
      | [name] => name == tag
      | [name, rest] =>
          let wanted := (rest.dropEnd 1).toString.replace "\"" ""
          rest.endsWith "]" && name == tag && attrs.lookup "value" == some (.str wanted)
      | _ => false

partial def find (selector : String) : Incremental.Node -> List Incremental.Node
  | node@(.element _ _ _ _ _ _ children) =>
      (if selects selector node then [node] else []) ++ children.flatMap (find selector)
  | .text _ => []

def event (runner : Runner) (step : Json) : Except String Runner := do
  let fields <- Behavior.keys step
  if fields.any (!["event", "selector", "value", "checked", "selectedValues"].contains ·) then
    throw "unsupported model interaction field"
  let name <- (<- step.getObjVal? "event").getStr?
  let selector <- (<- step.getObjVal? "selector").getStr?
  let some (.element _ identity _ _ _ _ _) := (runner.state.roots.flatMap (find selector)).head?
    | throw "missing interaction target"
  let some control := (<- liveControls runner).lookup identity | throw "target is not a model control"
  let some live := runner.forms.lookup identity | throw "control has no live state"
  let mut live := live
  if let .ok value := step.getObjVal? "value" then
    let value <- value.getStr?
    match control.kind with
    | .text => live := { live with value }
    | .select =>
        if control.multiple then throw "multiple selects take selectedValues"
        let first := control.options.findIdx? (· == value)
        live := { live with selected := control.options.zipIdx.map (fun (_, i) => first == some i) }
    | _ => throw "value steps apply to text inputs and single selects"
  if let .ok checked := step.getObjVal? "checked" then
    if ![Model.Kind.checkbox, .radio].contains control.kind then throw "checked applies to toggles"
    live := { live with checked := <- checked.getBool? }
  if let .ok values := step.getObjVal? "selectedValues" then
    if !(control.kind == .select && control.multiple) then throw "selectedValues needs a multiple select"
    let values <- (<- values.getArr?).toList.mapM Json.getStr?
    live := { live with selected := control.options.map values.contains }
  let mut assigned := none
  match control.kind, name with
  | .text, "input" =>
      if !control.lazy && !live.composing then assigned := some (<- transform control live.value)
  | .text, "change" =>
      if control.lazy then assigned := some (<- transform control live.value)
      if control.trim then live := { live with value := <- Model.trimJs live.value }
      if !control.lazy && live.composing then
        live := { live with composing := false }
        assigned := some (<- transform control live.value)
  | .text, "compositionstart" => if !control.lazy then live := { live with composing := true }
  | .text, "compositionend" =>
      if !control.lazy && live.composing then
        live := { live with composing := false }
        assigned := some (<- transform control live.value)
  | .checkbox, "change" =>
      match live.lastModel with
      | .arr items =>
          -- `looseIndexOf` then `splice(index, 1)`: only the first loose match leaves.
          let target := Json.str control.domValue
          let mut index := none
          for (item, i) in items.toList.zipIdx do
            if index.isNone && (<- Model.looseEqual item target) then index := some i
          match live.checked, index with
          | true, none => assigned := some (.arr (items.push target))
          | false, some i => assigned := some (.arr (items.eraseIdx! i))
          | _, _ => pure ()
      | _ => assigned := some (.bool live.checked)
  | .radio, "change" => assigned := some (.str control.domValue)
  | .select, "change" =>
      let chosen := (control.options.zip live.selected).filter (·.2) |>.map (·.1)
      let chosen <- chosen.mapM (fun value =>
        if control.number then Model.looseToNumber value else pure (.str value))
      if control.multiple then assigned := some (.arr chosen.toArray)
      else match chosen with
        | [value] => assigned := some value
        | _ => throw "single select change without one selection"
  | _, _ => throw s!"unsupported {name} event on a model control"
  let runner := { runner with forms := runner.forms.map (fun (id, state) =>
    (id, if id == identity then live else state)) }
  let some next := assigned | return runner
  let (runner, didChange) <- assign runner control next
  if !didChange then return runner
  rerender runner (if control.kind == .select then some identity else none)

/-- Observation JSON with live form properties, where the mounted observer reads them. -/
partial def nodeJson (runner : Runner) (pairs : List (Nat × Model.Control)) (selection : List Bool) :
    Incremental.Node -> Json × List Bool
  | .text value => (.str value, selection)
  | .element _ identity tag attrs disabled _ children =>
      let live := (runner.forms.lookup identity).getD {}
      let control := pairs.lookup identity
      let (selection, own) := match tag, selection with
        | "option", selected :: rest => (rest, [("selected", Json.bool selected)])
        | _, _ => (selection, [])
      let (kids, _) := children.foldl (fun (acc, sel) child =>
        let (json, sel) := nodeJson runner pairs sel child
        (acc ++ [json], sel)) (([] : List Json), if tag == "select" then live.selected else [])
      let form := match tag, control with
        | "input", some control => [("value", Json.str (Model.displayValue control live)),
            ("checked", Json.bool live.checked)]
        | "select", some control => [("value", Json.str (Model.displayValue control live))]
        | "button", _ => [("disabled", Json.bool disabled)]
        | _, _ => []
      (Json.mkObj ([("tag", Json.str tag), ("attributes", Json.mkObj attrs),
        ("children", .arr kids.toArray)] ++ own ++ form), selection)

def snapshot (runner : Runner) : Except String Json := do
  let pairs <- liveControls runner
  let tree := runner.state.roots.map (fun node => (nodeJson runner pairs [] node).1)
  pure (Json.mkObj [("tree", .arr tree.toArray), ("events", .arr #[])])

def insideLoop (program : Program) (region : Nat) : Nat -> Bool
  | 0 => true
  | fuel + 1 =>
      match program.regions.find? (·.id == region) with
      | some { parent := some parent, owner := some owner, .. } =>
          program.ops.any (fun op => op.id == owner && op.kind == .loop) ||
            insideLoop program parent fuel
      | _ => false

def run (program : Program) (rows : List Operand) (script : Json) : Except String Json := do
  if (<- Behavior.keys script) != ["context", "steps"] then throw "unsupported model scenario fields"
  let context <- script.getObjVal? "context"
  Behavior.validateState context
  Observation.validate program rows
  let mut controls := []
  for op in program.ops.filter (Model.modelRow rows) do
    let control <- Model.control program rows op
    let some target := (Values.forOp rows op.id).findSome? (·.target) | throw "untargeted model"
    let some element := program.ops.find? (·.id == target) | throw "missing model target"
    if insideLoop program element.region (program.regions.length + 1) then
      throw "model controls inside loops are outside the reference subset"
    controls := controls ++ [(target, control)]
  let mut runner <- rerender { program, rows, controls, context }
  let mut trace := [<- snapshot runner]
  for step in (<- (<- script.getObjVal? "steps").getArr?).toList do
    let fields <- Behavior.keys step
    if fields == ["patch"] then
      let patch <- step.getObjVal? "patch"
      Behavior.validateState patch
      let mut dirty := false
      let mut context := runner.context
      for (name, value) in (<- patch.getObj?).toList do
        dirty := dirty || changed (context.getObjVal? name).toOption (some value)
        context := context.setObjVal! name value
      runner := { runner with context }
      if dirty then runner <- rerender runner
    else if fields.contains "event" then runner <- event runner step
    else throw "unsupported model step"
    trace := trace ++ [<- snapshot runner]
  pure (.arr (trace ++ [Json.mkObj [("tree", .arr #[]), ("events", .arr #[])]]).toArray)

end Impeto.ModelBehavior
