# withDefaults macro ownership

Paired issue: [#7962](https://github.com/ubugeeei-prod/vize/issues/7962), with
[the implementation decision](https://github.com/ubugeeei-prod/vize/issues/7962#issuecomment-5995769323).
The inspected original baseline is actual main
`d8cd6a208b9aea02150b140a4b81b87222128a51`.

## Correction

`script/no-with-defaults` previously searched every script's raw bytes for
`withDefaults(`. Both original setup comments/string literals and both
ordinary `.ts` files therefore received findings without a macro call.
The rule now consumes the dispatcher's shared JS/TS Program and checks the
actual parsed SFC context's `is_sfc` and `is_script_setup` flags.
Offsets, filename extensions and macro names do not manufacture setup
ownership.

Only top-level expression statements and non-ambient variable initializers
can own this macro. Their direct, non-optional `withDefaults` call must
have a direct, non-optional `defineProps` call as its first argument.
Existing AST APIs unwrap top-level TypeScript assertions and preserved
parentheses without traversing nested functions. This follows the
[Vue 3.5.42 setup statement processing](https://github.com/vuejs/core/blob/v3.5.42/packages/compiler-sfc/src/compileScript.ts)
and [macro call shape](https://github.com/vuejs/core/blob/v3.5.42/packages/compiler-sfc/src/script/defineProps.ts).
Comments, strings, templates, regexes, member/prefixed names, nested
function calls, ordinary scripts and inline HTML scripts cannot supply
this ownership. There is no blanket binding scan; an unrelated nested
same-name binding cannot hide an actual outer macro.

The rule joins the existing at-most-one shared Program parse per script
block. When it is the only enabled script rule, it requires that AST parse
instead of the former byte-only scan; it never starts a rule-local second
parse or adds a semantic pass, pipeline stage or serialization. Original
identifier spans use the existing script offset. Rule name, description,
warning message, full help, default severity and public docs path remain
unchanged. Recognized macro owners with runtime props or missing defaults
still receive the finding; this preference rule does not validate Vue
compilation or accept invalid macro arguments.

The official migration inventory adds one raw OXC source row and three
test rows for the existing AST/parser/L0 APIs. Every prior row remains
unchanged; these are honest legacy consumer facts, not native admission or
an instruction-budget change.

## Complete original inputs and controls

`tests/_fixtures/differential/lint-with-defaults/` retains the whole
original issue body and every byte of its config, `SizeLabel.vue`,
`comment-only.ts` and `source-check.ts`. Source length/hash pins protect
those files and fourteen complete authored controls. Cases cover actual
typed, whitespace/comment-separated, parenthesized, TS-wrapped,
expression-statement, escaped-identifier and multi-declarator macro owners;
invalid runtime/missing-default arguments; ordinary nested/member/optional
expressions; non-setup SFC/module/HTML owners; a genuine outer macro beside
a nested same-name binding; and dual scripts with Unicode and CRLF.

Five Rust laws compare complete ordered diagnostic vectors, result counts,
filename, original spans, severity, message, full help, labels and fixes.
The existing positive unit source and complete block-local snapshot stay
byte-exact, now using an actual parsed setup descriptor. Whole-file laws
independently check the physical original script frame.

The public CLI corpus requires the existing exact source-build receipt and
retains full source, raw stdout/stderr and failure/status evidence. Two
fresh processes per case compare all seventeen complete JSON reports,
including error severity from the original config, scalar columns and
unchanged help/docs path. The whole original three-file plain command must
return zero with the complete clean report. Existing shared forty-four
history inputs/oracles, native admission and broader #6881 qualification
remain unchanged; `nativeHandled` remains zero.

## Delivery

The first Draft #8046 head `bd2aca7c` reached hosted compilation in
unused-bindings run37320718513/job111798771532, then failed with two E0308
errors in the existing unit's new setup helper: actual SFC content is
`Cow<str>`, while Parser and rule APIs need `&str`. The complete original
112756-byte raw failure has SHA256
`75cf3bb31c81b82428f334a7d7a44672bd7f18954fb365328a3b55fe93dfde76`.
[The ordinary correction](https://github.com/ubugeeei-prod/vize/issues/7962#issuecomment-5996028321)
borrows the original content with `Cow::as_ref()` in exactly two test
arguments, without copying, changing production or weakening any law.
All original unit/corpus/golden bytes and expected fields remain intact;
no new Rust-law execution is credited to the failed head.

Authored laws and source review do not establish execution. Fresh exact
head automatic Actions must execute the complete Rust and public CLI
contracts. The finite release hold keeps this independent PR Draft and off
the merge queue. Protected unchanged 104 instruction gates/full suites,
actual signed reporter-credited merge and released public payloads remain
separate requirements. Broader native linter/history replacement and the
10x goal remain unfinished. The verified reporter is `ubugeeei`, GitHub ID
`71201308`, with the literal verified noreply Co-Author trailer in the
meaningful source commit.
