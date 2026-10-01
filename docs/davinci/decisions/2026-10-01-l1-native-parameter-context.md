# L1 native formal-parameter Await context

This source decision repairs a concrete retained SlotParams context gap for
[#6836](https://github.com/ubugeeei-prod/vize/issues/6836). It keeps the published
expression/source handoff unchanged and is a separate provider prerequisite for
the private dense Vue ForHead consumer.

The pinned OXC 0.142.0 parser at
`fc702c1fa9f0412d06ec6908b58cd395b826cf7f` inherits the module Await setting
while parsing a parenthesized arrow head. It resets that setting only for the
arrow body. Consequently `x=await y` produces an AwaitExpression and no parser
diagnostic inside the generated non-async SlotParams arrow. Parser success does
not prove valid formal-parameter context; exposing that binding was incorrect.

The existing retained-AST module-context visitor now also tracks entry into
FormalParameters. It saves and restores the flag, resets it while walking a
FunctionBody, and refuses AwaitExpression while the flag is set. All function
parameter lists are covered in the same walk. Nested async bodies such as
`x=async()=>await y` remain valid, nested async parameter defaults remain
invalid, and a later outer parameter restores the original parameter context.
No second walk, parse, semantic artifact, dependency or pipeline stage is added.

`InvalidParameterContext` is a precise local hole. Generated/recovered parameter
roots stay private, but the original checked source, comments, complete owned
Diagnostics and checked coordinate views remain available with ordinary Drop.
An empty OXC diagnostic owner is kept empty; the provider does not fabricate an
OXC error. Parser-diagnosed reserved/escaped Await and new.target cases retain
all original diagnostic messages and corrected labels. Module-context refusal
and its precedence remain unchanged.

Real native laws check JS and TS defaults, computed binding keys, plain and
escaped Await, property names spelled await, nested async bodies/parameters,
later-default flag restoration, handler nested parameters, original file
coordinates, comment pointers and complete diagnostic ownership. The existing
handler lexical/module laws continue to run. The scoped pinned-OXC module
harness contains 54 passing source/admission/shape/handoff/context laws; it is
not evidence of a full current-main Cargo build. Exact-head Actions and strict
instruction ceilings remain publication gates.

Full grammar/admission, JSX/TSX, file-language selection, sparse ForHead,
FilterChain, identifier resolution and native product routes remain unfinished.
