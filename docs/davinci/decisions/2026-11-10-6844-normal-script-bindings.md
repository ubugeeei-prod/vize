# #6844 normal-script binding facts

Related: #6844, S7 in `2026-11-09-6844-native-script-inventory.md`

The normal `<script>` half of the native script-analysis route now has an L2
observer for its top-level runtime bindings. `OrdinaryObserver` runs inside the
existing `FileProducer::program_observed` traversal; it does not parse, walk,
serialize, or adapt Croquis output a second time.

The observer records source-backed runtime imports, variables, destructuring
bindings, functions, classes, enums, and named exported declarations. It reuses
the Vue binding classifier shared with `<script setup>`, while disabling setup
compiler-macro meaning. Consequently, a normal-script call named
`defineProps` remains an ordinary possibly-ref call, and only Vue imports make
the known ref/reactive call families authoritative.

Completeness is explicit: no Program, a second Program, a missing statement
event, or a span overflow returns `OrdinaryFactsIncomplete`; no partial fact
table escapes. Rollback truncates the statement's rows with the enclosing File
walk.

## Options API facts

`OptionsObserver` runs in the same walk and reads the component a normal
`<script>` declares: the `export default` options object (directly, through
`defineComponent(..)` or `Vue.extend(..)`, `as`/`satisfies`/`!` wrappers or a
same-file name) or a class component. `ComponentOptions` lists every static
top-level option key and each member of the option groups (props, computed,
filters, methods, inject, data, asyncData, setup, then same-file `mixins` and
`extends`) with its authored name, key and owner spans. Escaped string keys
are listed apart, never guessed. Module bindings stay in `OrdinaryFacts`:
a template reads options members through the instance, not the module.

The oracle is Croquis on the repository corpus (`tests/vue_options_facts.rs`,
`tests/vue_option_keys.rs`), equal except four files where a top-level
declaration named like a `data` member hides that member in Croquis. Vue
reads the instance there, so those rows are Croquis defects.

## Compiler switch: Options API bindings

The SFC compiler reads the template bindings of a normal `<script>` without
`<script setup>` from `OptionsFacts` (`compile/bindings/options.rs`), and
`collect_options_api_bindings` with its Croquis call is gone. The prepared
script content is parsed once by L1 with the block's `lang`, as the setup
route does. Members bind as Vue's `compileScript` does: `props` as `props`,
`data` (and Nuxt 2 `asyncData`) as `data`, `computed`, `methods` and
`inject` as `options`, and Vue 3 `filters` bind nothing. A name declared
by several groups, in the component or a mixin, binds as Vue's instance
proxy resolves it: a `setup()` return (setup state, so no option binding)
before `data`, before `props`, before the other options. Croquis let the
last member read win, so a mixin's `data` member could hide a `setup()`
return. Only a prop's kebab-case key also binds its camel case; Croquis
camelized every group, so `'kebab-key'` in `data` bound a `kebabKey` that
the instance does not have. A script the levels refuse forwards no bindings,
so its template reads members through `_ctx`, which resolves the same
members at runtime; nothing falls back to Croquis.

On the repository corpus (83 normal-script SFCs in the DOM, SSR and Vapor
shapes) the output is byte-identical except the four Croquis-defect files
above, where a module constant spelled like a `data` member now leaves the
member bound (`$data.filters` instead of `_ctx.filters`), as in Vue, and
`tests/fixtures/typechecker/options-api-computed-setters/WrappedOptions.vue`,
whose props `foo-bar` and `plain` are also computed names: they now read
`$props.fooBar`/`$props.plain` (props win over computed in the instance
proxy, and Vue does not define a computed that shadows a prop).

## Compiler switch: a paired normal script

Beside `<script setup>`, the template also sees the normal `<script>`'s
runtime bindings. The compiler now reads them from `OrdinaryFacts`
(`compile/bindings/ordinary.rs`), parsed once by L1 with the block's `lang`,
and `collect_normal_script_bindings`, the compiler's own second OXC walk, is
gone. The facts classify these bindings as Vue's `compileScript` does:
`ref`/`reactive` and their siblings count only when imported from `vue`, a
`let` is never a ref, and an ambient `declare` has no runtime binding. A
named `default` import from a `.vue` file is the component itself, as a
default import is. This rule was added to the shared L2 classifier, so
`<script setup>` follows it too. The first declaration of a name wins, and a
script the levels refuse exposes nothing.

On the corpus (32 SFCs with both blocks, DOM, SSR and Vapor) the output is
byte-identical except three `vue-language-tools` fixtures (`#2206`, `#4820`,
`events`). Their normal scripts only `declare` the components and functions
the template uses. The old walk bound those names, so the module returned
identifiers that do not exist at runtime (`const __returned__ = { D, A, … }`).
They now resolve as components (`_resolveComponent("A")`) or through the
render context, as in Vue. Across the whole corpus (5,988 compiles), one
more file changes through the shared classifier: `tsc/#5067` imports
`{ default as isComp1 }` from a `.vue` file in `<script setup>`, which now
binds as setup-const, so the template reads the component directly, as in
Vue.

## Open

This is not S7 completion. The Patina, Maestro and Canon readers of Croquis'
Options API descriptor remain, and so do two `OptionsObserver` gaps inherited
from Croquis: a returned name is resolved against module objects rather than
its lexical scope, and spread members are dropped rather than marking the
group incomplete.
