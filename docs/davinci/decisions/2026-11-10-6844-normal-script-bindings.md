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
`inject` as `options`; a later member of the same name replaces an earlier
one, a `setup()` return drops the option binding it replaces, and Vue 3
`filters` bind nothing. A script the levels refuse forwards no bindings,
so its template reads members through `_ctx`, which resolves the same
members at runtime; nothing falls back to Croquis.

On the repository corpus (83 normal-script SFCs in the DOM, SSR and Vapor
shapes) the output is byte-identical except the four Croquis-defect files
above, where a module constant spelled like a `data` member now leaves the
member bound (`$data.filters` instead of `_ctx.filters`), as in Vue.

## Open

This is not S7 completion. The compiler route that replaces
`collect_normal_script_bindings` (a paired `<script setup>`'s view of the
normal script) with `OrdinaryFacts` remains open, and so do the Patina,
Maestro and Canon readers of Croquis' Options API descriptor.
