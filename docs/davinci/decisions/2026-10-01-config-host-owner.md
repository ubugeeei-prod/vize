# Configuration host ownership

Issue: [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).
Provider: [in-memory documents](./2026-10-01-config-document-provider.md).

Carton owns the existing config path discovery, file reading, Node/PKL
evaluation, warnings and loaded-result records carrying source paths.
The loader module and its tests move unchanged in a dedicated move-only
commit. Carton's config module re-exports the exact L0 effective-model and
document types and consumes the opaque document provider. L0 has no public
forwarding loader API and no dependency on the host owner.

The CLI config facade and the LSP's explicit loading calls use Carton.
The JSON, JavaScript and PKL readers still deserialize once; LSP settings
still come from one evaluation. Discovery order, local runtime candidates,
process/evaluation failure distinctions, aliases, ordered lint rules,
Unicode values, warning/error bytes and fallback behavior are preserved.
The existing malformed-config snapshot is renamed with its owner while
its contents stay unchanged. Numeric ceilings and corpus fixtures remain
unchanged. The unused L0 `pklrust` dependency is removed; the host PKL
reader continues to invoke the existing external runtime.
The exact foundation storage-bridge inventory drops only the retired L0
loader's std String witness, without adding any storage exception.
The product storage gate recognizes only the exact CLI config facade and
three actual loader functions in their existing LSP source files. Mutation
laws still reject Carton storage/types, grouped imports and unreviewed APIs.

Replay with `vp node tools/support/levels/move-config-host.ts moves`, commit
the pure moves, then run `integrate` and `check`. The script requires the
provider and the published host-import gate companion, rejects source/target
collisions and supports repeated phases.
The two PRs form a native GitHub Stack; each exact source head is validated
before the ready prefix enters the protected merge queue.

The effective config types remain in L0 for this bounded split. Their final
framework placement, path matching/project context, i18n, profiler and
full L0 platform isolation remain unfinished under #6834. No product's
legacy compiler or checker route is replaced by this ownership change.
