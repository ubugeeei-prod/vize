# Component registration uses explicit project context (#8142)

The public 0.440 consumer authenticated by the official Darwin collector
revealed nine additional authored missing-component controls that
`vue/require-component-registration` silently accepted. Unknown lowercase
names, guessed web-component prefixes and unconditional Router/Nuxt names
were excluded without project evidence. Independent original Vue ESLint
10.9.2 packets diagnose those names. Separate complete positive controls show
that actual imports, explicit globals, built-ins and native HTML/SVG/MathML
must remain accepted.

Four additional public-consumer controls isolate the custom-element handoff:
the default tag and an unmatched neighbor warn, while explicitly configured
literal `my-widget` and glob `my-*` should be accepted. Actual public 0.440
still diagnosed both configured custom elements. Each research campaign used
the complete original 51-rule projection, repeated original provider packets,
three actual CLI routes and unchanged authenticated public payloads. These
are new authored controls, not edited historical n8n packets.

The focused source boundary reuses the existing shared native-tag classifier
and `vize_relief::options::CustomElementMatcher`. Private `Linter` state owns
the matcher; the template context borrows it immutably for one lint pass. No
new parser, analyzer, project evaluation, cache or pipeline stage is added.
The matcher is moved through an additive builder and borrowed without cloning
its pattern vector. Public `RequireComponentRegistration` fields and
`LintRuleOptions` struct literal compatibility stay intact.

Registration exemptions require actual setup/value imports, Options API
registrations, a recursive component name, explicit global names, Vue
built-ins, compiler custom-element policy, or an explicitly selected Nuxt
auto-import mode. The known prefix and framework-global guesses are removed.
Existing filename/self and setup-binding semantics are retained rather than
silently broadened. `globals` still uses its existing component-name matching.

Original Vue ESLint exposes `ignorePatterns` as JavaScript regular
expressions. Native global names and compiler custom-element literal/glob
patterns have different semantics and configuration shapes. Independently
observed original positive options are retained as reference controls; this
slice does not introduce a guessed regex compatibility layer or claim
complete option-surface parity.

Thirty whole authored SFC controls have complete diagnostic expectations.
The Rust judge records all 120 observations before comparison, covering both
matcher/global builder orders and repeated runs. Additional source controls
cover native and built-in tags, Unicode physical spans, static matcher
predicates and unmatched neighbors, eight complete JSX/TSX fallback packets,
explicit Nuxt mode, and rule selection.
The original #7979 fixture bytes remain unchanged; its positive Router case
now explicitly selects the global it previously assumed.
The repository gallery now explicitly imports its existing Vue Router components;
all template bytes and the oversized component line counts remain unchanged.

CLI and per-document LSP transfer must derive the matcher from the already
loaded `ConfigDocument` snapshot using
`document.clone().into_compiler_custom_elements()`. Calling a separate config
loader would reevaluate project configuration and is rejected. Shared loader
and LSP snapshot paths currently belong to the config delivery lane; this
integration waits for their genuine sealed ancestry or an explicit
coordinated handoff. It is unfinished until whole actual CLI and JSON-RPC
positive/negative context vectors pass.

Fresh exact-head Actions, the strict instruction ceilings, protected queue
and actual merge remain required before source delivery. Installed public
acceptance requires a later genuinely published source cut and its own
single official collector receipt. Public 0.440/0.441 research is not later
product credit. This change does not complete the remaining full n8n
adoption, typed diagnostics, default-RHS lookup, entity output or physical
coordinate gaps.

The two preserved kebab-filename self-reference cases remain native-clean while
original Vue ESLint diagnoses their Pascal names. These known historical
differences are retained in the complete reference packets; the source slice
does not claim native/reference self-name equivalence.
