import Impeto.Syntax

namespace Impeto

inductive Backend where
  | vdom
  | vapor
deriving Repr, DecidableEq

def Backend.name : Backend -> String
  | .vdom => "vdom"
  | .vapor => "vapor"

structure TraceEvent where
  backend : Backend
  opId : Nat
  label : String
  effect : Option Nat
deriving Repr, DecidableEq

def TraceEvent.format (event : TraceEvent) : String :=
  let base := s!"{event.backend.name} op#{event.opId} {event.label}"
  match event.effect with
  | some id => base ++ s!" effect#{id}"
  | none => base

structure Machine where
  remaining : List Op
  trace : List TraceEvent
deriving Repr, DecidableEq

def interpretVDom : OpKind -> String
  | .setProp => "patch-prop"
  | .setDynamicProps => "patch-dynamic-props"
  | .setText => "set-text"
  | .setEvent => "patch-event"
  | .setHtml => "set-html"
  | .setTemplateRef => "set-template-ref"
  | .insertNode => "create-element"
  | .prependNode => "prepend-node"
  | .directive => "apply-directive"
  | .branch => "branch"
  | .loop => "iterate"
  | .createComponent => "create-component"
  | .slotOutlet => "render-slot"
  | .getTextChild => "get-text-child"
  | .childRef => "child-ref"
  | .nextRef => "next-ref"

def interpretVapor : OpKind -> String
  | .setProp => "assign-prop"
  | .setDynamicProps => "assign-dynamic-props"
  | .setText => "text-effect"
  | .setEvent => "listen"
  | .setHtml => "html-effect"
  | .setTemplateRef => "template-ref"
  | .insertNode => "create-node"
  | .prependNode => "prepend-node"
  | .directive => "directive-effect"
  | .branch => "conditional-effect"
  | .loop => "list-effect"
  | .createComponent => "component-effect"
  | .slotOutlet => "slot-effect"
  | .getTextChild => "get-text-child"
  | .childRef => "child-ref"
  | .nextRef => "next-ref"

def interpret (backend : Backend) (op : Op) : TraceEvent :=
  let label :=
    match backend with
    | .vdom => interpretVDom op.kind
    | .vapor => interpretVapor op.kind
  { backend, opId := op.id, label, effect := op.effect }

def step (backend : Backend) (machine : Machine) : Option Machine :=
  match machine.remaining with
  | [] => none
  | op :: remaining =>
      some { remaining, trace := machine.trace ++ [interpret backend op] }

def runOps (backend : Backend) : List Op -> List TraceEvent -> List TraceEvent
  | [], trace => trace
  | op :: remaining, trace => runOps backend remaining (trace ++ [interpret backend op])

def run (backend : Backend) (program : Program) : List TraceEvent :=
  runOps backend program.ops []

def referenceTrace (program : Program) : List String :=
  ((run .vdom program) ++ (run .vapor program)).map TraceEvent.format

end Impeto
