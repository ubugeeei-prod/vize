# Authenticate the current Oxlint source addon at its actual load

The protected #8765 candidate `41470a7cbbbd802158af9a178f1382347b990f00`
failed its unchanged project transport command. This change removes a redundant
full-image read from unrelated Node starts and bounds the authentication buffer.
It does not establish the cause of the original deadline or a measured speedup.

## Original evidence

Check `38242094920`, JS job `114787233269`, retained both failures. The complete
original log is 1,453,528 bytes, SHA-256
`8777b5d7dee3cb12bc7eee14177c381fcdde6026a2b3a4884d45799118f42574`.
The two Oxlint hosts, 1.78.0 and 1.86.0, passed all four transport tests in
110.33 and 110.37 seconds. Each later HTML command reached 29 of 41 cases and
then failed its existing 180-second timeout. Both mixed-directory workers were
still incomplete at their shared 180-second deadline: each had completed 12
CLI cells and zero native cells. The original fatal statuses remain failures.
A successful leader waiting only for inherited output pipes is ruled out by
these incomplete captures.

Artifact `11743428765` belongs to that exact run/head and contains a
964,600,182-byte staged source addon, SHA-256
`dbfad4e1294d7f1f699a540c41c2973c0b9690ec653cce94f04f741ad9abb71b`.
Its original provider is the dev build, opt-level 0, debug information level 2.
Selected complete member bodies were obtained with authenticated HTTP ranges
and validated against their ZIP CRC and SHA-256; the entire 222,647,290-byte
ZIP was not downloaded or independently hashed. The selected journals contain
375 load events, 84 version handshakes and 210 host-child events. They do not
provide per-hash CPU time, peak memory, total startup counts or fault frames.

The original `project-native-custody.cjs` read the entire addon into a Buffer
and hashed it when every preloaded Node process started, including processes
that never loaded Vize. The actual binding could subsequently load bytes that
changed after registration. Both are concrete source boundaries; neither is a
complete attribution of the original timeout.

## Decision and preserved qualification

Keep the same complete image SHA-256 check immediately before each actual
Vize binding load. Read the whole regular file through a 64 KiB buffer and
require an unchanged descriptor/path identity, mode, length and timestamps
across that read. An open, read, hash, identity or close failure refuses before
the loader. File replacement, truncation and growth refuse. This check is not
an atomic hash-and-load operation; the existing external build and post-run
whole-body checks remain required.

A stock/version-only process and an unrelated addon do not emit any Vize load
or native-call receipt. The registered hook still refuses ambient NAPI binding
substitution. The actual staged Vize image, source, provider/build recipe,
original input bytes, native descriptors, complete call results and event
schemas are retained. The existing parent build/pre/post checks, two Oxlint
versions, mixed-directory cells, HTML cases, n8n replay, raw output and every
assertion remain unchanged. No command, concurrency, timeout, profile, feature,
checklist, corpus row or diagnostic is removed or relaxed.

## Controls and delivery

The new filesystem/process controls use inert files and an explicit fake
loader; they never execute a compiled addon and grant no native qualification.
They require the complete streamed image digest and original call receipts,
zero Vize claims for stock/unrelated starts, same-size/preserved-mtime mutation
refusal after registration, missing/read-error refusal, and mutation/path
replacement refusal during hashing. The original hook fails the exact
post-registration mutation refusal while the candidate passes it. All buffer
and descriptor assertions apply only to the physical source-image descriptor.

Ordinary exact-head Actions and protected full qualification must carry this
change through actual merge. A later original host result can establish that
all existing workloads fit their unchanged deadlines; current local controls
cannot claim that. The original #8765 remains off queue until a genuine source
repair is integrated and its current source/native/protected checks qualify.
Actual total time, CPU/RSS and any accepted speedup remain TODOs rather than
inferred results. Pair this decision with the existing CI issue #6830.

## Original hosted result and fixture enrollment

Source Check `38249549626` at `ac3e60be` passed the complete original transport
vectors. Both mixed-directory runs completed 98 observations, 45 CLI cells and
8 native cells in at most 97.335 seconds. Both hosts completed all 41 HTML
cases in at most 77.214 seconds under the unchanged 180-second deadline, then
completed the real n8n replay. The staged image differs from the failed
candidate, and the failed command did not reach n8n; these observations do not
establish a paired total speedup or the original timeout cause. Only selected
whole artifact members were authenticated, not the entire 234,939,986-byte ZIP.

The separate native run `38249549334` failed the original projection check:
71 comparisons versus the committed 64. It found no mapping divergence.
The actual merged #8757 introduced 49 physical Vue inputs. Keep the original
64-comparison assertion for all other inputs, authenticate the complete body,
mode and exact census of these 49 carriers, and run the same mapping checks on
every carrier. Print every carrier's complete actual diagnostic observation
before the unchanged assertions. A separate seven-comparison assertion is a
predicted partition of the observed total, not a measured per-file baseline;
if either partition is wrong, the test remains red. No 64-to-71 repin or input
exclusion is admitted. Real filesystem controls refuse missing, unknown,
duplicate, changed-body, symlink and changed-mode carriers. Their compilation
and the actual partition require fresh native Actions before merge.

The first carrier source plan refused three shifted import coordinates in the
existing generated consumer ledger. Place the support module after the original
imports so those exact coordinates remain unchanged; no generated ledger or
consumer classification is rewritten. The original planner refusal is retained.

The same native run refused compilation before carrier observations: this
repository's SHA output array does not implement `LowerHex`. Format each byte
as two lowercase hexadecimal digits, following the existing Canon observation
helper, with the complete digest and all admission checks unchanged. No actual
64/7 partition was acquired in that failed run or inferred as a passing result.

The next source run rejected the registry's directory-prefix assertion under
its existing exact-oracle policy. Require the immediate parent to equal one of
the two registered source directories. Full source hashes, exact 49-path census,
all mapping checks and strict 64/seven partitions remain unchanged; no assertion
allowlist is added. Preserve that original tooling failure separately.

Native run `38259064583`, job `114844597419`, at corrected source `18c52b2f`
completed successfully. The exact projection target passed all five tests with
zero ignored cases, including the three real Linux filesystem refusal controls.
Its complete log is 3,676,377 bytes, SHA256
`35792507c80dec02f1e6b2107dab553fe31526126645fa476b80f726f11be6b7`.
All 49 complete per-file JSON observations match their registered source SHA;
their compared diagnostic vectors total seven, with the original 64 separately
passing. This establishes that source's partition rather than a guessed total.
The stronger parent-directory guard and each new source composition still need
fresh exact-head qualification; the earlier source assertion failure is not
changed into success by this separate native result.
