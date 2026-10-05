# P0 delivery ledger (2026-10-05)

The maintainer requested continuing work until Vize has zero P0 issues. This
ledger records the original open set and delivery rules. It is a snapshot,
not a claim that any unexecuted source repair is complete.

## Audit and priority

- Source main: `66a9b15639c828b17d5f655b3dd58f697799b3fa`.
- Live audit: 198 open issues, including 108 labelled `priority:p0`.
- All open issues in this audit were authored by the maintainer. This does not
  exclude third-party or real-project reports relayed in their issue bodies.
  Prioritize those reports by impact and provenance, not account ownership.
- The separate merge ledger has 62 actual PR merges. The published baseline
  remains v0.432.0; the next finite release is not yet published.
- Public delivery coordination: [#6239](https://github.com/ubugeeei-prod/vize/issues/6239).
- Paired decision: [2026-10-05 maintainer objective](https://github.com/ubugeeei-prod/vize/issues/6239#issuecomment-5993465687).

## Operational checkpoint: first v0.433.0 admission HOLD

- Decision effective: 2026-10-05 13:10 UTC, superseding earlier admission thaws.
  [Root publication checkpoint](https://github.com/ubugeeei-prod/vize/issues/6239#issuecomment-5991644254)
  and [paired ledger decision](https://github.com/ubugeeei-prod/vize/issues/6239#issuecomment-5995446480).
- Authenticated readback at 13:22 UTC: actual main
  `d8cd6a208b9aea02150b140a4b81b87222128a51`, 102 open P0 issues and the
  coordinator's 69-actual-PR merge checkpoint. #8029 merged at 13:16:12 UTC.
  The complete 108 original rows below remain a byte-exact historical snapshot.
- Only qualified bottom Stack prefix #7857 remains admitted: source
  `9e7ea14327915b3eb65d2b68947f508cb7fa9599`, protected candidate
  `92e036de6f8dc651440e7c9a2f415a3c7fa69890`, position 1, awaiting checks.
  Its protected terminal checks and actual merge are still required.
- After that prefix actually merges and the healthy queue drains, root alone
  immediately resumes supported release #7811 from literal current main.
  Private/source preparation and exact-source Actions continue; additional
  queue or Ready-auto admissions await verified publication and explicit THAW.
  This ledger-only Draft and its source CI never delay the first finite cut.
  Bulk child #8038 and private performance work remain outside that cut.
- #7811 is still Draft; the v0.433.0 Release API returned 404 at this checkpoint.
  All six fresh candidate gates, atomic main/tag identity, GitHub assets,
  npm/crates/editor visibility, deployed docs and installed-payload replay of
  original fixtures remain required. Historical `42a` gates stay historical.
  Source-merged fixes are not yet claimed publicly delivered; unresolved
  reports, P0 zero, 10x and wider Davinci/fix-history work remain unfinished.

## Delivery rules

1. Retain the complete reported input and independently authored expected
   behavior. Copy retained `.vue.txt` bytes into real `.vue` inputs for E2E.
2. Use isolated worktrees and independent implementation/review lanes. Carry
   genuine source dependencies in a registered ordered GitHub native Stack;
   keep independent changes separate. Use existing Actions for actual checks.
3. Current-head source tests, protected candidate checks, actual signed merge,
   and released public payloads are separate evidence. Queue entry or enabled
   auto-merge is not delivery. Remove only a proven current failed candidate.
4. Include the issue reporter as Co-Author and pair decisions in the issue and
   the canonical decision record. Preserve incoming decisions and source caps.
5. Close fixes only when the reported behavior is actually resolved. Never
   clear the target by deleting labels, bulk closing unresolved reports, or
   claiming private proposals have run. Track publication after merge too.
6. Release ready corrections in finite batches. Hold admissions only during
   the exact-head cut and publication proof, then resume the next batch.
7. Preserve Davinci provider/fix-history and dependency-direction gates.
   Native-default, history, performance and broad roadmap work keep their own
   acceptance conditions; no legacy-backed shortcut grants native completion.
8. Measure typecheck/LSP speed with complete diagnostics and matched workloads.
   Existing cold/warm limits or source review do not establish a 10x result.

## Original open P0 set

A work-state entry names current custody; it does not grant runtime, merge or
publication acceptance. Unassigned entries must be triaged from live bodies,
existing fixes and original fixtures before starting another implementation.

- [#8026](https://github.com/ubugeeei-prod/vize/issues/8026) — fix(lsp): wrap long component hover contracts. **State:** Hover contract formatting; PR #8027; current source qualification.
- [#8016](https://github.com/ubugeeei-prod/vize/issues/8016) — lsp: documentLink for a directory import targets the folder instead of `index.ts`; tsconfig `paths` imports get no link; `[` / `]` are not encoded. **State:** Directory/alias document links; PR #8029; current source qualification.
- [#8015](https://github.com/ubugeeei-prod/vize/issues/8015) — lsp: template completion misses global attributes on component tags, `$emit` / `$attrs` / `$slots`, and JS globals; hover on a generic component's props shows `unknown`. **State:** Pending live-body and existing-fix triage.
- [#8014](https://github.com/ubugeeei-prod/vize/issues/8014) — lsp: comments inside template expressions are tokenized and navigated as code; template literals produce multi-line semantic tokens; `.art.vue` tag tokens include `<` / `>`. **State:** Template comments and token ranges; private implementation and source review.
- [#8013](https://github.com/ubugeeei-prod/vize/issues/8013) — lsp: `references` for script bindings is file-local unless the cross-file lint switch is on (never includes `.ts` importers); `workspace/symbol` only searches open documents. **State:** Pending live-body and existing-fix triage.
- [#8011](https://github.com/ubugeeei-prod/vize/issues/8011) — lsp: references / rename do not link event keys, slot keys, or props declared through a local type alias (`defineProps<Props>()`) to their uses. **State:** Pending live-body and existing-fix triage.
- [#8010](https://github.com/ubugeeei-prod/vize/issues/8010) — lsp: rename on an `emit("change")` string returns edits in TypeScript's `lib.dom.d.ts` (and misses the `defineEmits` key and parent listeners). **State:** Pending live-body and existing-fix triage.
- [#8009](https://github.com/ubugeeei-prod/vize/issues/8009) — lsp: rename / references / definition on component attributes (`:key`, `style`, …) fall back to the whole-element mapping and edit unrelated code (`defineEmits` → new name, `item-kind` → `showSuffixind`). **State:** Pending live-body and existing-fix triage.
- [#8008](https://github.com/ubugeeei-prod/vize/issues/8008) — perf(lsp): every type-backed request takes seconds once the project has a few hundred SFCs, and the time grows with project size (warm session). **State:** Warm LSP performance; private authority-preserving cache and measured wire preparation.
- [#8007](https://github.com/ubugeeei-prod/vize/issues/8007) — bug(zed): the extension still sends the recommended profile when the workspace has a `vize.config.*`, overriding its `languageServer` section (Zed half of #7196). **State:** Zed workspace profile; PR #8028; source green, full editor qualification awaits backend repair.
- [#8006](https://github.com/ubugeeei-prod/vize/issues/8006) — feat(lsp): `textDocument/documentSymbol` returns only the SFC blocks — no script bindings or template elements in the outline. **State:** Document symbols; PR #8030; current source qualification.
- [#8005](https://github.com/ubugeeei-prod/vize/issues/8005) — lsp(completion): a component event that shares a native event name is offered twice (`@change`, `@click`). **State:** Event completion union; PR #8031; current source qualification.
- [#8004](https://github.com/ubugeeei-prod/vize/issues/8004) — lsp(inlayHint): `computed()` bindings get the placeholder hint `: ComputedRef<_>` although the checker knows the type. **State:** Computed inlay checker information; private implementation.
- [#8003](https://github.com/ubugeeei-prod/vize/issues/8003) — check(checkUnknownComponents): every `<component :is>` reports a false TS2339 on an internal `__vize_dynamic_is_N` name. **State:** Dynamic component internal-name false diagnostics; implementation lane assigned.
- [#8002](https://github.com/ubugeeei-prod/vize/issues/8002) — lsp(diagnostics): with `typecheck` on, lint diagnostics after a `didChange` are published only when the Corsa type pass returns. **State:** Pending live-body and existing-fix triage.
- [#8001](https://github.com/ubugeeei-prod/vize/issues/8001) — lsp(completion): tag-name completion offers capitalized constants (`SORT_OPTIONS`, `DialogState`) as components. **State:** Pending live-body and existing-fix triage.
- [#8000](https://github.com/ubugeeei-prod/vize/issues/8000) — lsp(codeAction): a diagnostic that shares its range with another gets the other rule's quick fix. **State:** Pending live-body and existing-fix triage.
- [#7998](https://github.com/ubugeeei-prod/vize/issues/7998) — lsp: no code actions for `vue/v-bind-style` and `vue/prefer-props-shorthand` diagnostics, although `vize lint --fix` fixes them (not even "Suppress with @vize:forget"). **State:** Pending live-body and existing-fix triage.
- [#7997](https://github.com/ubugeeei-prod/vize/issues/7997) — lsp: completion inside a string literal offers identifiers: template bindings instead of the literal-union values, and Vue APIs in `<script setup>`. **State:** Pending live-body and existing-fix triage.
- [#7996](https://github.com/ubugeeei-prod/vize/issues/7996) — lsp: rename of a prop used through reactive props destructure (`const { label } = defineProps<…>()`) leaves the destructure out of sync (TS2339). **State:** Pending live-body and existing-fix triage.
- [#7995](https://github.com/ubugeeei-prod/vize/issues/7995) — lint(type/strict-boolean-expressions): ignores `noUncheckedIndexedAccess`, so `if (!list[0])` is reported as "an object is always truthy". **State:** Pending live-body and existing-fix triage.
- [#7994](https://github.com/ubugeeei-prod/vize/issues/7994) — lsp: rename does not expand a same-name shorthand: renaming the variable behind `:label` / `:id` (or the prop) rewrites the attribute name and breaks the binding. **State:** Pending live-body and existing-fix triage.
- [#7993](https://github.com/ubugeeei-prod/vize/issues/7993) — lsp: hover on a native attribute name (`:for`, `:id`) returns a `range` in virtual TS coordinates (line 5, column 17, sized like the DOM property `htmlFor`). **State:** Pending live-body and existing-fix triage.
- [#7992](https://github.com/ubugeeei-prod/vize/issues/7992) — lsp: documentHighlight and the code lens reference count match identifiers as text: tag names, attribute names and string contents count as uses of a binding. **State:** Pending live-body and existing-fix triage.
- [#7991](https://github.com/ubugeeei-prod/vize/issues/7991) — lsp: package.json `imports` (`#lib/*`) mapped to `.ts` files still report a false TS2307 in the editor, although `vize check` resolves them since #7031. **State:** Pending live-body and existing-fix triage.
- [#7990](https://github.com/ubugeeei-prod/vize/issues/7990) — maestro: opening an SFC that imports a `.vue` from a symlinked workspace package writes `X.vue.ts` / `X.d.vue.ts` into that package's source directory, and never removes them. **State:** Pending live-body and existing-fix triage.
- [#7989](https://github.com/ubugeeei-prod/vize/issues/7989) — lint: `script/prefer-ref-over-reactive`, `script/prefer-use-id`, `css/no-v-bind-performance` and `css/no-important` report text inside comments and string literals. **State:** Pending live-body and existing-fix triage.
- [#7988](https://github.com/ubugeeei-prod/vize/issues/7988) — lint(script/define-props-destructuring): enforces the opposite of eslint-plugin-vue's `define-props-destructuring` default, has no option, and contradicts `script/no-with-defaults`. **State:** Pending live-body and existing-fix triage.
- [#7987](https://github.com/ubugeeei-prod/vize/issues/7987) — lint(type/no-reactivity-loss): a snapshot in one function makes a same-named local in another function a "snapshot" (mutating a fresh copy is reported). **State:** Pending live-body and existing-fix triage.
- [#7986](https://github.com/ubugeeei-prod/vize/issues/7986) — cli: `vize lint` (even with `--no-config`) writes `node_modules/.vize/vize.config.schema.json` into the current directory, creating `node_modules/` if it does not exist. **State:** CLI schema side effects; implementation lane assigned.
- [#7985](https://github.com/ubugeeei-prod/vize/issues/7985) — lint: unknown rule names in `linter.rules` / `entries[].linter.rules` are silently ignored (a typo disables the rule without any message). **State:** Pending live-body and existing-fix triage.
- [#7984](https://github.com/ubugeeei-prod/vize/issues/7984) — lint(css/no-display-none): suggests `v-show` for `:deep()` targets, which live in a child component's template (follow-up to #7173). **State:** Pending live-body and existing-fix triage.
- [#7983](https://github.com/ubugeeei-prod/vize/issues/7983) — nuxt: the generated `.nuxt/oxlint.config.json` now uses `../` globs, which Oxlint rejects in `ignorePatterns` (config fails to load) and never matches in `overrides` (regression of the #7251 fix). **State:** Nuxt Oxlint scope and original file transport; PR #8024; current source qualification.
- [#7982](https://github.com/ubugeeei-prod/vize/issues/7982) — lint(ssr/no-browser-globals-in-ssr): browser globals in `<script setup>` are not reported, including the rule's own "Bad" example. **State:** Pending live-body and existing-fix triage.
- [#7981](https://github.com/ubugeeei-prod/vize/issues/7981) — lint(css/no-id-selectors): every finding is reported at the start of `<style>`, not at the selector (follow-up to #7171 / #7055). **State:** Pending live-body and existing-fix triage.
- [#7980](https://github.com/ubugeeei-prod/vize/issues/7980) — lint(css/no-utility-classes): class names inside CSS comments are still reported (follow-up to #7210). **State:** Pending live-body and existing-fix triage.
- [#7979](https://github.com/ubugeeei-prod/vize/issues/7979) — lint(vue/require-component-registration): the reported range is shifted one character left (`<MyButto` instead of `MyButton`). **State:** Pending live-body and existing-fix triage.
- [#7978](https://github.com/ubugeeei-prod/vize/issues/7978) — lint(vue/require-component-registration): no way to declare globally registered components (Musea `previewSetup`, `app.component()`). **State:** Pending live-body and existing-fix triage.
- [#7977](https://github.com/ubugeeei-prod/vize/issues/7977) — lint(a11y/no-aria-hidden-on-focusable): `inert`, `disabled` and `type="hidden"` elements are reported as focusable. **State:** Pending live-body and existing-fix triage.
- [#7976](https://github.com/ubugeeei-prod/vize/issues/7976) — lint(css/no-display-none): suggests v-show for elements reached through `:deep()` / `:slotted()`, which v-show cannot reach. **State:** Pending live-body and existing-fix triage.
- [#7972](https://github.com/ubugeeei-prod/vize/issues/7972) — compiler: await inside if / try blocks in <script setup> is not wrapped with withAsyncContext(), so the instance is lost after it. **State:** Pending live-body and existing-fix triage.
- [#7970](https://github.com/ubugeeei-prod/vize/issues/7970) — compiler: text and interpolations in component slot content become separate text vnodes (Vue merges them), so the DOM has extra text nodes. **State:** Pending live-body and existing-fix triage.
- [#7969](https://github.com/ubugeeei-prod/vize/issues/7969) — fmt(template): a hugged multi-line `{{ }}` inside an inline element is indented one level deeper than Prettier/Oxfmt. **State:** Pending live-body and existing-fix triage.
- [#7968](https://github.com/ubugeeei-prod/vize/issues/7968) — fmt(style): multi-value declarations (`transition`, `box-shadow`) lose the Prettier/Oxfmt one-value-per-line layout. **State:** Pending live-body and existing-fix triage.
- [#7966](https://github.com/ubugeeei-prod/vize/issues/7966) — glyph: `<style>` gets a blank line between every pair of rules and selector lists are joined onto one line (Oxfmt / Prettier keep the layout). **State:** Pending live-body and existing-fix triage.
- [#7965](https://github.com/ubugeeei-prod/vize/issues/7965) — fmt: generic arrow functions in `<script lang="ts">` get a TSX-only trailing comma (`<T>(…) =>` → `<T,>(…) =>`). **State:** Pending live-body and existing-fix triage.
- [#7963](https://github.com/ubugeeei-prod/vize/issues/7963) — lint(nuxt/nuxt-config-keys-order): `--fix` adds a trailing comma after the last property. **State:** Pending live-body and existing-fix triage.
- [#7962](https://github.com/ubugeeei-prod/vize/issues/7962) — lint(script/no-with-defaults): `withDefaults(` in comments and string literals is reported, in `<script setup>` and in plain `.ts` files. **State:** Pending live-body and existing-fix triage.
- [#7959](https://github.com/ubugeeei-prod/vize/issues/7959) — nuxt: `vue.compilerOptions.whitespace` forwarded from nuxt.config overrides `entries[].compiler` in vize.config, so per-directory whitespace never applies. **State:** Pending live-body and existing-fix triage.
- [#7951](https://github.com/ubugeeei-prod/vize/issues/7951) — fix(ci): preserve and diagnose patterned runtime child failures. **State:** Pending live-body and existing-fix triage.
- [#7950](https://github.com/ubugeeei-prod/vize/issues/7950) — check: the TS2353 for an unknown `v-model` modifier is placed after the attribute instead of on the modifier. **State:** Pending live-body and existing-fix triage.
- [#7949](https://github.com/ubugeeei-prod/vize/issues/7949) — check: a `.vue` outside the tsconfig directory makes every `defineModel()` `never` (TS2349) and skips native attribute checks under pnpm; explicit inputs regressed in 0.432.0. **State:** pnpm application helper resolution; PR #8032; current native/source qualification.
- [#7945](https://github.com/ubugeeei-prod/vize/issues/7945) — lsp: no lint diagnostics for `*.art.vue` files, while `vize lint` reports the `<variant>` markup (#7240). **State:** Pending live-body and existing-fix triage.
- [#7942](https://github.com/ubugeeei-prod/vize/issues/7942) — scoped CSS: a :slotted() anywhere in the block puts [data-v] outside :where() again and scopes the compound before > :slotted() (regression of #6985 / #6986 since 0.430.1). **State:** Pending live-body and existing-fix triage.
- [#7940](https://github.com/ubugeeei-prod/vize/issues/7940) — lint: in *.art.vue, no-unused-setup-bindings and require-component-registration ignore <script setup> bindings used inside <variant>. **State:** Pending live-body and existing-fix triage.
- [#7938](https://github.com/ubugeeei-prod/vize/issues/7938) — lint: vue/no-unused-setup-bindings reports an import used only in <script setup generic="...">. **State:** Pending live-body and existing-fix triage.
- [#7937](https://github.com/ubugeeei-prod/vize/issues/7937) — lint: script/require-function-return-type reports function type annotations like `(e: Event) => void`. **State:** Pending live-body and existing-fix triage.
- [#7936](https://github.com/ubugeeei-prod/vize/issues/7936) — vite-plugin: relative `import.meta.glob` in an SFC returns root-absolute keys, and matches nothing when the pattern leaves the Vite root (follow-up to #884). **State:** Pending live-body and existing-fix triage.
- [#7935](https://github.com/ubugeeei-prod/vize/issues/7935) — lint --cross-file: linter.rules cannot turn off or downgrade ecosystem/vue-router-unknown-route (and cross-file findings are always errors). **State:** Pending live-body and existing-fix triage.
- [#7934](https://github.com/ubugeeei-prod/vize/issues/7934) — lint: `script/no-export-in-script-setup` treats any `.ts` module with top-level `await` as `<script setup>`; `script/require-typed-ref` ignores the binding annotation. **State:** Pending live-body and existing-fix triage.
- [#7932](https://github.com/ubugeeei-prod/vize/issues/7932) — lint --cross-file: a test file's throwaway createRouter() becomes the route table, so Nuxt page routes are reported by vue-router-unknown-route. **State:** Pending live-body and existing-fix triage.
- [#7929](https://github.com/ubugeeei-prod/vize/issues/7929) — fmt(script): redundant parens are added before a trailing line comment inside `!( … )`. **State:** Pending live-body and existing-fix triage.
- [#7928](https://github.com/ubugeeei-prod/vize/issues/7928) — fmt(json): arrays and objects are always fully expanded and blank lines between members are dropped. **State:** Pending live-body and existing-fix triage.
- [#7927](https://github.com/ubugeeei-prod/vize/issues/7927) — lint --cross-file: provide/inject through a slot is reported as unmatched/unused again (regression of #869). **State:** Pending live-body and existing-fix triage.
- [#7926](https://github.com/ubugeeei-prod/vize/issues/7926) — fmt(style): selector lists are joined onto one line (no width limit) and a blank line is inserted between adjacent rules. **State:** Pending live-body and existing-fix triage.
- [#7924](https://github.com/ubugeeei-prod/vize/issues/7924) — fmt: non-idempotent — a multi-line directive value whose first line is a `//` comment drifts +2 columns on every run (regression from the #6694 fix). **State:** Pending live-body and existing-fix triage.
- [#7915](https://github.com/ubugeeei-prod/vize/issues/7915) — glyph: continuation lines of a multi-line CSS declaration drift on every pass with `tabWidth: 4` (indent doubles) or `useTabs` (indent shrinks). **State:** Pending live-body and existing-fix triage.
- [#7912](https://github.com/ubugeeei-prod/vize/issues/7912) — lint(vue/no-undefined-refs): a prop whose type comes from an imported props interface is reported as undefined when used bare in the template. **State:** Pending live-body and existing-fix triage.
- [#7908](https://github.com/ubugeeei-prod/vize/issues/7908) — lint --cross-file: `croquis/cf/browser-api-ssr` reports browser APIs in event-handler functions and in `watch` callbacks, even behind `if (!import.meta.env.SSR)`. **State:** Pending live-body and existing-fix triage.
- [#7907](https://github.com/ubugeeei-prod/vize/issues/7907) — lint --cross-file: script diagnostics (`croquis/cf/browser-api-ssr`) are positioned as if their offset were relative to `<template>` (wrong line, or past the end of the file). **State:** Pending live-body and existing-fix triage.
- [#7906](https://github.com/ubugeeei-prod/vize/issues/7906) — lint --fix: overlapping fixes need several runs (`v-bind:title="title"` → `:title="title"` → `:title`); `--fix` should repeat until no fixable diagnostics remain. **State:** Pending live-body and existing-fix triage.
- [#7905](https://github.com/ubugeeei-prod/vize/issues/7905) — lint --fix: `vue/html-self-closing`, `vue/component-name-in-template-casing`, `vue/v-slot-style` and `vue/no-boolean-attr-value` are documented as fixable but produce no edits (follow-up to #7204). **State:** Pending live-body and existing-fix triage.
- [#7904](https://github.com/ubugeeei-prod/vize/issues/7904) — oxlint-plugin-vize: `oxlint-vize -f json` reports offsets into the temporary bridge file, and template-only SFCs get a bogus column (`1:301`). **State:** Pending live-body and existing-fix triage.
- [#7903](https://github.com/ubugeeei-prod/vize/issues/7903) — oxlint-plugin-vize: `oxlint-vize` lints `.vue` / `.html` files that `.gitignore`, `ignorePatterns` and `--ignore-pattern` exclude. **State:** Pending live-body and existing-fix triage.
- [#7902](https://github.com/ubugeeei-prod/vize/issues/7902) — lint(type/no-unsafe-template-binding): fully typed `v-on` handlers (assignments, `() => fn()` on components) are reported as `any` / `unknown`. **State:** Pending live-body and existing-fix triage.
- [#7901](https://github.com/ubugeeei-prod/vize/issues/7901) — lint(script/prefer-computed): any `.value =` in a watch callback is reported, including async loads, resets, editable copies and DOM `value` writes. **State:** Pending live-body and existing-fix triage.
- [#7900](https://github.com/ubugeeei-prod/vize/issues/7900) — lint: in `.art.vue` files, bindings and components used only inside `<variant>` are reported by `vue/no-unused-setup-bindings` and `vue/require-component-registration`. **State:** Pending live-body and existing-fix triage.
- [#7899](https://github.com/ubugeeei-prod/vize/issues/7899) — lint: false positives in opt-in rules `script/prefer-computed` (watchers with side effects) and `a11y/use-list` (lone `*`). **State:** Pending live-body and existing-fix triage.
- [#7898](https://github.com/ubugeeei-prod/vize/issues/7898) — lint(type/no-reactivity-loss): mutating a DOM element from a template ref, or a deep `ref` object through `.value`, is reported as reactivity loss. **State:** Pending live-body and existing-fix triage.
- [#7896](https://github.com/ubugeeei-prod/vize/issues/7896) — lint(vue/no-undefined-refs): plugin globals (`$t`, `$vuetify`, …) and `const enum` names are still reported as undefined (follow-up to #7214). **State:** Pending live-body and existing-fix triage.
- [#7895](https://github.com/ubugeeei-prod/vize/issues/7895) — lint(type/no-unsafe-template-binding): calls of setup bindings and arrow handlers are reported as `any` / `unknown` (vize check is fine). **State:** Pending live-body and existing-fix triage.
- [#7893](https://github.com/ubugeeei-prod/vize/issues/7893) — compiler: enums from the plain `<script>` block (and `const enum` in `<script setup>`) compile to `_ctx.X` in the template and throw at runtime. **State:** Pending live-body and existing-fix triage.
- [#7891](https://github.com/ubugeeei-prod/vize/issues/7891) — atelier(ssr): `<Suspense>` renders its `#fallback` after the resolved content, and `<TransitionGroup>` is resolved as a user component. **State:** Pending live-body and existing-fix triage.
- [#7888](https://github.com/ubugeeei-prod/vize/issues/7888) — atelier(vapor): `.attr` / `.prop` bindings change attribute semantics: `:value.attr` sets the property, and attributes after `v-bind="obj"` lose to the object. **State:** Pending live-body and existing-fix triage.
- [#7886](https://github.com/ubugeeei-prod/vize/issues/7886) — atelier(vapor): default values in a destructured slot scope (`v-slot="{ label = 'x' }"`) are dropped. **State:** Pending live-body and existing-fix triage.
- [#7885](https://github.com/ubugeeei-prod/vize/issues/7885) — atelier(vapor): component listeners: a typed arrow handler is wrapped and never called, and `@kebab-case` is not camelized. **State:** Pending live-body and existing-fix triage.
- [#7884](https://github.com/ubugeeei-prod/vize/issues/7884) — atelier(vapor): a self-referencing component and `vFoo` directives from `<script setup>` are not resolved. **State:** Pending live-body and existing-fix triage.
- [#7883](https://github.com/ubugeeei-prod/vize/issues/7883) — atelier(vapor): `:key` outside `v-for` is dropped, so changing it does not re-create the element or component. **State:** Pending live-body and existing-fix triage.
- [#7882](https://github.com/ubugeeei-prod/vize/issues/7882) — atelier(vapor): template refs break: `ref` on a component is passed as a prop, and `ref` inside `v-for` is not collected into an array. **State:** Pending live-body and existing-fix triage.
- [#7881](https://github.com/ubugeeei-prod/vize/issues/7881) — atelier: dynamic directive arguments (`@[evt]`, `:[attr]`) read `_ctx.x` instead of the `<script setup>` binding in VDOM output. **State:** Pending live-body and existing-fix triage.
- [#7880](https://github.com/ubugeeei-prod/vize/issues/7880) — build: `compiler.whitespace` in vize.config is ignored (`"preserve"` still condenses), while `compiler.vapor` from the same file applies. **State:** Pending live-body and existing-fix triage.
- [#7877](https://github.com/ubugeeei-prod/vize/issues/7877) — glyph: a blank line is inserted between a root-level comment and the SFC block it documents. **State:** Pending live-body and existing-fix triage.
- [#7876](https://github.com/ubugeeei-prod/vize/issues/7876) — glyph: printWidth is ignored for long attribute values and whitespace-sensitive inline content in templates. **State:** Pending live-body and existing-fix triage.
- [#7874](https://github.com/ubugeeei-prod/vize/issues/7874) — check: checkUnknownProps / checkUnknownDirectives are still ignored on 0.432.0 (follow-up to #7234). **State:** Pending live-body and existing-fix triage.
- [#7871](https://github.com/ubugeeei-prod/vize/issues/7871) — glyph: template text whitespace is trimmed even with `compiler.whitespace: "preserve"`, changing the rendered text. **State:** Pending live-body and existing-fix triage.
- [#7868](https://github.com/ubugeeei-prod/vize/issues/7868) — glyph: a last-argument arrow function with a return type is not hugged (Oxfmt ≥ 0.62 and Prettier hug it). **State:** Pending live-body and existing-fix triage.
- [#7866](https://github.com/ubugeeei-prod/vize/issues/7866) — glyph: one declaration such as `opacity: .5` or `animation: …` stops `;` and `: ` normalization for the whole `<style>` block. **State:** Pending live-body and existing-fix triage.
- [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) — roadmap(ci): split PR checks from release gates. **State:** Pending live-body and existing-fix triage.
- [#6826](https://github.com/ubugeeei-prod/vize/issues/6826) — roadmap(davinci): restructure the Davinci crates around levels. **State:** Pending live-body and existing-fix triage.
- [#6258](https://github.com/ubugeeei-prod/vize/issues/6258) — fix(content-mapper): preserve disjoint diagnostic directive ownership. **State:** Pending live-body and existing-fix triage.
- [#6239](https://github.com/ubugeeei-prod/vize/issues/6239) — chore(release): track v1 alpha quality for Vue Fes Japan 2026. **State:** Root release coordination and publication evidence.
- [#6100](https://github.com/ubugeeei-prod/vize/issues/6100) — feat(davinci): finish production compiler lane and evidence gates. **State:** Pending live-body and existing-fix triage.
- [#4075](https://github.com/ubugeeei-prod/vize/issues/4075) — fix(content-mapper): support rename across camel and kebab event names. **State:** Pending live-body and existing-fix triage.
- [#3984](https://github.com/ubugeeei-prod/vize/issues/3984) — feat(canon): typecheck complete tsconfig projects and declaration emit. **State:** Pending live-body and existing-fix triage.
- [#3957](https://github.com/ubugeeei-prod/vize/issues/3957) — roadmap(tooling): make typecheck and editor workflows production-complete. **State:** Pending live-body and existing-fix triage.
- [#3952](https://github.com/ubugeeei-prod/vize/issues/3952) — test(tooling): exercise authored LSP features across real-project fixtures. **State:** Pending live-body and existing-fix triage.
- [#3864](https://github.com/ubugeeei-prod/vize/issues/3864) — chore(deps): drop the RUSTSEC-2026-0235 audit waiver once lightningcss stops pinning rkyv 0.7. **State:** Pending live-body and existing-fix triage.
- [#3295](https://github.com/ubugeeei-prod/vize/issues/3295) — chore(atelier): retire the CSS engine panic boundary once upstream lightningcss fixes land. **State:** Pending live-body and existing-fix triage.
