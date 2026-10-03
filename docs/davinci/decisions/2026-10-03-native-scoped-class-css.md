# Native scoped simple-class CSS and complete component emission

The native SFC path now consumes the genuine L1 CSS provider from #7558. Each
original scoped StyleView is parsed once during the existing private
NativeSfcObservation construction and retained beside the descriptor, original
script/template and genuine File. Styles are neutral to File membership; the
product decides whether their actual parser receipt is usable. The existing
move/into_parts handoff retains the CSS observation. No copied CSS source,
private raw selector scanner, normalized AST or legacy helper grants authority.

The family is exactly one complete simple-class rule per scoped CSS block with
flat declaration values. Multiple original scoped and plain blocks retain their
original order and optional CSS/trim/newline result contract. Each scoped block
requires a private simple-class receipt; only the exact original class-name token
end may receive generated `[scope_id]`. Escaped class names retain their original
spelling. Every remaining emitted original byte has its original file span;
inserted scope attributes and block-join newlines are generated and unlinked.
CSS maps retain the full original SFC and Unicode/CRLF coordinates. Recording
changes no emitted module/CSS bytes.

NativeSfcCompileOptions.scope_id accepts an optional full `data-v-*` identity
with a nonempty ASCII alphanumeric/underscore/hyphen suffix. The default uses
the established ordinary filename policy: DefaultHasher low 32 bits as eight
lowercase hex digits, prefixed with `data-v-`. The source-bound ordinary SFC
oracle checks the complete scoped CSS and its actual identity for the same
filename; ordinary compilation leaves component scope attachment to its Vite
consumer. No legacy semantic helper runs in normal code. A pinned default
`Scoped雪🌸.vue` fixture has `data-v-63f39f9f`, which is mounted through actual
Vue 3.5.35 with the same ID. Scope options affect scoped styles only.

The native and pinned Vue compiler preserve authored space before a rule's
opening brace. The ordinary Rust scoped printer trims that selector space.
Four separately pinned whole `ordinaryCss` results retain the exact dev-only
oracle, including both divergent whitespace cases; native/Vue expected CSS and
all original source/module/map contracts remain unchanged. No normalization or
weaker CSS-equivalence comparison replaces either complete byte contract.

L4 ScopeId validates CSS/JS-safe structure. ModuleParts.scope_id defaults to
None, preserving every existing complete module byte and link. The native
product supplies it after all styles are successfully emitted; assembly attaches
component.__scopeId after complete render/script fragments and before export.
Actual Vue DOM mounting then sets that attribute on the component's rendered
elements. No partial module or CSS is returned when any style/profile/option
is unavailable. Source-owned CSS parser errors, descriptor and File survive.

Four committed whole-SFC/full-module/CSS fixtures cover canonical scoping,
Unicode/CRLF, ordered scoped+plain blocks including empty trailing CSS, and
original setup with default scope identity. Rust compares each actual complete
result, independent Source Map v3 segments, original source pointers and custody
across moves. The pinned Vue compiler independently checks CSS output. Real
Vue DOM mount, DOM selector.matches, outside-component exclusion, setup update
and unmount laws execute complete native fixture modules in HappyDOM. The hosted
native runtime helper captures fresh Rust complete modules/maps/CSS and compares
them to those same exact source-bound runtime fixtures, then mounts those fresh
modules and retains runtime/source receipts as artifacts. Its dedicated action
runs in protected merge-group tooling shard 1 after CLI/dependency preparation,
independently of another native feature's pending PR. Local fixture runtime
checks are not relabelled as fresh Rust capture proof; hosted exact-head Actions
and protected full/all100 gates remain required.

At-rules, multiple rules per scoped block, empty scoped blocks, pseudos,
combinators, attribute selectors, nested rules/values/functions (including
escaped v-bind), Modules/preprocessors/external/custom input and unproven
boundaries remain typed refusals. CSS property semantics, broader scoped
selector families, complete bindings/Modules/preprocessors, SSR scoped output
and whole-product/history gates remain unfinished. Ordinary product routes,
legacy oracles, output policy, history gates and instruction budgets are unchanged.
