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

The only formatter behavior change backports official OXC commit
`7f350d166ff5b3d460c27c2eac7dd28123e7ba8c`:
https://github.com/oxc-project/oxc/commit/7f350d166ff5b3d460c27c2eac7dd28123e7ba8c

Four private files share a printed-state-independent comment predicate. When
unary grouping already retains comments around a binary/logical operand, that
operand omits its duplicate parentheses and duplicate indent group. Vize
[#7929](https://github.com/ubugeeei-prod/vize/issues/7929) retains both complete
original conditions, LF/CRLF variants, whole outputs and three public passes.
There is no output-text correction or additional parse stage.
