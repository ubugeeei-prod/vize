# Release catalog shipping targets

Paired issues: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830)
and [#8115](https://github.com/ubugeeei-prod/vize/issues/8115).

The unpublished 0.435.1 cut failed the final catalog comparison because eight
Cargo auto-discovered tests appeared in five published crates after H. The
complete manifests, shipping descriptors, features, non-development
dependencies and publication authorities otherwise matched. The release
operator superseded this unpublished cut and selected the next patch, 0.435.2,
under the maintainer's instruction to complete publication;
[the retirement record](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6049870396)
preserves the old source, runs, integration and failures. No tag or registry
publication is performed by this change.

The catalog descriptor omits a target only when `kind` is a nonempty array
containing exclusively `test`, `bench` or `example`. Library, binary,
proc-macro and custom-build targets retain every field, even when `test` is
true. Mixed, unknown, missing and malformed kinds remain in the strict
descriptor, so additions or changes cannot disappear from comparison.
`crate_types: ["bin"]` does not classify a target as development-only.

Full manifest bytes, features, non-development dependencies, registry
closure, native catalog and all four publication authorities remain exact.
Declaring a development target through a manifest edit still changes the
manifest authority and is rejected. Whole-report/verdict validators and the
existing release modes, budgets, source contract and publishers are unchanged.

The committed fixture retains the complete ordered target vectors of all
five affected crates at H `fd6241bf8ea5466794cc955138a59a9b75a8aac2` and
V `18ce7841e693550334c81584bae2f9782d27f786`: 224 original and 232 current
targets, including all eight added tests. Only source paths become relative
to each original metadata workspace root, matching the existing catalog
recipe; every other target field remains intact. The original full metadata
and its SHA256 provenance are preserved. An independent saved-metadata replay
compares the complete projected catalogs equally; hosted Rust execution
remains required before acceptance.

Rust controls exercise real Cargo discovery of test/bench/example files,
binary and build-script additions, complete manifest bytes, features and
normal dependencies. Further controls replay all eight original additions
and reject mixed/unknown targets and altered shipping metadata. The existing
public release CLI test executes the complete production Rust test entry.

The next official operator must pin its **driver source** to the actual signed
merge containing this fix, including the entry point and full release support
closure, before producing fresh C2/H2 and all gates/builds. Moon compiler
`0.10.7+bc794d341` stays unchanged: the Moon command delegates to the current
Rust source and contains no bundled old catalog code. Old compiler archives,
cut references, source runs and failure evidence remain preserved. Fresh
exact-source Actions, protected acceptance, actual merge and subsequent
public-consumer verification remain required; no old release proof transfers.

The Rust entry records `shipping-target-catalog-v2` as its explicit source pin.
[Rust-script 0.34.0](https://github.com/fornwall/rust-script/blob/0.34.0/src/main.rs#L358-L378)
can reuse a normal-mode executable after only imported modules change. Changing
the entry pin makes it rebuild from the new imported source. A normal-mode
control executes an unchanged cached entry with module value 1, retains its
stale value after the module changes to 2, then requires value 2 after only the
entry pin changes. The public CLI test requires this actual Rust control;
there is no launcher-wide force flag or compiler-version change.
