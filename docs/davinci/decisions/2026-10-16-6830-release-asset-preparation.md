# Prepare signed release assets during registry publication

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The final GitHub Release job previously started SBOM generation and signing only
after all its registry publication dependencies had succeeded. In
[Release 38805261385, attempt 3](https://github.com/ubugeeei-prod/vize/actions/runs/38805261385/attempts/3),
job 116688789141 took 104 seconds: CycloneDX and SPDX generation took 4 and 6
seconds, cosign installation 1 second, signing 59 seconds, and final publication
27 seconds. That attempt's overall Release result was failure; this is a step
timing baseline, not proof of a complete successful release.

Move the same preparation to a job that starts after the unchanged candidate,
protected metadata integration and immutable tag promotion barrier. Registry
publication runs alongside it. The final GitHub Release still requires every
existing publication dependency, plus successful asset preparation. Preparation
has read-only repository permission; SBOM actions cannot upload release assets.
Only the final publishing job can write the GitHub Release.

Carry existing binaries, checksums, signature bundles, certificates and SBOMs in
one uncompressed tar envelope. Its producing job supplies the immutable artifact
ID and SHA-256. Before extraction, the consumer verifies the complete digest and
the source H, workflow run, tag and producing attempt receipt. Both jobs require
H to match their event and actual checkout. A failed-job rerun can reuse the
same run's earlier successful preparation. The extracted recipe and helper stay
inside the pinned publication catalog authority.

No source-equivalence inference, rebuild, tarball lifecycle change, native ABI
relaxation, merge-queue bypass or registry-visibility shortcut is introduced.
The existing signing commands and final publication file selection are retained.

The split can overlap the observed 70 seconds of SBOM and signing preparation,
but adds a runner start, Node setup, envelope hashing, one upload and one download.
The prior CLI and editor workflow artifacts totaled 160,228,784 bytes; this
transfer is material and the new envelope size must also be recorded.
Its net effect needs an official Release measurement including those costs;
neither this graph change nor the earlier 9m19s build-only benchmark establishes
the ten-minute end-to-end target. Focused contracts exercise byte corruption,
foreign source/run/tag, rerun attempts, symlinks and missing signature refusal,
and preserve the full final publication dependency barrier.

## Remaining release time

A later audit of the same original release found that source checks, metadata-PR
checks and release builds already overlapped. Candidate readiness at 21:12:02 UTC
was followed by metadata merge at 21:12:21 and the annotated tag at 21:12:51 on
October 15. Publication started at 21:13:32; GitHub Release became public at
21:48:48, 60m01s after the operator started. These timestamps span three attempts,
and the Deno consumer still failed; they are not a successful release benchmark.
The npm critical path was native 11m27s, CLI 5m02s, Vite plugin 2m09s and Nuxt
3m53s. Platform publication already used concurrency four; this change uses eight.

The [trusted cache seed](https://github.com/ubugeeei-prod/vize/actions/runs/38904921642)
spent 9m47s planning and building, plus 8m10s queued. Windows ARM CLI was its
8m33s critical build. Qualification, metadata integration, tagging, publication
and public acceptance are outside that measurement. The remaining ten-minute
target needs a complete successful release measurement with each phase retained.

Do not move metadata merge-queue admission before readiness as a timing shortcut.
The existing catalog gate authenticates source and metadata identity but does not
check the release run's readiness or H's required checks; the operator checks
those before admission. There were no merge-group runs in the historical release
window, so its short merge interval is not evidence of today's protected queue
cost. No queue-guard change is justified by these observations.
