# Source-native staging for original project replay

The main `9d974d27b6823772d36a6855557cbbc27b3c1cd1` JS package gate
failed before either real Oxlint host or the licensed n8n replay could run.
Its fixed `target/oxlint-original-project-transport/source-native.node`
already existed. `COPYFILE_EXCL` correctly refused to overwrite it.

The complete original failure is [Check 37790211385, job 113356226977](https://github.com/ubugeeei-prod/vize/actions/runs/37790211385/job/113356226977).
The retained raw log is 1,254,207 bytes, SHA256
`da6c004ec0e051b5d45274f23eaa07a2ea58a32bcc5ea967e8098fb8865ee063`.
This staging collision is separate from the unexplained historical `828b`
reference timeout; no timeout repair is claimed.

Each invocation now allocates its own `run-*` directory under the existing
artifact root. It copies the current authenticated binary with `COPYFILE_EXCL`,
checks its complete SHA256, and copies the complete current build receipt
exclusively. Existing cached binaries and evidence are neither reused nor
removed. Complete and partial invocation evidence remains under the existing
recursive artifact upload. Only a successful qualification whose evidence upload
also succeeded cleans its exact freshly emitted directory afterward. Failed
qualification or failed upload keeps all current raw and custody evidence.
Cleanup refuses the shared root, unrelated names, outside paths, symlinks, and
incomplete stages. Existing host temporary-directory cleanup is unchanged.
The standalone caller retains local evidence when `GITHUB_OUTPUT` is absent;
that variable is optional and used only for the Actions upload cleanup handoff.

Filesystem regressions preserve the old fixed-target `EEXIST`, execute two
consecutive stages with different current binary and receipt bytes, verify
distinct paths and complete staged bytes, preserve stale cached evidence,
refuse an overwrite, reject an incorrect current hash, and check owned cleanup
without deleting cached siblings or failed evidence. The complete actual caller
staging paragraph also runs with and without GitHub output, proving both the
standalone path and exact Actions path emission without duplicating real hosts.
These authored
filesystem controls do not qualify a native addon or the whole project replay.

The existing Actions source-native build, both real Oxlint versions, all
sealed project and reference inputs, whole protocol assertions, licensed n8n
replay, and original 180-second reference limit remain mandatory and unchanged.
No public package catalog, manifest, collector, or unpublished TypeScript
consumer migration changes belong to this repair. Fresh exact-head Actions and
protected delivery are pending; this leaves #8142 open. The helper and its laws
join the existing strict erasable native TypeScript gate without a new job.
