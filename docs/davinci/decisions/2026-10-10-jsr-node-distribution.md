# JSR Node distribution (#8367)

Publish one `@vizejs/vize` ESM facade with root, `/config`, `/native` and `/vite`
exports. Every entry point re-exports the same exact Vize release from npm;
native binaries stay in the existing platform packages. Node >=22 is supported,
with Vite's additional Node >=22.12 requirement. Runtime portability is not
claimed for Deno, Bun, browsers or Workers. CLI executables and the other npm
packages are outside this first JSR offering.

Package metadata and exports are staged from tracked `jsr/vize/jsr.json` and
aligned release manifests. Publish dry-run checks the genuine JSR source graph,
types and current public npm dependencies. A repository-linked OIDC workflow
publishes only after maintainer scope/package bootstrap, then installs through
the actual JSR CLI in fresh Node consumers on Linux, macOS and Windows. Retain
registry source checksums, exact installed package versions, type checks,
configuration/native compiler output and Vite bundle evidence.

JSR package guards live in `tests/tooling/jsr-package.test.mjs`, preserving the
existing audited 34-file release-contract inventory. Unknown JSR inputs retain
the existing conservative full source gates; no selector or budget is narrowed.

Normal releases freeze the default-disabled `jsr/vize/channel.json` policy so
missing registry ownership cannot block urgent existing-product delivery. An
enabled H requires JSR publication and all four published consumers in its
official Release run. `VIZE_JSR_ENABLED` only authorizes inside that required
publisher: clearing the variable fails rather than skips the required channel.
Existing GitHub Release creation prerequisites stay intact. The 0.440 source cut
was owned by the release delivery lane. This change stayed outside the queue
until its metadata, tag, terminal publication and exact-version public checks
completed. That hold cleared before this change rebased onto actual main
`c6570cb8706dd9a4f16be984c7ecd04bdb57b26d`. The pinned catalog still compares
`release.yml` authority byte for byte, including disabled jobs; each later
publication window requires the same preservation.

Future source cuts also capture the JSR reusable workflow, generator, public
consumer, package template, package README, license, policy and policy reader
as publication authorities.
Any changed byte differs from the qualified catalog; a partial lane fails closed.
The four existing authorities and every existing graph field remain intact.
Enabling JSR is a maintainer bootstrap decision after successful initial manual
publication and all consumer receipts. The catalog captures the H policy,
exact package identity, version and four exports. The public verifier derives
expected facade/README/license bytes from raw H, verifies actual published
checksums, and rejects a missing, skipped or unsuccessful official publisher
or original consumer. Disabled H has no JSR delivery credit. Legacy source H
without this lane retains the original plan and receipt shape. Record exact
publication/consumer receipts with the first and each later enabled release.

At preparation time the public `@vizejs` scope and package metadata return 404.
Required TODO: an authorized JSR scope administrator creates the scope/package,
links `ubugeeei-prod/vize`, configures Node-only compatibility, enables the
repository variable and runs initial publication. Registry authorization,
actual publication and all published consumer results remain pending. #8367
must stay open and JSR command tabs must not imply completed support until those
receipts exist.
