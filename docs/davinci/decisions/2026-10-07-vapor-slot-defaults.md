# Vapor slot destructuring defaults (#7886)

The original [report](https://github.com/ubugeeei-prod/vize/issues/7886) supplies
complete `App.vue` and `Child.vue` sources: a missing `label` must render
`fallback:1`. Preserve its issue body, shell reproduction, both SFCs and both
reported JavaScript snippets byte for byte in the compiler differential corpus.
The runtime pack also pins independently authored update controls.

The Vapor generator previously retained only destructured binding names, then
resolved every local to `_slotPropsN.local`. That discarded defaults and original
property paths. Default-bearing slot patterns now retain checked binding paths
and wrap each assignment read in Vue's lazy `getDefaultValue` helper. The existing
expression owner resolves fallback expressions and computed property keys in the
current slot/loop scope. The binding walk uses an explicit stack and the existing
expression nesting guard. Patterns without defaults retain their existing path;
no extra pipeline stage, level serialization or budget change is introduced.

The [reported Vue 3.6.0-rc.10 generator](https://github.com/vuejs/core/blob/v3.6.0-rc.10/packages/compiler-vapor/src/generators/for.ts)
uses the same lazy helper. Ordinary Actions run the original complete Vapor SFCs
and seven authored controls under the locked Vue 3.6.0-rc.9 runtime. The controls
compare complete VDOM/Vapor observations, namespaces, fallback side effects,
reactive updates, diagnostics and unmount. Missing/undefined values use defaults;
null, empty strings, false and zero retain their supplied values. Named aliases,
comments, quoted keys, nested object/array paths and nested-parent defaults are
covered. Installed execution of the report's rc.10 remains unexecuted.

Default-bearing slots remain outside the current native L3 admission. This repair
changes the shared retained generator only; whole-product native SFC credit and
the compiler history gate remain unfinished. Source Actions, unchanged protected
instruction ceilings/full suites, actual merge and released installed verification
are pending. The release owner must verify the original two-SFC example after
publication. No n8n upstream state is changed.
