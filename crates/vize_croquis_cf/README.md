# vize_croquis_cf

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_croquis_cf` is the opt-in cross-file companion to `vize_croquis`.

It aggregates semantic facts across module and component boundaries: dependency
graphs, provide/inject relationships, event and emit flows, fallthrough
attributes, reactivity flows, and project-level complexity facts. It does not
own source syntax; `vize_relief` owns that layer, while `vize_croquis` owns the
single-file semantic identities and relationships this crate consumes.

Cross-file analysis is kept separate because it has different caching,
invalidation, and cost characteristics from ordinary single-file analysis.

Its Atlas `CroquisProjectProduct` is produced only when a project recipe asks
for it. The provider declares one cross-source `CroquisSemanticProduct` request
per supported `.vue`, `.jsx`, or `.tsx` source before execution, then builds an
owned deterministic project snapshot. Ordinary compiler, lint, and typecheck
recipes create neither the project product nor project-level cache state.

## Key Entry Points

- `CroquisProjectProduct`
- `CrossFileAnalysisProduct`
- `CrossFileAnalyzer`
- `CrossFileOptions`
- `DependencyGraph`
- `ModuleRegistry`

## License

MIT
