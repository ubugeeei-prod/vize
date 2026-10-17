# Public declaration marker text (#9300, related to #3984)

`Button.vue` is a minimal valid public literal type, not a generated binding.
`tsconfig.json` and `vize check --format json --declaration --declaration-dir types`
produce declarations with the official published v0.457.0 macOS arm64 CLI.

The authenticated archive SHA-256 is
`f6016dfc0ecce3dd631b1621613527071a2f461ad069f4b24040ed23b3249f75`;
the extracted executable SHA-256 is
`c45adea0d02352c526a9ae5f31e70f2bdf4fd315426c2cab838f7a5605526d57`.
The frozen release source is `03882089eff75664abe99ba71868e33739496b01`.
Its `declaration_outputs.rs` blob `f365a4bab02ac216b43336de8b3d3f44156e0a4f`
is identical to audited main `8f2e12d8519205bf2b5eae2db0e5e323a45a57d0`.
This is historical installed-CLI RED evidence, not a current-source binary pass.

The raw producer packet preserves inputs, argv, executable/native hashes,
status, stdout/stderr, and every emitted file (including declaration maps).
The independent consumer packets use only copied outputs in `node_modules`;
no library source workspace links participate. Native TypeScript 7.0.2 reports
TS2339 for a valid indexed access and misses the wrong literal assignment.
The ordinary-literal control has the correct clean/TS2322 outcomes. The TSX
packets additionally retain the false positive on valid component usage.

The generated owner is the single uninitialized, top-level, non-exported
`declare const __vize_component__` emitted by
`virtual_ts/generator/component_export.rs`. Public authored declarations with
the same exported name conflict with that generated binding: the separate
binding-collision packet records exact authored TS2395 and skipped emit.
Other names, exported declarations, nested declarations, mutable bindings,
interfaces, nested member types, comments and marker string literals are not
owned metadata. The fix must only remove direct readonly properties with
one of the three exact private keys from the generated leading type literal.

The maintained regression emits with the actual current-source CLI, removes
the library, and checks separate packed TS and TSX clean/broken/repaired
consumers with `skipLibCheck: false`. Pure AST laws retain the old generated
stripping cases and cover adjacent members, multiline types, public prefixes,
suffixes, comments and Options API branches. A UTF-16/CRLF map law composes
import rewrites, exact owned removals and the helper prefix, retaining authored
coordinates and removing only the private-field token.

#3984 remains open for its broader complete-project and backend parity contract.
