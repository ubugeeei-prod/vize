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
  | "impeto.set-prop" => some .setProp
  | "impeto.set-dynamic-props" => some .setDynamicProps
  | "impeto.set-text" => some .setText
  | "impeto.set-event" => some .setEvent
  | "impeto.set-html" => some .setHtml
  | "impeto.set-template-ref" => some .setTemplateRef
  | "impeto.insert-node" => some .insertNode
  | "impeto.prepend-node" => some .prependNode
  | "impeto.directive" => some .directive
  | "impeto.if" => some .branch
  | "impeto.for" => some .loop
  | "impeto.create-component" => some .createComponent
  | "impeto.slot-outlet" => some .slotOutlet
  | "impeto.get-text-child" => some .getTextChild
  | "impeto.child-ref" => some .childRef
  | "impeto.next-ref" => some .nextRef
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
