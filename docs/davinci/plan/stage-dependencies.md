# Stage dependency policy

L0–L4 are physical Cargo package identities. The conversion packages consume
the levels on both sides of their boundary. `vize_l0_derive` is the proc-macro
leaf used by L0; `vize_carton` is a host and compatibility consumer of L0.

| Package          | Direct stage dependencies |
| ---------------- | ------------------------- |
| `vize_l0_derive` | none                      |
| `vize_l0`        | `vize_l0_derive`          |
| `vize_l1`        | L0                        |
| `vize_l2`        | L0, L1                    |
| `vize_l3`        | L0, L2                    |
| `vize_l1_to_l2`  | L0, L1, L2                |
| `vize_l2_to_l3`  | L0, L2, L3                |
| `vize_l4`        | L0, L1, L1→L2, L2, L3     |
| `vize_carton`    | L0                        |

The authoritative role map is
[`stage-dependencies.ts`](../../../tools/support/compat/davinci/stage-dependencies.ts).
The dependency gate reads Cargo metadata, so normal and build declarations are
checked even when optional or limited to another target. Direct stage edges must
fit the package's role. A helper cannot hide a reverse path. The policy graph
itself must be acyclic, and newly numbered stages need an explicit role.
Dev-only differential oracles remain permitted.

The broader [level dependency gate](../../../tools/support/compat/davinci/level-dependencies.ts)
also rejects stage-to-product paths and new legacy debt. Existing legacy
permissions can only shrink. Its five-minute Actions job runs mutation controls
for L4, the derive leaf, build edges, optional/target-specific declarations,
aliases and helper paths without installing JavaScript dependencies.
The [stage package tests](../../../tests/tooling/davinci-stage-dependencies.test.ts)
add exact normal-edge, physical identity and registry-publication checks.

## Product declarations

Products declare direct stage dependencies in their Cargo manifests. The table
below records those declarations, including optional dependencies. They describe
the product boundary and do not prove complete native adoption.

| Product         | Direct declared stages and conversions | Owning manifests                   |
| --------------- | -------------------------------------- | ---------------------------------- |
| Compiler        | L0–L4, L1→L2, L2→L3                    | `crates/vize_atelier_*/Cargo.toml` |
| Native linter   | L0–L4, L1→L2                           | `crates/vize_patina/Cargo.toml`    |
| Typechecker     | L0, L1, L2, L4, L1→L2                  | `crates/vize_canon/Cargo.toml`     |
| Language server | L0, L1, L2, L1→L2                      | `crates/vize_maestro/Cargo.toml`   |
| Formatter       | L0, L1                                 | `crates/vize_glyph/Cargo.toml`     |

The [consumer inventory](./consumer-migration-surfaces.md) scans every level and
both conversions, separating production/manifest mentions from test oracles and
retained compatibility spellings. Its generated shards and on-demand summary
show which source files use each surface. Manifest and lexical mentions do not
grant zero-fallback or behavior/performance parity credit.

L3's native decisions borrow their actual sealed L2 owner. L4 consumes those
decisions and the admitted source artifacts. The resident `vize_incremental`
consumer remains above the stages; stage libraries cannot depend on it.
The [storage boundary](./storage-boundary.md) owns shared strings and collections,
and the host-runtime boundary keeps filesystem/process integration outside the
stage dependency graph.
