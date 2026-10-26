# Literal declaration reference regression

The production batch checker receives a real declaration file outside the checked
project, in a directory containing an ampersand and Unicode. The declaration is
materialized from the pinned `.d.ts.txt` carrier, which is excluded from project
discovery. Only the generated reference directive can admit its ambient binding.

The valid case must have no diagnostics. The invalid case must retain the complete
authored TS2322 diagnostic. Encoding the path as XML entities instead produces an
unresolved binding and fails both cases. The fixture runner supplies the existing
`VirtualTsOptions.reference_paths` option; it does not alter provider declarations
or compiler settings.

The independent compiler oracle in
`tests/tooling/reference-path-compiler-oracle.test.ts` compares actual TypeScript
5.9.3 and native TypeScript 7.0.2, including valid/invalid/repaired inputs and the
original entity-encoding mutation. Quote and angle directory names are tested on
hosts where those names are legal. Source-built CLI and LSP qualifications remain
separate from this compiler grammar oracle.

The oracle retains special-name fixture directories in `literal-fixtures.tar`,
with their original filenames and a byte/hash manifest in
`fixture-archive-packet.json`, because the artifact service rejects those legal
Unix filenames. Compiler process packets and each phase's complete source remain
separate files in the same artifact.
