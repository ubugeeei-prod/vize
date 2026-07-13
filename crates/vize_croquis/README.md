# vize_croquis

`vize_croquis` owns Vize's frontend-neutral semantic product for Vue templates,
SFCs, JSX, and TSX.

Relief records what source syntax was written and where. Croquis derives what
that syntax means and how it relates: symbol identity, scopes, bindings,
reactivity, usage, and analysis graphs. Cross-file aggregation is kept in the
separate `vize_croquis_cf` crate.

## Highlights

- Scope tracking for template and script bindings
- Binding metadata used by compilers and type checking
- Reactivity and macro analysis
- Frontend-neutral semantic facts used by virtual TypeScript and diagnostics

## Key Entry Points

- `CroquisSemanticProduct`
- `CroquisSemanticSnapshot`
- `CroquisSemanticSnapshotBuilder`
- `Drawer`
- `DrawerOptions`
- `Croquis`
- `BindingMetadata`
- `ScopeChain`
- `SymbolTable`

## Feature Boundary

- `--no-default-features` exposes the owned Atlas product, semantic snapshot,
  queries, and syntax-independent builder without depending on `vize_relief`.
- `analysis` adds Croquis script/scope/reactivity analysis while remaining
  Relief-free. JSX and Croquis CF use this feature explicitly.
- `relief-compat` is the default compatibility surface. It adds the Vue
  template `Drawer` adapters and legacy virtual-TS helpers over Relief nodes.

Relief can therefore produce Croquis facts, but it is not part of the cached
Croquis graph contract and is not required by non-SFC frontends or project
aggregation.

## Related Crates

- `vize_armature` provides the parsed template tree
- `vize_relief` owns source-faithful syntax nodes and locations
- `vize_croquis_cf` aggregates Croquis facts across files on demand
- `vize_rendu` separately carries render intent to DOM, Vapor, and SSR backends
- `vize_flow` separately carries control, data, and effect flow
- `vize_canon` and `vize_maestro` reuse the analysis layer for type-aware features

## License

MIT
