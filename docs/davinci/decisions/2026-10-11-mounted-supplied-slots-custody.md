# Original supplied-slot child diagnostic custody

Paired issue: [#7951](https://github.com/ubugeeei-prod/vize/issues/7951).
This diagnostic does not close that issue or qualify a failed full-source run.

## Original negative

[Check38071105461](https://github.com/ubugeeei-prod/vize/actions/runs/38071105461)
failed at head `f2ade1f7ca6dda81e14a75b46cb43980bd9762e2`, source-coverage job
`114268519341`. The original supplied-slot test reached its second fixture,
`named-prop`, Vapor backend. All four printed observations equal its Lean
reference, but the child terminated with SIGSEGV11. The original Rust status
assertion correctly failed before JSON/oracle assertions:59tests passed, one
failed, Cargo101, before Corsa. Its Node PID, actual running image and cause
were not captured. Rust's panic thread number is not a Node PID.

The whole original log digest is
`f16a76bdd91bbaf4b8f89ba9c68d7fee36c6b557c8fdf0e2d27d8a22553eae17`.
A separate shipping-source success and earlier SFC packets do not identify or
explain this failed child. The historical1d replay route from #8525 stays intact.

## One bounded current-source diagnostic

A separate manual-only workflow selects literal f2 and verifies tree
`6d4c9627d893bfeb9923cb116cf516241fbd2640` before installing its dependencies.
It snapshots authored helpers outside that checkout first. It reuses the
original source-coverage runner, immutable Action pins, environment, default
coverage profile and40minute limit. Only the integration target and exact test
name narrow execution. All eight fixtures, both backends, complete observations
and original fatal assertions remain unchanged; no observer variants,
saturation, retries, new provider or product gate are introduced.

The selected test still stops naturally at its first fatal assertion. A complete
successful subset must contain all16 child outcomes and whole original oracles.
This subset is not whole-source coverage acceptance; existing thresholds and
release gates remain unchanged.

A private launcher activates only for the exact original mounted trace script.
It forwards and retains every original stdin/stdout/stderr byte, recording the
actual Node wait result and RustPID→launcherPID→NodePID chain. This diagnostic
mediation changes parent topology and timing. A Node SIGSEGV remains signal11 in
the actual wait packet; the launcher returns139 without signaling itself or
creating another core. The unchanged Rust success assertion remains fatal.
Its outer exit139 must not be described as a direct Node wait signal.

Existing phase/trace and core/sysroot hooks are reused. Supplemental records
retain `/proc/self/exe` through an actual Node descriptor and compare mapped
backing files' device/inode. Core process/signal notes, unique PID/time, mapped
ELF identity bytes physically present in the core, and all raw byte hashes must
agree. A pathname hash alone is not a running-image proof. Missing, deleted,
changed or ambiguous images/core identity remain incomplete evidence and fatal;
no historical running-image claim follows from a current diagnostic.

Whole source peer and exact-byte-forwarding, wrong-source, PID/core mismatch and
missing-capture negative controls precede publication. After normal source CI
and actual merge, one explicitly reviewed dispatch may run. Its full artifact
and original negative remain preserved. A successful observed subset is one
nonreproduction under diagnostic mediation; it neither erases the original
failure nor establishes historical cause or #7951 completion.
