# SSR scoped slot metadata and published Rust option literals

The customer SSR fix derives the component's own slotted style flag from
scoped styles. The published 0.429.0 option structs are constructible public
structs. Adding a field breaks exhaustive downstream Rust literals, even when
the field is optional and the type derives Default.

This followup restores the exact published fieldsets of
TemplateCompileOptions and SsrCompilerExperimentalOptions. The computed SFC
flag travels in the private template compile context and private SSR/L4
request. A hidden additive SSR entry accepts the SFC flag; all existing public
compile signatures and direct-template true defaults remain unchanged.
There is no change to parsing, emission, serialization or pipeline stages.

The two ordinary public-option integration tests compile exhaustive old
literals. The SFC test checks the complete existing scoped-module fixture;
the SSR test compares complete code, preamble and map with the existing direct
compile entry. Existing twelve SFC whole modules, compiler facets and real
Vue 3.5.35/3.5.43 HTML/warnings are the behavior controls.

A PR comparison against already merged main does not establish compatibility
with published 0.429.0. Publication requires the actual registry baseline
cargo-semver-checks gate without a major override. Source fieldset/signature
identity, runtime regressions and actual registry semver results are distinct
proofs; pending results must remain pending.
