# Compact Musea prop inference

Issue: [#8495](https://github.com/ubugeeei-prod/vize/issues/8495).

## Decision

Musea's shared `analyzeSfcFallback` split a direct `defineProps` type literal
by newlines and matched one property per line. Consequently the unchanged
`defineProps<{ label?: string; constructor?: string; hasOwnProperty?: string }>()`
input inferred only `label`, with the remaining declarations included in its
type. Its equivalent multiline input inferred all three fields. Both the
analysis and palette routes use this fallback when the native binding does
not expose `analyzeSfc`.

Use the existing TypeScript dependency to find the actual direct `defineProps`
call and read its direct `PropertySignature` members. Preserve decoded literal
names, each complete authored type span and its optional marker. Identifier,
string, numeric and computed string-literal names are supported. Nested object
types, function types and strings containing semicolons stay whole; comments
and strings resembling macro calls do not become declarations. Existing
`withDefaults` and emits extraction remain unchanged.

Recovered names such as `scope.name` also require quoting in the palette's
exported TypeScript interface. Keep valid identifiers, including Unicode
identifiers, unchanged; serialize other property names as string literals.
This changes neither Rust nor the native API, and adds no compiler replacement.
Imported type aliases, index signatures and dynamic computed keys are outside
this direct-literal slice and are not expanded or evaluated.

## Regression custody

Eight complete analysis and native Art palette vectors retain the original
compact and multiline source bytes, paired three-field and five-field forms,
computed literal/Unicode keys, nested/delimited/function types, macro defaults,
emits and unrelated local variables. The five-field vectors contain `label`,
`scope.name`, `__proto__`, `constructor` and `hasOwnProperty`. Whole palette
responses, existing ordinary output and syntactically valid exported TypeScript
are strict expectations, rather than selected field checks.

Two real browser vectors generate controls from this metadata, edit every
declared field, render the actual generated preview, copy the exact visible
component usage, compile and mount that copied SFC, save, reload, Reset and
reload again. Current values, received messages and persisted JSON retain all
literal own keys and unchanged object prototypes. Vue itself excludes
`__proto__` from component props; the complete component DOM expectation
records that boundary while the editor/message/storage dictionaries retain it.
Message and storage custody uses raw JSON strings so browser automation's
object transport cannot discard that own key. No compiler or route is mocked.

The [original alias failure packet](../../../tests/_fixtures/differential/musea/usage-component-tag-before.tar.gz)
is untouched, including its original compact input and failed inference.
SHA-256: `dca396d230814c9c7717a0bbdf4851fb3855aa76c3db2d6ac82db7e59b71c45b`.
The [compact failure packet](../../../tests/_fixtures/differential/musea/compact-props-before.tar.gz)
retains all four genuine before failures, complete native API and browser
observations, unchanged authored inputs, source custody and complete local
after observations. It also retains the two initial harness failures and the
published binding's custody receipt. SHA-256:
`3111b20363e371f4eaccef1db50f9153c9fafef19deed43b78c38d0a664da8a4`.

## Qualification boundary

Local source tests pass the original defaults laws and all eight vectors.
Local native API/browser observations use the verified original published
`@vizejs/native-darwin-arm64@0.440.0` binding, with the supported override
removed after the official wrapper loads it. They diagnose the real integration
but do not qualify current Rust or a release. Chromium sandbox advisories are
retained; page errors and Vue warnings must be empty.

The existing Actions native job must build the binding from the exact successor
source and execute the unchanged Art, inline-preview and copied-tag commands
alongside the appended compact laws. Its complete artifacts remain required.
This isolated preparation follows the frozen copied-tag parent and grants no
actual-main integration, PR, protected merge or publication credit. Existing
qualified props predecessors and the private copied-tag parent remain frozen.
Fresh actual-main composition, full exact-head checks, protected delivery and
successful publication are still required before reporting delivery complete.
