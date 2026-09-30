# vize_l2_to_l3

`vize_l2_to_l3` lowers Davinci L2 semantic regions into the flat L3 Impeto
program shape and exports the shared static/dynamic partition facts computed
during that lowering.

Lowering copies tags, namespaces, attributes, expressions, binding targets,
model read/write contracts, modifiers, branch conditions, loop aliases and slot
parameters into L3-owned operands. L2 and its source arena can be released after
lowering. The shared partition facts and graph-only Folio remain unchanged;
`vize_l3::values_folio::L3ValuesFolio` exports the paired value page.

`PartitionFacts::stale` names the first fact that stops describing the
program. The P3-10 `vize_l3::optimize` pipeline writes only the placement
overlay, so the export stays current after it at every `-O` tier.

Opaque reasons, foreign dialect identifiers and Vue filter classifications are
preserved without interpreting them. Foreign dialect fact tables and retained
expression ASTs are not serialized by this value-page contract. Stateful Lean
execution and migration of the backend generation payloads remain separate work.

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## License

MIT
