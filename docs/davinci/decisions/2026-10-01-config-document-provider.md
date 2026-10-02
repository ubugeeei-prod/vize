# In-memory config document provider

Issue: [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).

L0 owns an opaque `ConfigDocument` for one deserialized configuration value.
It transparently wraps the existing private raw model and exposes the pure
effective-model, feature, linter-plan, entry and compiler-setting projections
already used by the host loader. The existing loader consumes this provider
before its ownership changes, making the subsequent Carton move an actual
provider-to-consumer dependency.

The provider takes no path, environment, process, clock or filesystem input.
It adds no parse, serialization roundtrip or pipeline stage. Existing private
raw structs remain private; effective model types and `VueVersion` retain
their identities for normal dialect consumers. Their eventual framework
ownership is separate from this bounded platform split.

Legacy aliases, compiler/experimental precedence, ordered entry rules and
rule options, editor timeout defaults/minimum and error locations are
unchanged. The stable-rule-options loaders retain their legacy compatibility
fallback, while the full config-rule-options loader retains its existing
distinct projection. Existing config tests and two in-memory document laws
validate the provider without changing frozen fixtures or numeric ceilings.

The dependent Carton slice owns discovery, Node/PKL evaluation and loaded
records carrying source paths. Until that slice merges, L0 still contains
the host loader. Config path matching/project context, i18n, profiler and
full L0 platform isolation remain open; #6834 stays open.
