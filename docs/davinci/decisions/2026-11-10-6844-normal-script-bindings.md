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

This is not S7 completion. The Options API descriptor and the compiler route
that replaces `collect_normal_script_bindings` / `collect_options_api_bindings`
remain open. Until those land with corpus and product acceptance, the existing
normal-script compiler collectors remain unchanged.
