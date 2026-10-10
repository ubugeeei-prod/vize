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

Normal releases use an opt-in `VIZE_JSR_ENABLED` variable so missing registry
ownership cannot block urgent existing-product delivery. When enabled, JSR
publication and published consumers form an additional Release run requirement.
Existing GitHub Release creation prerequisites stay intact. The pinned 0.440
source cut is owned by the release delivery lane. This change stays outside the
queue until 0.440 version metadata, tag and publication complete: the pinned
catalog compares `release.yml` publication authority byte for byte, including
disabled jobs. Preserve that guard; a source pin alone does not clear this hold.

At preparation time the public `@vizejs` scope and package metadata return 404.
Required TODO: an authorized JSR scope administrator creates the scope/package,
links `ubugeeei-prod/vize`, configures Node-only compatibility, enables the
repository variable and runs initial publication. Registry authorization,
actual publication and all published consumer results remain pending. #8367
must stay open and JSR command tabs must not imply completed support until those
receipts exist.
