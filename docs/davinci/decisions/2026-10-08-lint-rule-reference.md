# Bilingual lint rule reference

Issue: [#6101](https://github.com/ubugeeei-prod/vize/issues/6101).

The rule reference covers every current Patina implementation with an individual English and
Japanese page. Each page includes purpose, severity, presets, applicable source, options, and Bad
and Good examples. The searchable index uses short metadata rows and links instead of code cells.
Vite+ integration via `@vizejs/vite-plugin/vite-plus` and `vp run lint` comes first; standalone
configuration remains a compatibility path.

Examples keep the original source context: imports, SFC blocks, filename-sensitive checks,
petite-vue HTML detection, type-aware prerequisites, and configured restrictions/design tokens.
The public linter executes each published pair in Rust tests. Good must avoid the specific finding
and parser diagnostics; unrelated rules can still report. Six type-aware pairs require the actual
Corsa Actions runtime. Generating and checking the reference requires source files, not a stale
local native binary. Generated EN/JA code blocks are identical.

The ESLint migration map retains all 252 pinned identifiers: 123 mapped rules, two intentional
divergences, and 127 unimplemented rules. Mapping does not promise identical findings, options,
or fixes. Unsupported rules remain with the original checker. Literal Vite+ configuration diffs
show where renamed rule IDs and `ruleOptions` belong.

Project diagnostics require complete graph context. Active cross-file findings document shared
files and exact Bad/Good changes. Published diagnostic contracts without a current producer are
identified explicitly; an illustrative risk/fix scenario must not claim that enabling a flag
produces that code. Typed Router examples include reachable router declarations and entry files.

Validation pending: exact-head Actions must execute the example pairs and build the rendered
English/Japanese reference before queue admission. The navigation companion owns the site-wide
menu and Vite+ onboarding; this change owns rule generation, content, and rule-specific coverage.
