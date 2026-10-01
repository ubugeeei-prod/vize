# Native ForHead: proposed first bounded slice

This is a private implementation proposal for
[#6836](https://github.com/ubugeeei-prod/vize/issues/6836), pending review. It
depends on the retained-expression handoff and checked source pieces; it does
not claim ForHead support or change any product route.

The original [embed design](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929)
requires one L1 parse per language piece, authoritative source and OXC syntax.
The current strict repository splitter in `vize_l1_to_l2/src/lower/vfor.rs`
selects the first whitespace-delimited `in`/`of` before JS parsing. It works
over untrimmed text, so ` in xs` contains an empty authored alias. A complete
head such as `a in b in c` must never become a JS `in` expression tree.

Vue's [current compiler parser](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/parser.ts)
parses the collection as an ordinary expression and nonempty aliases as formal
parameters. Its wrapper for parameter syntax is an arrow. This confirms that
object/array destructuring and defaults require binding syntax, not object/array
expression syntax. Vue's independently stripped parenthesis compatibility is a
separate policy; the proposed first slice preserves Vize's existing strict
paired-parenthesis rule and does not claim full upstream/version equivalence.

## Carrier and ownership

The Vue dialect produces checked decoded-relative ranges for the collection
and every authored alias position. The language provider receives only each
checked `EmbedSource` piece and its selected language. It never receives HTML
or parses the composite head. Empty positions retain exact zero-width source
points; the first three positions have value/key/index roles, and additional
authored positions remain retained. No synthetic placeholder parameter is made.

The collection uses the actual `RetainedExpression` handoff. Each nonempty alias
is parsed once with SlotParams context and must expose exactly one formal
parameter or rest binding. The per-position carrier must retain the true OXC
binding pattern, comments, full owned diagnostics, coordinates and local hole.
The existing expression handoff cannot substitute for this carrier. Before a
native L2 consumer receives these references, add a safe consuming SlotParams
handoff that moves the existing authored parameter/rest roots or their arena
slice out of the generated arrow. Preserve descendants and observations, drop
diagnostics normally, and expose neither generated parameter-list containers nor
generated arrow/body nodes as authored syntax.

Per-piece observation owners contain complete Diagnostics with ordinary Drop.
Any composite collection of those owners must also retain ordinary destruction;
it cannot park them in drop-free arena storage. Binding/root references and
checked coordinate records may live separately in the shared arena.

The composite keeps its original checked source and each exact checked piece.
Use `slice_in`; never decode a piece again or derive authored offsets by adding
decoded lengths. A partial entity cut becomes a typed source-boundary hole.
Whole-head finite work admission must precede per-piece parses, as well as the
existing 31-unit admission over each actual wrapped parser input. Otherwise an
unbounded list of tiny aliases can bypass the per-parse budget. No parsing or
language work is repeated in L2 or L4.

## First partition scope

Use the repository's first viable textual separator and strict paired outer
parentheses. For alias positions, retain all comma-separated positions with a
checked bracket stack and escaped single/double-quoted strings. Detect slash,
backtick and generic-angle ambiguity outside strings before producing parsed
positions and retain them as typed unsupported lexical forms until a complete
lexical provider exists. This is an explicit bounded subset: slash covers
comments/regex/division defaults; backticks cover interpolation nesting; angle
forms include TS generic commas. A naive scanner misclassifies
`a // comment, b` as two aliases even though both separate parameter fragments
could parse successfully. Per-piece syntax success does not prove partition
correctness.

Collections continue through the real Expr provider, including supported
comments/regex/templates and JS/TS forms within current admission. Alias
positions retain empty, syntax, admission and unsupported holes independently.
The first slice does not claim complete lexical grammar, identifier resolution,
file language selection, ForOp lowering, FilterChain or older Vue repeat syntax.

## Native laws before publication

- Real v-for fixtures: value/index over number, string, array and Map; three
  positions over object/record; TS sources `({} as T)`, `TArr` and `TIter`.
- `a in b in c` retains source `b in c`, and `of`, Unicode whitespace and
  leading whitespace preserve checked authored/decoded positions.
- Missing/empty/sparse positions and more than three aliases are never removed,
  collapsed, renamed or replaced by synthetic AST parameters.
- Nested object/array patterns, defaults with quoted commas and JS/TS context
  preserve true binding roots; source/entity projections remain exact.
- Comments, regex, templates and generic commas are typed unsupported until
  their lexical partition laws exist; they never expose invented positions.
- Each successful root and descendant is the original OXC node; complete
  observations and roots survive ownership transfer without reparse/clone.
- Whole-head and actual-wrapper admission bound work and keep all authored
  source for holes. Syntax/module-context failure remains local to its piece.

The paired central decision and issue comment belong in the eventual approved
source change. This proposal alone neither resolves the TODO nor enters a queue.
