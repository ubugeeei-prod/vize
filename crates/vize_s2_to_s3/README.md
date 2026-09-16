# vize_s2_to_s3

`vize_s2_to_s3` lowers Davinci S2 semantic regions into the flat S3 Impeto
program shape and exports the shared static/dynamic partition facts computed
during that lowering.

Lowering copies tags, namespaces, attributes, expressions, binding targets,
model read/write contracts, modifiers, branch conditions, loop aliases and slot
parameters into S3-owned operands. S2 and its source arena can be released after
lowering. The shared partition facts and graph-only Folio remain unchanged;
`vize_s3::values_folio::S3ValuesFolio` exports the paired value page.

Opaque reasons, foreign dialect identifiers and Vue filter classifications are
preserved without interpreting them. Foreign dialect fact tables and retained
expression ASTs are not serialized by this value-page contract. Stateful Lean
execution and migration of the backend generation payloads remain separate work.

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## License

MIT
