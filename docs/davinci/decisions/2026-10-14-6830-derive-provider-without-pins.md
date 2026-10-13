# Check where the derive provider comes from, not which version it is

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The Canon library test-scope qualification checked serde_derive's lock closure
against a table of exact versions, checksums and dependency lists, and required
the workspace serde requirement to be `=1.0.228`. Any crates.io update of
serde, serde_core, serde_derive, syn, quote, proc-macro2 or unicode-ident
failed the tooling tests, so routine dependency updates were held back.

The qualification keeps what it protects: Canon takes serde from the
workspace with the `derive` feature, no dependency table shadows serde or the
standard derive provider, every crate in serde_derive's lock closure resolves
to exactly one crates.io package with a checksum, and no `[patch]`,
`[replace]` or Cargo source override substitutes any of them. It no longer
pins versions, checksums or dependency edges. The provider test refuses a
non-registry source and a missing checksum, and admits a crates.io version
update.
