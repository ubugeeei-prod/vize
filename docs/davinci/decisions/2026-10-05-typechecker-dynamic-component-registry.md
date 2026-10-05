# Dynamic component registry checks (#8003)

The original [P0 report #8003](https://github.com/ubugeeei-prod/vize/issues/8003)
shows CLI and editor TS2339 on `__vize_dynamic_is_N` for the built-in
`<component :is="'button'">` whenever `checkUnknownComponents` is enabled.
Its full issue bytes are retained locally; original body SHA-256 is
`987008b0a555afa2ede3d82ad418c54c08ea06d22e2cb3f509842c3085c7276b`.
Reporter `ubugeeei` / GitHub ID71201308 is verified from the live issue API.
The independent source starts at actual main
`66a9b15639c828b17d5f655b3dd58f697799b3fa`; it does not modify the separate
configured Batch-session/publication repair, helper materialization, releases,
old CPU campaign or its source/driver/raw profiles.

Croquis gives non-identifier dynamic component expressions an inference alias.
The existing template scope declares that alias and checks its props/events.
`GlobalComponentPlan::emit` incorrectly checks the same synthetic name against
Vue's global registry and emits a second module-scope fallback declaration.
The repair excludes only an alias proved to belong to its actual authored
`<component>` element with a bound `is` expression. Proof combines the exact
Croquis alias derived from the usage offset and the existing AST locator;
matching a name prefix alone is insufficient. Authored internal-looking tags,
including an exact alias/offset collision, remain registry checked. The AST
locator also declines `:is` on an ordinary authored component tag.
The private plan constructor receives the matched facts and AST together;
the existing generator layout and oversized-file line count stay unchanged.

This changes the existing generator, with no new product stage, checker API,
helper declaration, ambient global, diagnostic filter or `any` fallback.
Dynamic value/name resolution, constructor inference, props checks and source
mapping stay in their existing paths. Complete generated code and projection
mapping are compared with the default option for literals, conditionals,
lookups, calls, refs/spreads, independent same-element loops, unknown expression
reads and constructor-union props. Static unknown tags keep authored registry
mappings across Unicode/CRLF and synthetic-looking names remain real checks.

The legacy fixture retains the original App.vue, tsconfig and editor config.
The existing runtime tooling suite exercises the real CLI and stdio LSP with
explicit true, false and absent `checkUnknownComponents`; each live session
moves through original/expanded dynamic cases, a real missing component,
an authored internal-looking tag, an unresolved dynamic expression, a bad
known-component prop and repair. Complete CLI and diagnostic publication
objects are retained before assertions. Expected counts, codes, order, authored
starts, severity and editor source are checked; clean cases require empty
complete arrays. No new normalization or message-deletion path is added.
The existing always-upload corpus action retains these objects under
`target/differential/typechecker-dynamic-component-8003/receipt.json`.
Existing full differential and native-runtime gates remain required.

Validation is source preparation only: rustfmt parsing, configured TypeScript
format/lint and pure inventory checks run locally. No local native/Rust build,
provider installation or old-binary outcome qualifies this change. Fresh
exact-source Actions must prove generator laws and the real native 7.0.2
CLI/editor runtime cases. Ordinary, full-source and protected queue results are
reported separately; a failing candidate is removed. Actual signed merge and
published product release are still pending and #8003 stays open until the
fix actually merges. Every instruction ceiling remains unchanged.

The same decision is recorded on #8003 and in the canonical record in this
slice. Include the verified reporter as a Co-authored-by trailer. Independent
technical peer review qualifies only the frozen source; it does not transfer
historical CPU, runtime, merge or release acceptance.
