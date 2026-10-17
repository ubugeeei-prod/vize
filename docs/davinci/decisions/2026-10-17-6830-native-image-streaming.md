# Bound native image authentication memory

Issue: #6830. Full qualification [38993084178](https://github.com/ubugeeei-prod/vize/actions/runs/38993084178)
at `2b0cf6c084ea0d4ab0a3dce33cfb9ebf0e7c8db2` completed the original Misskey
edit-churn workload but failed its unchanged 136 MiB server-RSS assertion:
the peak was 140.94 MiB at the first file-lifecycle cycle. RSS subsequently
fell; this observation alone does not establish an accumulating edit leak.

Each native editor-session authentication previously read the entire
24,101,026-byte executable into a temporary vector solely to hash it. The
same opened file is now read through a 64 KiB stack buffer. SHA-256 consumes
every successful read until EOF; interrupted reads are retried, every other
read error rejects authentication, and a digest is returned only at EOF.
There is no metadata shortcut, path cache, partial hash or changed provider.

The complete initialize-response comparison still runs first, every session
still authenticates the full executable, and the known digests, hexadecimal
encoding, decision placement and shutdown protocol remain unchanged. Opening
the path once retains the existing file-handle semantics. The buffer and
hasher have bounded storage independent of image size; this removes the
whole-image allocation without changing Cargo profiles or allocator settings.

Focused controls compare streaming with one-shot SHA-256 for empty input,
hash-block and buffer boundaries, and multiple buffers; exercise repeated
short reads and interruptions; reject an error after a complete buffer; and
detect corruption beyond the first buffer. Existing complete-initialize and
shutdown transcript controls remain in place.
The four new controls pass in a small local Cargo harness containing the exact
helper and test bodies with `sha2:0.11.1`. All six native authentication and
shutdown controls also passed in the actual Canon module after the Actions
workload below. Rust formatting and whitespace checks pass.

The earlier same-fixture [38929265536](https://github.com/ubugeeei-prod/vize/actions/runs/38929265536)
at `a00fb6cff9268e1371ba260739d905050e2a2573` passed at 133.37 MiB, but several
product changes separate those sources. It is a workload reference, not an
isolated attribution of a product change.

The single approved [run 38994698499](https://github.com/ubugeeei-prod/vize/actions/runs/38994698499)
at `8cb5a680cd9adec97ec251a32a9d35305fffc20b` passed on its first attempt.
That source is the failed `2b0cf6c` source plus only this production patch,
its controls and a temporary diagnostic workflow. The unchanged Vue parity
action ran in its original order on Blacksmith 32 vCPU, Node 24.14.0 and
Rust 1.99.1, with the original CI profile and budget scale 1.

Misskey churn peak server RSS measured **140.94 → 128.69 MiB**, passing the
unchanged **136 MiB** limit. Both peaks occurred at the first file-lifecycle
cycle. All 40 edit cycles, 20 file-lifecycle cycles and 63 RSS samples remain.
All 287 complete publish records match both original runs after replacing
only their temporary workspace prefix. The original 186 Node cases retain
185 passes and the same one GoGoCode-fixture skip; all 43 process-budget
cycle output hashes also match. All incremental lanes, thresholds and batch
reuse counters remain unchanged.

The retained source-build receipt identifies this exact diagnostic source.
After measurement, the provider image still measured 24,101,026 bytes with
SHA-256 `4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4`.
The artifact retains original metrics and 19 complete source-bound snapshot
sessions. Original churn does not capture raw wires or individual native
capability decisions; no instrumentation or capture environment was added
to its measurement.

Separately, observed median edit-cycle time was 234.29 → 230.42 ms and Vben
second-app cold open was 1,334.98 → 1,308.41 ms. A single run establishes no
stable latency gain, accumulating-leak diagnosis or allocator-page ownership.
The bounded allocation is established by the code; the RSS result is the
same-workload observation above. Combined full qualification with subsequent
main changes remains separate, and the diagnostic workflow does not land.

The first ordinary Check at `346308a3e2` rejected unchecked buffer slicing
under the repository's `clippy::indexing_slicing` rule. The reader now uses a
checked slice and rejects an invalid reported read length with `InvalidData`;
a controlled reader verifies that rejection. Complete-file hashing and the
64 KiB storage bound remain unchanged. Fresh Actions must validate this final
source; the diagnostic measurement above predates this boundary-check fix.
