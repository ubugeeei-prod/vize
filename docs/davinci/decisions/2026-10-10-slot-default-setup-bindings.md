# Inline setup bindings in slot default expressions (#8142)

Two original typed slot runtime laws fail because a literal `fallback` declared
in script setup becomes `_ctx.fallback` inside the slot parameter default.
The declaration is lexical in an inline render function; the public component
proxy does not contain that binding. The original initial slot is empty, its
first click and saved initial callback record undefined, and the later payload
still updates normally. Both complete original compiler lanes exhibit this
preexisting error.

The bounded correction borrows the existing Core `CodegenContext` and native
`PrefixScope`. After the unchanged local-declaration and allowed-global checks,
a default reference remains lexical only in inline mode when its existing
metadata is `LiteralConst` or `SetupConst`. Object shorthand uses the same
predicate. Every other metadata kind, missing metadata and non-inline mode keep
the previous decision. This is preservation of those neighboring policies,
not completion of their wider semantics.

Core keeps the parent's single binding-name set, sorted name vector and retained
formal-parameter visitor. The native lane keeps its existing guarded wrapper
parser and existing constant-binding policy. There is no new parse, wrapper,
compiler pipeline stage, clone of metadata, public option, cache or hidden AST
storage. The independent Croquis raw boundary remains unchanged. This change
does not claim universal parse-once ownership.

The independent 24 complete parameter strings predate this source change and
cover positive constant reads, non-inline/absent metadata, all neighboring
binding kinds, local parameters, nested shadows, globals and shorthand.
Each source unit retains all whole contexts, inputs and results before judging
all 24 strings. The six original TypeScript SFC inputs and 12 independently
authored desired runtime laws remain byte-exact. Their historical classification
preserves all original sources, comparisons, error packets and unfinished laws.

The source Actions campaign captures all 12 complete SFC Results, descriptor
and counter debug data before launching its actual Node child. The runtime uses
the unchanged checksum-pinned Vue 3.5.26 loader and the already locked TypeScript
6.0.3 consumer. It retains every original and transformed module, source map,
diagnostic, initial/update/unmount tree, click record, saved callback, function
identity, framework error, host call and actual process status before judging
any desired runtime law. It authenticates Node/native/TypeScript bytes and the
actual Rust-to-Node parent PID relationship before and after execution.

Static source preparation is not execution credit. Current source Actions,
unchanged strict instruction ceilings, original adoption cohorts, protected
actual merge and a later included published consumer must qualify separately.
The current 0.440/0.441 public receipts do not contain this correction. Entity
physical coordinates, other default binding kinds, complete #8142 adoption,
project registration routes and unrelated editor/compiler gaps stay unfinished.
