# Original strict literal receipt

Issues: #6836, #6838 and #6840. Original syntax admission intentionally does not
run semantic early-error analysis. Real native setup modules containing `010`,
`08`, `\\1` or `\\8` were accepted by the inherited Let/Var and additive Const
family, then actual Node 24 rejected those unchanged strict-module spellings.
Const #7521 was removed from the protected queue before this repair.

The original lexer records one private boolean while it already decodes legacy
numeric literals and legacy octal/non-octal decimal string escapes. Both
checkpoint paths retain it and rewind restores it; speculative lookahead cannot
supply an actual input fact. Existing committed unambiguous await reparsing
retains the completed original tail receipt across its selective rewind. The complete original ParserReturn privately keeps
that bit, and the immutable admitted Program can query it. Raw caller Programs
cannot manufacture the observation or the borrowed admission. No new source
scan, parse, AST walk, allocation, table or serialization is introduced.

Ordinary ASTs, syntax diagnostics and existing admission remain unchanged.
Modern zero, radix prefixes, Unicode, null, hex and Unicode escapes, escaped
backslashes and nondecimal identity escapes remain legal. This is only a literal
spelling receipt; it does not certify general strict semantics, lexical binding
names, TS erasure, target emission or product completion. The consumer must join
it to authentic source/profile/Program custody and normal File completion.

All 74 focused locked Cargo parser unit tests passed, including actual
invalid/valid original decoder, both checkpoint/rewind and committed selective
await reparse tail-retention laws. Parser library Clippy also passes. Exact-head Actions, unchanged instruction
ceilings, protected Stack candidate checks and actual merge remain required.
The child consumer will join the receipt into existing setup eligibility and
reject normalized strict binding names in its sole declaration walk, retaining
neutral File facts and original owners. It will preserve whole source-built
module/refusal captures and mandatory Node strict-module checks in the actual
merge-group tooling runner. TS broadening remains paused until this repair is
accepted; default compiler and #6880 remain unchanged.
