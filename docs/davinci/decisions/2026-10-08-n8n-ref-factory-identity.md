# Resolve Vue ref factories by lexical import identity

Issue: [#8275](https://github.com/ubugeeei-prod/vize/issues/8275), a P0 child of
[n8n adoption #8142](https://github.com/ubugeeei-prod/vize/issues/8142).

The native CLI requirements select `script/no-ref-as-operand` at error severity.
On source `db6f2c09fe0b11f6660c8cda40a642ce5ba9e875`, direct spelling checks miss
`ref as makeRef` and `Vue.ref` imports while reporting local functions, imports
from unrelated packages, and shadowing parameters named `ref`.

Resolve factory and result identity through the existing OXC script scope walk.
Each private scope frame distinguishes ordinary bindings, imported ref factories,
composition-module namespaces, and ref results. Prime local names before resolving
initializers so local declarations and parameters shadow imported factories.
Hoisted `var` declarations, class/static/switch frames, and separate parameter
initializer/body scopes preserve JavaScript visibility.
Factory aliases, static namespace members, namespace destructuring, and copied ref
results retain their binding identity. The supported provider modules are `vue`,
`@vue/composition-api`, and `#imports`; default imports and unrelated modules do
not implicitly become composition namespaces.

No new parsing, pipeline stage, cross-level serialization, public struct field,
or normal dependency is added. Existing operand diagnostics retain their message,
severity, byte locations, labels, help, and native fix representation. Legacy unit
sources now explicitly import the factories they exercise, rather than treating
an undeclared spelling as proof of Vue identity. Instruction-count acceptance runs
in the protected merge queue; no speed improvement is claimed from local timings.

The [companion legacy corpus](../../../tests/_fixtures/differential/lint/ref-factory-identity-8275/) records complete native before/after packets and
complete independent ESLint observations for 37 owned positive, negative, and clean
controls. The independent provider uses all 51 explicitly mapped n8n rules,
including the three literal options; it does not filter provider errors, fix data,
suggestions, source, or counts. Provider versions and implementation hashes are
retained. Native diagnostics and ESLint diagnostics have different presentation
contracts, so native whole-packet goldens and independent rule-membership laws
are asserted separately.

The complete 58-case audit and all original 26 raw packets remain recorded alongside this focused
change. It also exposed unregistered lowercase/framework-prefix component gaps,
missing `valid-v-slot` checks, and overwritten comment children. Those production
changes remain separate from this issue. Broader defineModel inference and
TypeScript-wrapper parity and constant-evaluated destructuring keys also remain unfinished; no 51-rule semantic parity,
full n8n workspace adoption, native CLI deployment, or release completion is
claimed by this authored library API slice. See the [audit](./2026-10-08-n8n-next-cli-rule-audit.md).

Upstream n8n remains read-only. Only functional settings and owned sources are
recorded; adoption-branch source is not copied, executed, changed, or distributed.
