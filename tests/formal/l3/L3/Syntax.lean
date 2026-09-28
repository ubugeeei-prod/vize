namespace Impeto

inductive Phase where
  | built
  | partitioned
  | scheduled
deriving Repr, DecidableEq

def Phase.parse (text : String) : Option Phase :=
  match text with
  | "built" => some .built
  | "partitioned" => some .partitioned
  | "scheduled" => some .scheduled
  | _ => none

inductive OpKind where
  | setProp
  | setDynamicProps
  | setText
  | setEvent
  | setHtml
  | setTemplateRef
  | insertNode
  | prependNode
  | directive
  | branch
  | loop
  | createComponent
  | slotOutlet
  | getTextChild
  | childRef
  | nextRef
deriving Repr, DecidableEq

def OpKind.parse (text : String) : Option OpKind :=
  match text with
  | "l3.set-prop" => some .setProp
  | "l3.set-dynamic-props" => some .setDynamicProps
  | "l3.set-text" => some .setText
  | "l3.set-event" => some .setEvent
  | "l3.set-html" => some .setHtml
  | "l3.set-template-ref" => some .setTemplateRef
  | "l3.insert-node" => some .insertNode
  | "l3.prepend-node" => some .prependNode
  | "l3.directive" => some .directive
  | "l3.if" => some .branch
  | "l3.for" => some .loop
  | "l3.create-component" => some .createComponent
  | "l3.slot-outlet" => some .slotOutlet
  | "l3.get-text-child" => some .getTextChild
  | "l3.child-ref" => some .childRef
  | "l3.next-ref" => some .nextRef
  | _ => none

inductive EdgeKind where
  | domOrder
  | effectOrder
  | dataDependency
deriving Repr, DecidableEq

def EdgeKind.parse (text : String) : Option EdgeKind :=
  match text with
  | "dom-order" => some .domOrder
  | "effect-order" => some .effectOrder
  | "data-dependency" => some .dataDependency
  | _ => none

structure Span where
  start : Nat
  stop : Nat
deriving Repr, DecidableEq

structure Region where
  id : Nat
  parent : Option Nat
  owner : Option Nat
  span : Span
deriving Repr, DecidableEq

structure Op where
  id : Nat
  kind : OpKind
  region : Nat
  effect : Option Nat
  span : Span
deriving Repr, DecidableEq

structure StateEdge where
  source : Nat
  target : Nat
  kind : EdgeKind
  effect : Option Nat
deriving Repr, DecidableEq

structure EffectScope where
  id : Nat
  owner : Nat
  region : Nat
  span : Span
deriving Repr, DecidableEq

structure Program where
  phase : Phase
  regions : List Region
  ops : List Op
  edges : List StateEdge
  effects : List EffectScope
deriving Repr, DecidableEq

def Program.empty : Program :=
  { phase := .built, regions := [], ops := [], edges := [], effects := [] }

end Impeto
