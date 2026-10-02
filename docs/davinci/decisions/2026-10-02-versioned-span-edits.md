# Versioned L1 span edits

Issue: [#6876](https://github.com/ubugeeei-prod/vize/issues/6876).

The existing embedded source retains authored offsets and decoded bytes, but
neither its decode map nor its text identifies the complete preparation input.
Checking equal decoded text or equal numeric spans would admit unrelated input.
`EmbedSource` now retains a private borrowed `authored_root`, populated only
from the actual preparation input. Native attribute decoding, authored sources,
trimmed Vue interpolation and checked slices retain that same reference.
The readonly accessor performs no parse, decode, traversal or allocation.

This is physical buffer provenance: compare the original pointer and length.
It provides no semantic document, language, grammar, File or producer authority.
Documents can share borrowed bytes, and empty buffers need not have distinct
pointers. A real caller must retain its own document identity and version.
Maestro already exposes URI, client version and a changing document revision;
its separately obtained `text()` values are distinct owned snapshots. A future
adapter must capture those host identities with its retained parsed source,
and must not substitute an independently copied equal-text buffer.

| Retained 64-bit payload | Before | After | Change |
| --- | ---: | ---: | ---: |
| `EmbedSource` | 40 bytes | 56 bytes | one 16-byte borrowed `str` |

The source remains Copy and has no Drop. The reviewed production storage row
stays exactly one L0 String import/two bound uses and one arena Vec import/two
bound uses; the getter adds no owned storage. The new origin laws have their
own reviewed test-only L0 String row for distinct live allocated buffers.
Plain preparation and identity slicing still use zero arena bytes in the law.
That observation does not establish unchanged instructions or whole-process
allocation performance, especially for arrays of the larger retained payload.

Local validation compiles the actual complete L1 library under strict Clippy
against the retained official dependency cache and runs five laws against that
real library: separate equal allocated roots, Unicode/entity exact boundaries,
identity/entity slices, trimmed interpolation, shared/empty limitations and
the payload size/no-Drop/plain-allocation observation. It is a scoped cache
proof, not a fresh whole-Cargo or hosted acceptance result.

The shared checked edit API and real diagnostic/fix/code-action consumers are
next. Stale host versions, incorrect source roots, UTF-8/range errors, partial
entities and conflicting edits must refuse application atomically. Product
defaults and legacy fix application remain unchanged until their fix-history
gates close. Fresh exact-head Actions, all 100 unchanged instruction ceilings,
full native observations and protected merge are required before acceptance.
