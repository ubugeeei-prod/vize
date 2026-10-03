# Original handler input and reference coordinates

Tracks #6836, #6838 and #6844. Depends on the genuine retained HandlerBody
provider and its original static-event operand ancestry, not a reparsed
expression or a caller-selected AST/source pair.

L2 now normally owns the complete original `RetainedHandlerBody` in
`HandlerInput`. Only the original stock no-hole admission can construct this
input. The original arena FunctionBody, directives, statements, intrinsic
JS/TS profile, parser prefix, source, decode map, comments and complete ordinary
diagnostics remain intact. Movable inputs neither clone the body nor manufacture
a Program, expression or synthetic script unit. The minimal `NativeHandlerInput`
receiver instead retains the complete genuine `NativeAttributeHandler`, keeping
its private original event association beside the same complete handler body.
Its short join still requires the actual original selection and attribute;
siblings and independent equal-byte parses cannot substitute. Failure returns
the complete original event operand without discarding that authority.

The crate-private `HandlerReferenceSource` derives its body and coordinates only
from the original owner, or from the sealed input that already retained that
admission. The shared reference-coordinate dispatch now removes the actual
generated prefix and checks the complete decoded UTF-8 window. Authored
projection uses the same original EmbedSource and map. The generated body
container and delimiters are not authored ranges; partial entity boundaries
remain precise source errors. Numeric projection alone grants no descendant
membership or semantic observation.

Every refusal returns the original owner, including its local hole, comments,
complete diagnostic envelopes and parser/source profile. The conservative L1
resource refusal remains a refusal even for otherwise valid source.

Six general whole-library laws cover original descendant/directive pointer custody
across owner movement and arena lifetime, original profile/comment identity,
independent wrapper/entity/Unicode spans, generated and partial-entity
boundaries, complete diagnostic envelope retention, empty/comment-only input
and original resource refusal. Three native laws additionally cover one-pass
original header custody, foreign/sibling refusal and original event holes.
Three compile-fail laws deny public AST/source substitution and owner duplication.
Actions and the protected merge queue decide delivery; no instruction ceiling changes.

Remaining work is genuine single-walk handler scopes, local declarations,
implicit event parameters and reference resolution; authentic original header
and File association; attached ui.on construction; L3 meaning; and native DOM
runtime emission. Consuming a syntax-only owner does not retain the selected
attribute authority carried separately by NativeAttributeHandler. The actual
NativeHandlerInput retains that event owner for its future File receiver and
original-header join. Current native lowering/emission still refuse events.
No product route, legacy helper or #6880 closure changes.
