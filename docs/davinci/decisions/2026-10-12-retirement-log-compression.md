# Complete compressed retirement logs

The failed v0.446.0 candidate remains unpublished and cannot qualify a successor.
Source PR #8656 has H `9466fdbb90bfae8803d5e57ae267b656be051721`,
Release run `38147482427`, and original operator run `38147034594`.
Its official retirement run `38149566953` failed before archive creation or PR
closure because complete failure logs exceeded the 8 MiB raw-log storage ceiling.
Original failure logs and the failed retirement log remain intact.

All 26 terminal-failed jobs were subsequently captured from their raw Actions
job-log APIs: 22 Matrix jobs, two full Check jobs, pinned Release preflight, and
the original operator. Complete raw responses total 10,119,127 bytes. Python gzip
encodes the same responses in 1,727,835 bytes, with all 26 complete round trips
equal to the originals. The accounting fixture records every job, complete raw
byte size and SHA256, and diagnostic gzip measurement. These are failure
diagnostics, never green release or product evidence.

New archives use an explicit v2 schema and one `failure/<job>.log.gz` blob per
actual failed job. Each original stream has its full byte length and Git blob
identity recorded separately from the stored compressed blob identity and size.
Reading or installing an archive verifies the complete job inventory, stored
blobs, gzip checksum, exact decoded byte length, and original byte identity.
Unknown schemas, missing jobs, forged identities, corruption, truncated streams,
additional gzip members and trailing bytes refuse the operation.

The compressed log aggregate remains capped at 8 MiB. Both stored archive size
and metadata plus complete decoded logs remain capped at 64 MiB. Decoding stops
at the declared finite size plus one byte; lengths and aggregate arithmetic are
checked before accepting data. Original v1 archives keep their exact raw format,
8 MiB raw-log bound, complete receipt equality and original parent/ref identities.
This changes the new evidence representation; it does not raise qualification
budgets, omit failures or shorten logs.

Regression laws preserve BOM, invalid UTF-8, NUL, ANSI, CRLF and trailing blank
lines. A separate inert accounting model retains the measured 10,119,127-byte
log footprint alongside the prior 28,364,208-byte complete history footprint.
It exercises real local Git archive installation/readback and preservation of
every original source/pin/integration ref. The inert packet is explicitly a
storage model and does not replace the original public failure streams.

PR #8672 merged as `e08fce23bdd675517fa31c3679c5090129cd3513` after its
exact-source Check and all five protected workflows succeeded. Official main-only
retirement run `38155071068` then completed successfully at
`2026-10-11T16:25:53Z`, preserving the same immutable candidate. Source PR #8656
and integration PR #8657 closed without merging or changing their heads.

The durable `release-retired/v0.446.0` ledger is
`a166fa76f751fec1fc0b39929188bfcdda4be249`. Its two parents are original pin
`c01b6b73ab4d9447902a45a3def19082a4dbb4bf` and integration head
`92ad2441649f869343100d4d225b3bce1a3cb768`. Independent readback checks every
stored blob against its Git identity, validates each single gzip member and CRC,
and compares every decoded stream byte-for-byte with the separately retained
26 original Actions API responses. Actual stored logs total 1,736,912 bytes;
decoded logs remain 10,119,127 bytes. Complete metadata is 43,877,035 bytes, with
SHA256 `6515893bf70d0047e59b3ab655cb9d7c20a0685b72ec061eec7061206bc84dfb`.
Metadata plus decoded logs totals 53,996,162 bytes, below the unchanged 64 MiB
ceiling; compressed logs remain below 8 MiB. Diagnostic Python compression above
is a separate measurement, not the actual stored size.

The ledger preserves all 54 public absence observations: 26 exact npm versions,
26 exact crates.io versions, a complete Marketplace version inventory, and the
exact Open VSX version. A fresh independent replay of those same public queries
also passed after retirement. Authenticated GitHub lookups still find no
`v0.446.0` tag or Release, all 21 publication jobs were skipped, and the original
source/pin/integration remote refs remain intact. Latest published release remains
v0.443.0 with 78 assets. Actions artifact metadata is retained, but this archive
does not claim permanent custody or reuse qualification for binary artifacts.

The retired version is skipped. A successor requires a new source cut and fresh
full gates, including the real-project failures that blocked v0.446.0; successful
retirement does not qualify those failures or authorize successor publication.
