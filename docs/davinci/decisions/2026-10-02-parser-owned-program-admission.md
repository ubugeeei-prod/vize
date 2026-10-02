# Parser-owned native Program admission

Tracked with [#6836](https://github.com/ubugeeei-prod/vize/issues/6836).
This provider builds on the actual ordinary Program and pinned parser repair.
The real `const value = /x/uv;` case leaves a recovered AST and a nonfatal
parser diagnostic. A raw Program, matching source and an independent language
walk cannot establish that its original parse was clean.

The pinned parser now provides `Parser::parse_observed()` only for the three
built-in `NoTokensParserConfig`, `TokensParserConfig` and `RuntimeParserConfig`
types. One private helper snapshots the
actual private source/profile/options and calls the existing `parse()` once.
`ProgramObservation` privately owns the entire original `ParserReturn`,
including diagnostics, comments, tokens, module record and irregular
whitespace. The existing `parse()` result and implementation remain unchanged.
The supporting official OXC AST/allocator/span identities remain unchanged.

Only `ProgramObservation::admitted()` can mint `AdmittedProgram`. It rejects
panicked, Flow and actual error-diagnostic results. The short owner borrow
exposes shared Program, original source, requested profile and actual options;
there is no public constructor, mutable Program view, raw-result conversion
or unchecked extraction. This is syntax admission, not semantic validation.
Shared structural syntax does not freeze OXC's semantic node/scope ID Cells;
consumers must establish their own semantic identities.
`ParseOptions` gains equality so a consumer can reject nondefault options
without reconstructing feature-dependent parser policy. An Unambiguous input
keeps that original requested identity even if its AST infers Module.

L1 retains this whole ordinary-drop observation for Program only and exposes
its private borrowed capability through `NativeSyntax::admitted_program()`.
The original source, typed hole, actual comments and complete diagnostic views
remain available when no capability is admitted. Wrapped Expr/HandlerBody/
SlotParams paths keep their existing parser result, guards and consuming
handoff. A non-Expr `into_expression` returns the whole owner intact.
No parser or semantic pass, AST clone, stage, serialization or legacy normal
dependency is added.

The six original real laws cover nonfatal `/uv` denial with retained Unicode
comments/source/diagnostics, exact clean AST/source borrow identity, actual
nondefault options, Unambiguous identity, fatal and Flow observations.
Compile-fail laws reject constructing either private owner/proof from raw
public data and obtaining mutable Program access through a live capability.
The existing PURE recovery laws now inspect the same retained observation's
comments directly. Current wrapped handoff and Program laws remain active.

A supported `benchmarking` configuration reproduced an authority violation:
a safe custom byte handler called `advance_to_end()` and returned `Kind::Eof`.
The invalid `const broken = ;` source then acquired a clean empty Program,
full source span, default options and explicit Module identity through the
initial generic observation method. The ordinary parser reports an error.
Custom configurations retain the existing generic `parse()` API, but have no
`parse_observed()` method. There is no caller-controlled trusted marker,
runtime identity flag or new parse. Three added built-in configuration laws
retain exact options/profile/comments and reject actual `/uv` errors; a fourth
compile-fail law refuses even a custom configuration delegating the built-in
lexer. The original supported-feature skip-source reproducer now fails with
E0599, while that custom configuration's ordinary parse remains compatible.

Scoped matching-cache validation checks the actual source modules with the
same pinned supporting AST/span/diagnostic packages. It is not fresh complete
workspace or production acceptance. Hosted full output, formatter, native
Program fuzz and strict100 gates remain required on the final source head.
The consuming L2 file provider must take this capability directly, reject
nondefault/nonexplicit native profiles and retain its existing whole-source
guard; its declaration/use walk must not parse again. That consuming repair
is a genuine dependent slice. No product path replacement is made here.
