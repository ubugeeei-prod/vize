# Original L1 handler body handoff

Issues: [#6836](https://github.com/ubugeeei-prod/vize/issues/6836),
[#6838](https://github.com/ubugeeei-prod/vize/issues/6838),
[#6844](https://github.com/ubugeeei-prod/vize/issues/6844).

`NativeSyntax::into_handler_body` consumes only a genuine `Shape::HandlerBody`
observation into `RetainedHandlerBody`. The existing stock OXC
`HandlerBodyObservation` supplies the original body and privately borrowed
`AdmittedHandlerBody`; callers cannot substitute an AST, status, arena or
numeric source window. Raw immutable arena descendants remain alive after the
normally owned observation is dropped. Their lifetime does not grant parser
admission or native File/template custody.

The handoff retains the actual language and parse profile, original decoded
source/map, fixed wrapper prefix, complete comments and diagnostic metadata.
Only original authored descendants pass decoded/authored coordinate projection;
the generated FunctionBody container is not an authored span. Existing JS/TS
non-async handler lexical-context, wrapper, syntax and resource refusals remain
unchanged. A local hole keeps the original observations but exposes no admitted
body. Another grammar returns the original NativeSyntax intact. There is no new
parse, HTML decoding, traversal, serialization or pipeline stage.

Eight ordinary library laws cover original statement/directive/descendant and
comment pointers, raw-root lifetime, admission/profile identity, entity/Unicode
projection, TS syntax, complete diagnostics, context/wrapper/resource holes,
empty/comment-only results and intact rejection of other original shapes.
The private field rustdoc law guards observation substitution. Fresh exact-head
Actions and the protected full/instruction merge queue decide acceptance.

Remaining work is explicit: event Expr-versus-HandlerBody grammar selection,
resolution through the original File recorder with real local scopes and event
parameter binding, attached canonical `ui.on` construction, owner-bound L3 event
meaning, DOM emission and actual Vue runtime laws. The current native Component
pattern table still refuses events, and selected File root construction is a
separate provider. This API does not admit arbitrary AST/source pairs or replace
any product path. #6836, #6838, #6844 and compiler fix-history #6880 stay open.
