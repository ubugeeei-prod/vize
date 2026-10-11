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

The repaired operator must actually merge before the same immutable candidate
can be retired through the official main-only command. Retirement then still
requires authenticated no-publication proof and original-run/source identities.
The next release requires a new source cut and fresh full gates, including the
real-project failures that blocked v0.446.0.
