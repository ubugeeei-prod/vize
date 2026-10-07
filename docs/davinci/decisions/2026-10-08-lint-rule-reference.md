# Bilingual lint rule reference

Issue: [#6101](https://github.com/ubugeeei-prod/vize/issues/6101).

The rule reference covers every current Patina source catalog entry with an individual English and
Japanese page. Each page includes purpose, severity, presets, applicable source, options, and Bad
and Good examples. The searchable index uses short metadata rows and links instead of code cells.
Vite+ integration via `@vizejs/vite-plugin/vite-plus` and `vp run lint` comes first; standalone
configuration remains a compatibility path.

Category pages are compact indexes of the individual references, so outdated duplicate examples
do not contradict qualified cases. Previously authored category examples are retained as small
generator input modules; source-comment examples still read their implementations directly.

Examples keep the original source context: imports, SFC blocks, filename-sensitive checks,
petite-vue HTML detection, type-aware prerequisites, and configured restrictions/design tokens.
The public linter executes each published pair in Rust tests using the CLI configuration path
(with_additional_rules), preserving opt-in registration. Good must avoid the specific finding
and parser diagnostics; unrelated rules can still report. Four SFC catalog entries are explicitly
marked as non-emitted (the empty Vapor attribute callback and three unsupported header checks),
and their tests assert that current boundary. Invalid attribute spelling documents the actual
parser/template diagnostic before the defensive rule, rather than claiming a second finding. Six type-aware pairs require the actual
Corsa Actions runtime, with a targeted invocation in the existing PR native-phases lane. Generating and checking the reference requires source files, not a stale
local native binary. Generated EN/JA code blocks are identical.

The ESLint migration map retains all 252 pinned identifiers: 123 mapped rules, two intentional
divergences, and 127 unimplemented rules. Mapping does not promise identical findings, options,
or fixes. Unsupported rules remain with the original checker. Literal Vite+ configuration diffs
show where renamed rule IDs and `ruleOptions` belong.

Project diagnostics require complete graph context. Active cross-file findings document shared
files and exact Bad/Good changes. Published diagnostic contracts without a current producer are
identified explicitly; an illustrative risk/fix scenario must not claim that enabling a flag
produces that code. Typed Router examples include reachable router declarations and entry files. The 60 cross-file
codes comprise 19 CLI surfaces (18 source pairs and one tracked-flow scenario), 16 library-only
producers, and 25 reserved contracts. The async-no-suspense producer reads macro async facts,
while source parsing currently records async on the script-setup scope; its full source pair is
explicitly non-emitted and the actual CLI test asserts that boundary. The 25 complete project
pairs stay exercised (24 Bad findings plus that one non-emitted pair); none are silently dropped.

Runtime qualification also exposed exact source prerequisites: provide/inject types need explicit
annotations, uncaught-error scans template expressions, and hydration-risk currently uses the
prop-to-ref producer rather than an unconstructed non-reactive-watch variant. The emits validator
example uses a block-body arrow that passes the current SFC prefilter; method shorthand dispatch
remains implementation follow-up work. No production behavior or allowlists change here. Genuine signed main 8f01ff9c is incorporated;
the newly shipped component-registration globals are retained in both option references, generated
rule configuration, and the public runtime test. All 14 typed rule-option entries are documented.

An independent module-grammar audit caught six inherited source-comment examples with duplicate
bindings, template markup inside a script, or a return outside a function. The declaration,
async-computed, reactive-destructuring, useId, and useSlots references now show one complete
alternative per SFC. The existing Babel TypeScript parser checks every Good script in tooling;
the public-linter Bad/Good assertions remain strict and still require actual rule findings.
Five further inherited examples repeated defineProps or referenced undefined Props/Emits types.
They now use one macro context and locally defined types. The already pinned Vue 3.5 oracle checks
all Good SFC script contexts; this validates authored examples and makes no claim about Vize
compiler output or parity. No new dependencies or product behavior are introduced.

Validation pending: exact-head Actions must execute the example pairs and build the rendered
English/Japanese reference before queue admission. The navigation companion owns the site-wide
menu and Vite+ onboarding; this change owns rule generation, content, and rule-specific coverage.
