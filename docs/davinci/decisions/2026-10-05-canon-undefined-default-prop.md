# Optional callable props with undefined defaults

Issue: [#7819](https://github.com/ubugeeei-prod/vize/issues/7819).

The parser retains the exact authored `undefined` default expression. A later
imported-default resolver also wrote the string `undefined` into prop metadata
as a marker for a proven non-undefined default. The template model treated
every retained default as defined, overriding the existing literal-undefined
filter and adding both a `Pick<Props, ...>` default and an `Exclude<..., undefined>`
assertion. Conditions on optional functions then produced false TS2774.

Retain the existing resolver's proven default-name set separately from authored
expressions, using its already-built set without another source scan. Template
default substitution uses those names and actual non-undefined authored
defaults. Macro bindings consume the same normalized set. A literal undefined
default therefore preserves the callable's optional type, while a real
imported callable default still resolves as defined. The change repairs the
current product and claims no native-stage acceptance.

The original SFC and tsconfig are pinned under
`tests/_fixtures/differential/typechecker/undefined-default-prop/`.
Source-built CLI tests compare complete diagnostic vectors for the original
valid conditions, genuinely defaulted imported callable calls, and rejected
unguarded calls on an undefined default. Independent typed macro/template
statements use the installed Vue declarations with official TypeScript 7.0.2;
all native stdout/stderr and statuses are compared with authored expectations.
The real editor diagnostic service must return the complete empty vector,
and the real native hover service must retain the exact optional callable
signature and authored range.

The native-phase Actions explicitly requires these CLI and editor tests,
retaining full input/output, official package/version and source/binary hash
receipts. Ordinary PR Rust's native-disabled runs are insufficient. Fresh
exact-head source/native Actions, unchanged protected full suites and
instruction ceilings, actual merge and published consumer proof are required.
Queue admission remains under the release owner's publication barrier.

The first required run failed closed before product checks because workspace root
has no direct Vue dependency. Create isolated cases beneath the existing pinned
`npm/cli` workspace package, so both independent native oracle and source CLI
resolve its unchanged real Vue declarations without installation or symlinks.
Pin the editor runtime to the same authenticated oracle binary, retain complete
editor fixture/config inputs and full responses before assertions. This repairs
qualification only; original corpus and all expected diagnostics remain intact.
