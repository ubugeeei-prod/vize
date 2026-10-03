# Original handler scope and reference resolution

Tracks #6838 and #6844. Depends on the actual original HandlerBody and
NativeAttributeHandler providers, and their normally-owned L2 input receivers.

`resolve_handler` consumes a genuine NativeHandlerInput, retaining the entire
original event/body owner beside its bounded semantic facts. One Resolver
instance walks the actual authored directives, statements and initializers;
the existing 64-depth/4096-node budget covers the entire body. The generated
FunctionBody container is not projected. No source decoding, parsing, additional
AST pass, synthetic script unit or AST/source pair is introduced.

The walk records declarations, lexical block scopes and pending references.
A flat fact resolution then resolves nearest lexical bindings, forward local
references and function-scoped var hoisting. Handler-local identities remain
distinct from caller-supplied enclosing BindingIds. The actual original event
operand alone establishes the implicit `$event` parameter; a generic body or
external lookup cannot substitute for that event provenance. Generated root
scopes and implicit parameters have no fabricated authored declaration span.

Repeated var declarations preserve one identity while retaining each original
declaration site. Var/lexical collisions are checked in both declaration orders,
including nested declaration sites, while disjoint lexical shadows remain
independent. Root lexical `$event` conflicts refuse; var reuse and nested block
shadowing remain supported. Forward lexical resolution establishes binding
identity only, not TDZ/read validity or execution success.

Supported statements are empty, expression, return, simple var/let/const,
blocks and if. The existing expression family preserves read/write/read-write,
escaped shorthand and constructor-callee facts. Functions/classes, loops,
destructuring, dynamic eval, TS declarations/annotations and other unfinished
forms remain precise refusals. The complete original native owner returns on
syntax/scope/resolution failure; no partial table escapes. Intrinsic TS input
retains its original profile, with unsupported annotation ranges preserved.

Fifteen actual whole-library laws cover parameter/outer identity separation,
forward lexical references, shadowing and var hoisting, original declaration
sites, both collision orders, if branches, semantic failure custody, empty
sources, Unicode/entity/escape maps, source-relative usage and intrinsic TS
admission/refusal. A compile-fail law denies table replacement. Production
Clippy and an independent peer source review validate the bounded subset;
fresh exact-head Actions and protected Stack/queue acceptance decide delivery.

Remaining work: authentic File lookup/visibility and original header/cursor
association, attached ui.on construction, complete handler statement/type
families, L3 meaning and native DOM runtime emission. Caller-provided enclosing
identities confer no File association, product completion or runtime execution
proof. Current native lowering/emission still refuse events. No legacy helper,
default product route, numeric instruction ceiling or #6880 closure changes.

The literal-main For resolver and original alias payload providers are preserved
beside the genuine handler source family. Their combined registrations reach
352 lines in the shared walk. A separate move-only commit relocates existing
`export_local` verbatim into a private sibling and retains its private reexport,
checkpoint/rollback and shared 4096/64 budget; the shared walk is now 338 lines.
No additional parse, walk, allocation, stage or public authority is introduced.
Fresh hosted exact-head and protected queue checks validate this composition.
