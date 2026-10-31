# Public acceptance for issue 11145

`editor-vite-stub-public.yml` is an independent release-owner-dispatched acceptance
route. It does not build a source SDK, publish, select a release version, or gate
the existing release workflow. Pull requests run the filesystem observer's
falsification controls and an actual fresh pinned pnpm tooling-only bootstrap on
Linux, macOS and Windows. No Vize product is installed by that control. The pnpm
entry is selected only from the installed package's exact 12.1.0 manifest, with
lexical/realpath ownership and regular-file checks, never a guessed extension.
Manifest and selected entry hashes, original manifest bytes and version output
are retained. Pure missing/version/escape controls never execute package code.

The release owner supplies `version`, `source_sha` and `authority_blob` after the
containing version is public. The workflow must run from `main`. Its authority
capsule has the same fields as the existing Windows public-config consumer:

```json
{
  "version": "<exact public version>",
  "tagSourceSha": "<immutable vVERSION commit>",
  "producerSource": "<genuine producer source commit>",
  "npmSRI": {
    "<package>": {
      "integrity": "<exact public SRI>",
      "tarball": "<exact public tarball URL>",
      "metadataOrigin": "<registry.npmjs.org metadata URL>"
    }
  },
  "receipts": [
    {
      "rawBase64": "<unchanged vize.platformVsix.v1 receipt bytes>",
      "sha256": "<receipt digest>",
      "origin": { "runId": 1, "artifactId": 1, "member": "<genuine member>" }
    }
  ]
}
```

SRI entries are required for `vize`, `@vizejs/vite-plugin`, `@vizejs/native` and
each participating platform package. Each host requires exactly one matching
producer receipt, binding the loaded native image's bytes, SHA-256 and build token.
The actual merge `66eba1da97f1b94b3a01f79b3ec9c630e696bb33` must be an ancestor
of the producer, and the producer an ancestor of the immutable tag. Capsule bytes
and origins are retained; this route does not download the producer archive again.

The fresh project uses the unchanged reported config, app and two-file declaration
stub from `tests/_fixtures/differential/typechecker/editor-vite-stub-11145`.
Actions runs Node 26.6.0, pnpm 12.1.0, original Vite+ 0.3.0, original Vue 3.5.43
and TypeScript 6.0.3. The report did not pin pnpm or TypeScript; these host pins are
disclosed rather than attributed to the original report. A root runtime Vite must
be absent after installation; the test never deletes one to manufacture coverage.

The original single-file shell glob is passed as its exact expansion:

```sh
pnpm exec vize lint src/app.vue
```

In each of five phases, the route retains the whole command streams/status,
complete installed `vize/config` result, machine-readable CLI report and complete
installation inventory. Positive objects must be equal before the stub, with the
stub, after repairing a runtime-owning negative package, and after stub removal.
The runtime-owning package must remain authoritative and fail with its authored
error. Extra direct-Node CLI/API observations use the unchanged native-loader
custody hook; these are separate executions from the original pnpm command.
The negative JS API import can fail before its native normalizer is required;
its complete journal is retained without requiring or inventing a native load.
Every positive API execution, CLI observation and LSP session requires the actual
expected native loader to return successfully.
The negative CLI follows `lint/config_load.rs`: actual process status is exactly
2 and Rust exits directly, so no Node exit-hook event is expected. Its native
load attempt and successful return remain required. The negative JS API has
actual status 1 and a Node exit event. These are distinct recorded contracts.

The editor phase starts the actual installed public LSP with that same hook,
private TMP/TEMP/TMPDIR, and no source/checker overrides. It opens the original
app, discovers the live private Canon mirror, and deliberately replaces only the
mirror's owned `node_modules` with a whole link to the real installation. Windows
uses a real directory junction; Unix uses a directory symlink. Two additional
same-project, previously unopened hosts force cold and warm host preparations.
Both must detach the link into an owned directory, materialize the probe's actual
bytes, preserve every external file/mode/link, and shut down cleanly. Complete RPC
and native-loader journals are retained. This tests recovery from the reported
stale topology; it does not claim a fresh minimal pnpm workspace naturally creates
the old link, or establish general LSP navigation conformance.

Issue 11145 remains open until the three OS public executions and all their
source, package, image and filesystem assertions actually pass.
