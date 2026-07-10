# vize_croquis_cf

`vize_croquis_cf` is the opt-in cross-file companion to `vize_croquis`.

It aggregates semantic facts across module and component boundaries: dependency
graphs, provide/inject relationships, event and emit flows, fallthrough
attributes, reactivity flows, and project-level complexity facts. It does not
own source syntax; `vize_relief` owns that layer, while `vize_croquis` owns the
single-file semantic identities and relationships this crate consumes.

Cross-file analysis is kept separate because it has different caching,
invalidation, and cost characteristics from ordinary single-file analysis.
