# Pinned OXC formatter source

The source and MIT license were copied unchanged from official OXC commit
`fc702c1fa9f0412d06ec6908b58cd395b826cf7f` (oxc_formatter 0.60.0):
https://github.com/oxc-project/oxc/tree/fc702c1fa9f0412d06ec6908b58cd395b826cf7f/crates/oxc_formatter

The source-only import is a separate commit. Reproduce it with:

`node tools/support/dependencies/vendor-oxc-formatter.ts VERIFIED_OXC_CLONE`

The standalone manifest gives this unofficial fork the package name
`vize_oxc_formatter` and retains the Rust library name `oxc_formatter`.
Like the original formatter, its formatter core and Vize's glyph crate, it is
unpublished. Official CLI/native releases build it as a workspace dependency.
It depends directly on the existing Vize parser; every supporting OXC crate
retains the original revision and shared AST/allocator/span identity.
The adjacent LICENSE retains the complete original MIT notice. The local
rustfmt configuration and crate-level Clippy policy preserve upstream style.
Upstream fixture-generator build dependencies are omitted because the upstream
fixture tree is not imported; Vize's public API/CLI and differential suites
exercise the fork.

The formatter behavior change backports official OXC commit
`7f350d166ff5b3d460c27c2eac7dd28123e7ba8c`:
https://github.com/oxc-project/oxc/commit/7f350d166ff5b3d460c27c2eac7dd28123e7ba8c

Four private files share a printed-state-independent comment predicate. When
unary grouping already retains comments around a binary/logical operand, that
operand omits its duplicate parentheses and duplicate indent group. Vize
[#7929](https://github.com/ubugeeei-prod/vize/issues/7929) retains both complete
original conditions, LF/CRLF variants, whole outputs and three public passes.
There is no output-text correction or additional parse stage.

The local adaptation retains an independent group around the flattened operand
body, so a required nested logical group does not inherit a trailing comment's
forced line break. The public control retains its independently authored whole
output, also confirmed with Prettier 3.9.6; inline operand comments and long
chains still break where required.

## Typed last-argument arrows

The additional private `print/call_like_expression/arguments.rs` change is the
exact 44-line removal from official OXC commit
`ca1ba71595d175da6b3b44b6100e09bfaa66f8e1`:
https://github.com/oxc-project/oxc/commit/ca1ba71595d175da6b3b44b6100e09bfaa66f8e1

Remove the obsolete type-reference return annotation veto from existing arrow
argument grouping; retain body-based grouping and every supporting dependency
revision. Vize [#7868](https://github.com/ubugeeei-prod/vize/issues/7868) retains
the full original program, LF/CRLF carriers and whole independent references,
with three public passes and real CLI checks and writes.

## Parsed directive call layout

The local `lib.rs` entry `format_unary_call_argument` formats the retained call
argument of one unary-expression statement. It accepts the existing
`parse_for_format` result and keeps the Program, ExpressionStatement and
UnaryExpression parents in the same arena, including their comment spans.
There is no new parser pass or output-text rewrite. Vize
[#7876](https://github.com/ubugeeei-prod/vize/issues/7876) supplies the actual
attribute-value indentation budget and retains sixteen complete independent
Oxfmt references, leading/trailing/argument comment and raw-literal controls,
LF/CRLF cases, three public passes and full CLI check/write streams. This
special-purpose entry is a local adaptation rather than an upstream backport.
