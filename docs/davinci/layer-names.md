# Davinci layer names

Davinci uses **L0–L4** for its numbered layers. The previous S0–S4 spellings
remain in historical implementation and measurement records, and in specific
versioned wire identities described below.

| Layer | Role                                  | Current Rust dependency | Cargo package            |
| ----- | ------------------------------------- | ----------------------- | ------------------------ |
| L0    | Source text, spans, arena and storage | `vize_l0`               | `vize_carton`            |
| L1    | Lossless surface trees                | `vize_l1`               | `vize_l1`                |
| L2    | Semantic UI IR                        | `vize_l2`               | `vize_l2`                |
| L3    | Reactivity and backend scheduling     | `vize_l3`               | `vize_impeto`            |
| L4    | Emission documents                    | Atelier targets         | Existing target packages |

The conversion packages are `vize_l1_to_l2` and `vize_l2_to_l3`. There is no
separate `vize_l4` package. Carton and Impeto keep their published package names;
the workspace imports them through the L0 and L3 dependency aliases.

## Rust package migration

The four S-named packages at `0.428.1` remain published under their original
names. The renamed packages first publish at `0.429.0`:

| Previous package | Current package |
| ---------------- | --------------- |
| `vize_s1`        | `vize_l1`       |
| `vize_s2`        | `vize_l2`       |
| `vize_s1_to_s2`  | `vize_l1_to_l2` |
| `vize_s2_to_s3`  | `vize_l2_to_l3` |

Update Cargo dependencies, Rust imports and stage-specific module/type names
when moving to these packages. The source API rename is intentional for the
`0.429` minor release; renaming a package does not redirect an old dependency.

The first-release registry semver gate uses the exact published `0.428.1` source
archive of the previous package as its baseline. Repository comparisons use the
actual base Git tree. Only the selected Cargo package/library identity is mapped
to the current name; a base workspace keeps its old dependency key and path with
an explicit package identity link. Versions, dependency aliases and Rust source
API remain unchanged, so removed public APIs still participate in the comparison.
Later releases use the renamed package's published baseline normally.

## Compatible wire identities

L names describe the current implementation and user-facing layer rail. Existing
versioned contracts keep their identities:

- Artifact-key hash domains and printed `sN.v1` keys are unchanged. Physical
  layer metadata uses `lN`; it is separate from the stable wire identity.
- Folio keeps `[s1]`, `[disegno]` and the existing `s2-`/`s3-` page headers.
  Named derive overrides preserve those headers for L-named Rust types.
- Extension WIT fields, feature tokens such as `s2-page@1`, schema numbers and
  frozen contract/SDK fixtures retain their original bytes. The SDK exposes
  L-named constants and `pages::l1`/`pages::l2`, with S-named API aliases for
  existing guest source. Output request fields `s2`/`s3` stay wire-compatible.
- Resident query accounting keeps its existing `s1_block`/`s2_page` IDs.
- Existing profile, remark and Spolvero stage strings remain valid. The
  playground maps them to L names for display. CLI pipeline selectors accept
  L names and their existing S aliases; terminal summaries always display L.
  Saved Folio/profile IDs retain their versioned S wire identities, and remark
  selectors accept either spelling against those saved records.

Historical exact-head evidence continues to name the packages and stages that
were actually measured. This naming change does not mark a phase complete,
change a performance target or replace an acceptance record.
