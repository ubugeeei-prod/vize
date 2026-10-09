# Opinionated withDefaults defaults

Paired issue: [#8338](https://github.com/ubugeeei-prod/vize/issues/8338), with the
[same decision comment](https://github.com/ubugeeei-prod/vize/issues/8338#issuecomment-6078517721).
The inspected production baseline is actual main
`f46c40071c78b191f382956a88aef1bef7380b16`; delivery is rebased onto its
verified successor `2bcc327c57e38c6b5d9709a158390ebce8457b7a`.

## Decision

Register the existing `script/no-with-defaults` rule with
`OPINIONATED_SCRIPT_PRESETS` only. That constant contains `opinionated`;
the existing `strict` and `all` aliases select the same preset. Warning
severity, explicit rule overrides, and the shared AST macro classifier
remain unchanged. Generic `happy-path`, Nuxt and all other presets retain
their previous behavior.

The existing authored-source Actions step builds the current CLI and invokes
`lint --no-config --preset opinionated --max-warnings 0`. The rule now joins
that existing gate; no additional source scanner or workflow is needed.
Generated EN/JA rule metadata is regenerated with the existing docs command.
The two touched reference pages retain its Markdown hard breaks using the
same narrow Git whitespace attribute convention as the generated indexes.
No generator, diagnostic oracle, pipeline stage or performance ceiling changes.

## Authored SFC defaults

Musea `ActionsPanel`, `TokenPreview`, `TokenCard`, `TokenCategorySection`
and `DevtoolsTracePanel` use Vue 3.5 reactive props destructuring defaults.
Required and optional prop declarations, template bindings and behavior are
preserved. Array/object default expressions remain per instance; computed
and function accesses observe prop replacements. The omitted category level
still uses its existing fallback instead of acquiring a new prop default.

The source-backed runtime laws read those five complete SFCs and compile
their actual setup through the installed Vue compiler. A headless Vue renderer
checks prop replacement, computed results, scalar defaults, mutable default
ownership and default restoration. Existing non-Vue helpers and child UI are
isolated; these laws make no native/Vize compiler or template runtime claim.

## Regression evidence

With the original empty preset registration, the new actual preset law fails:
`opinionated` produces no finding instead of Warning at original bytes 39–51.
After registration, all 18 preset laws pass. The membership snapshot gains
exactly one opinionated entry. All five original macro ownership integration
laws pass; their source, expected diagnostics and fixtures are unchanged.

The actual original-registration CLI also exits successfully with no findings
for the pinned macro source under `--preset opinionated --max-warnings 0`.

The pinned legacy CLI corpus contains 13 complete cases, each run twice in a
fresh process with a source-built binary receipt. It compares whole JSON
reports, Warning severity, original identifier locations, help text and the
existing warning-limit failure. Default/other presets, aliases, reactive
destructuring, comments, strings and ordinary helpers are explicit controls.
This is CLI corpus coverage, not admission to the separate shared 44-case
API/native linter pack. Compiler compatibility fixtures and Bad examples
remain unchanged, as does the ordinary rich-text helper named `withDefaults`.

Local setup runtime coverage passes 5/5 with Vue/compiler-sfc 3.5.41,
TypeScript 6.0.3 and Node v25.8.1. Fresh exact-head source Actions, protected
performance qualification and actual merge are still required before delivery
is complete. This independent style fix does not delay publication of the
already-merged third-party compiler and Musea fixes.
