# Verify third-party fixes from published packages

Tracking: [#8337](https://github.com/ubugeeei-prod/vize/issues/8337),
[#6239](https://github.com/ubugeeei-prod/vize/issues/6239) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Original reports: [#8328](https://github.com/ubugeeei-prod/vize/issues/8328)
and [#8329](https://github.com/ubugeeei-prod/vize/issues/8329).

Local public-registry operations failed DNS resolution. The existing Native
Smoke workflow builds and installs workspace tarballs, and the source Musea
browser contract uses explicit fixture APIs. Those checks remain useful source
evidence. They cannot establish that an available public version contains the
reported fixes.

The independent [public acceptance workflow](../../.github/workflows/release-public-acceptance.yml)
runs **after** successful official publication. It adds no cut, build, promotion
or merge gate. The official pinned release operator, fresh source qualification
and existing artifact checks continue independently. No prospective version or
historical run supplies acceptance.

GitHub API reads are anonymous; the workflow grants only the checkout's required
`contents: read` permission. Shared runner egress can exhaust GitHub's anonymous
API quota and produce HTTP 403. That bounded failure retains its diagnostic log
and supplies no public acceptance; there is no token fallback or automatic retry.

## Inputs and authority

Dispatch only reviewed protected `main`, providing the complete official C/H,
stable minor tag, frozen source PR and successful Release run R. The verifier
requires raw H's sole parent C, the source PR's exact unchanged H/branch and
unambiguous immutable markers, a complete terminal successful R with the same
source identity, and the actual annotated tag pointing to H. The tag annotation
must bind the same C/H/PR/R. A published, nondraft GitHub Release must reference
that tag. Its `target_commitish` branch label grants no source authority.

H is the ordinary frozen version commit; this verifier adds no signature gate.
It preserves the observed tag annotation, public metadata, run/jobs and raw H
source blob hashes. These observations make **no cryptographic signature claim**.

The plan comes from replacement-disabled raw H Git objects with checked blob
types and modes. It derives the actual npm publish targets and native catalog,
the ordered crate plan and editor identity. Current counts are derived each time;
a previous release's package count is not an allowlist.

Each planned npm package is queried by exact version and its returned archive is
hashed against the public SHA512 SRI. Crate versions and returned archive
checksums are checked independently. Marketplace availability is read separately.
Optional Open VSX availability is a separate named result and cannot delay
official publication. Zed is checked only as an own-repository GitHub Release
archive when present in H's plan. No upstream registry change or PR is made.

## Fresh public consumer

The Ubuntu consumer lives outside the source checkout. It starts with a new
private manifest, exact public Vize versions and a fresh npm lock v3. Installation
uses the public npm registry with install scripts disabled. There are no source
package links, workspace aliases, source native bindings or runtime overrides.
The fixed browser infrastructure versions and complete resolved lock are retained.
Other platforms' optional native lock entries may be absent from disk; their lock
versions and public metadata still agree. Every required Linux package must be installed.

The native probe replays all prepared #8328 DOM and SSR pairs: the original
nested selector, component props/default slot, entity-encoded selector and the
ordinary `is` controls. It compares complete code, preamble and helper outputs.
The resolved Linux x64 GNU provider must be the installed public `.node` file,
have the exact version and supply the successfully loaded umbrella exports.
Both probes record the same loaded provider path/digest and installed package
manifest/lock/public archive identities.

The gallery probe imports the installed public Vite and Musea plugins. Its Vue
SFCs, themes and optional setup hook are copied byte-for-byte from raw H's
consumer example. The real public native compiler scans and compiles them;
there is no mocked art API or preview generator. A consumer setup wrapper only
observes setup count and initial globals before calling the original hook.

Real Chromium verifies brand, scheme and locale on **every** mounted preview,
the changed CSS variables and unchanged actual preview `Document` objects after
each toolbar operation, alongside iframe URLs and a single setup call. The parent
retains the original documents; setup also observes a per-document token. A deliberate
same-URL reload in the copied-URL context must fail that identity comparison.
The probe also checks another art and return navigation, a new preview
tab, a copied URL in a clean context and retained variant hashes. Sibling and
foreign commands must actually reach the preview before refusal is asserted.
An unconfigured gallery and an existing one-argument hook still compile and mount.

## Evidence and completion

The artifact retains the authenticated publication plan, source request, executed
tool authority, exact H fixture hashes, consumer lock, public download digests,
native and browser observations, screenshot and complete step logs. Shell
pipelines preserve failures; a failed observation cannot create the final success
receipt. Successful native and gallery observations must agree on package and
provider identity before `acceptance.json` is written.

Inert unit tests qualify identity, registry and input refusal logic. They do not
claim a real public install, native return or browser replay. Source Actions and
protected merge are separate from running this workflow against an actual
published version.

Remaining work: deliver this verifier through its own protected PR, dispatch it
with the actual official publication identities, inspect the complete terminal
run/artifact and record the available version on both original issues. The
original reports stay open until publication and installed public acceptance
succeed. Preserve the reporter trailers already verified on their actual merged
fixing commits. The independent verifier integrates signed actual release-operator
main `2bcc327c57e38c6b5d9709a158390ebce8457b7a`, preserves the complete incoming
operator and all earlier canonical rows, and registers its one new test within
the existing audited release scope (34 total release files, 29 scoped). Its local
inert laws and scope controls pass; fresh source Actions, protected delivery and
the real public replay remain separate pending proof.

The initial exact `7b1fe51fe6c1eb79cac99b1c8cfc02136d1e83ab` source Check run
`37912752099` passed formatting but its strict JavaScript job rejected three
type-aware `no-base-to-string` warnings in inert fetch fixtures. The fixture now
extracts string, URL and Request URL identities explicitly, preserving every
refusal and the zero-warning guard. The successor requires fresh full Actions.
The manifest-only filesystem unit also isolates and restores the source CI's
warning-only `NODE_OPTIONS`; the actual public workflow retains every strict
override refusal. All local laws are replayed under the real source CI flags.
Review follow-up removes the unused Actions permission, matches each bad input's
specific refusal reason and restores mutated fixtures even when an assertion fails.
