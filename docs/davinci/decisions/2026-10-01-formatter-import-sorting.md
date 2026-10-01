# Script import sorting (#7258)

Pass the Oxfmt-shaped `sortImports` object to the pinned Oxc formatter for every
script stabilization pass. The default remains disabled, preserving historical
formatting bytes. The opt-in policy applies to ordinary/setup and JSX/TSX SFC
scripts and standalone script files; native/WASM entry points support it too.

Keep the published exhaustive `FormatOptions` and `FormatterConfig` structs
unchanged. A private raw formatter wrapper retains the new setting, and an
additive ConfigDocument projection / host loader reads it in the existing parse.
A new formatter builder and standalone script function carry the validated Oxc
options. Validate incompatible sorting controls before any CLI source write.

The Vite+ helper inherits Oxfmt's `fmt.sortImports` and allows
`fmt.vize.sortImports` to override it, including explicit false. Its Oxfmt config
is retained. Pkl, generated TypeScript, JSON Schema, and native declarations
expose the option. Move script/CLI dispatch and schema definitions in separate
move-only commits to keep source-size limits and concurrent fixes intact.

Acceptance uses authored source and whole-byte references, idempotence, custom
groups/newline boundaries, descending/internal groups, safe side-effect ordering,
comment/newline partitions, CLI check/write/no-config/disabled behavior, invalid
config write protection, a native-binding full reference, and Vite+ inheritance
and false-override tests. Required Actions and merge-group validation remain
pending; no provider is promoted and no fixture/performance budget is relaxed.
