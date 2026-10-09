# Native-element component casting (#8328)

Original report: https://github.com/ubugeeei-prod/vize/issues/8328

`original.template.txt` preserves the reported template. The adjacent fixtures
cover ordered static/dynamic props and the default slot, a decoded prefix and
attribute value, and customized native elements without the `vue:` prefix.

Vue 3.5.43 parser and transformElement are the reference: custom elements and
`v-pre` stay native, static `is="vue:name"` selects `name`, consumes `is`, and
uses component props and slots. Dynamic `:is` on a native element stays an
ordinary binding. `ordinary-template.template.txt` covers a template without
structural directives; it selects a component while template if/for/slot
carriers and authored slot outlets retain their special contracts. The fixing commit credits reporter Danila Poyarkov.

The public L1-to-L2 and native construction regressions assert actual component
operations without a legacy compiler oracle. Cross-backend regressions compare
complete DOM/SSR/Vapor output with the equivalent named component and retain
the plain native and dynamic-component controls.

The four `reserved-*.template.txt` fixtures preserve static casts whose names
are `component`, `Component`, `slot` and `template`. Their complete DOM/SSR
render-function oracles derive from the installed primary Vue 3.5.41
compilers; the referenced 3.5.43 parser and resolver retain these semantics.
The oracles adapt only established Vize function formatting: DOM's six
parameters and whitespace, and SSR's inline resolver, compact text array and
stable-slot annotation. DOM hoisting is disabled for this finite comparison.
These are static component references, never dynamic selectors, outlets or
fragment wrappers. The Vapor regression independently proves native admission
and a regular component operation with props/default slots, then compares
complete native and retained output.
