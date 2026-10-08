# Original SSR syscall observation

Refs [#7951](https://github.com/ubugeeei-prod/vize/issues/7951) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
The [existing IO diagnostic](./2026-10-08-canonical-corpus-linux-io-diagnostic.md)
remains unchanged.

## Observed boundary

Original SSR at source `83b6b0068d38db63063f45e5873ce33218a144e9`, tested union
`de6e71471dd7b7860c6d450b9b0a9c2f18af1cc6`,
[Check 37761047607](https://github.com/ubugeeei-prod/vize/actions/runs/37761047607/job/113257411258),
failed reading person/[itemId].vue. The complete printed path is 827 UTF-8 bytes
with 36 `packaging/deb/root` repetitions. This differs from the old 775-byte,
33-cycle UpNext failure. The authenticated native vector position is 14,068;
the original SSR had not independently archived its own vector or ordinal.
Artifact `11542008621` has 26,473,695 bytes, SHA-256
`45fd8a982802dc8238c14f3ff339bcf6099b536303c5688e379108b6ffca80a3`.
All 52 ZIP CRCs pass. Whole inputs, collector source/harness, toolchain and
148 gitlinks remain byte-exact. The subsequent same-worker diagnostic passed
both 44,367-input roots and their 177,468 actual operations each, including the
exact failing path. Neither success explains the original failure.

## Bounded diagnostic

Observe the first original SSR Cargo invocation with strace, preserving its
complete argv, inherited corpus setting, cwd, original exit/signal result and
ordinary `&&` Pug execution. Preserve the unchanged post-SSR four-operation
probe and every existing fatal observer/aggregate condition. Trace threads,
full pathname bytes, flags and syscall return errno; read buffers use raw
pointers/counts rather than cropped source payloads. Do not dump credentials
or the whole inherited environment.

Retain the actual linked test executable, its SHA-256 and ELF/link metadata,
Cargo/workspace toolchain/config/lock ownership, kernel, process root and mount
facts before cleanup. Actual execve trace selects the executable; a basename
alone is not a binary identity. Cargo's Wild linker configuration differs from
the standalone direct-rustc probe, but this is a custody boundary, not evidence
that Wild caused the error.

The test-only hook archives the once-collected original raw absolute/relative
NUL vectors and root without re-collecting or reading an SFC. The original
read helper executes exactly once. On its failure, retain the raw path,
original ordinal, PID/thread/cwd/exe and entire original panic bytes, then
resume the same panic. It adds no replacement reader or fallback. Diagnostic
custody completeness is metadata, not an additional acceptance layer.

Tracing and diagnostic file writes perturb scheduling, syscall timing and
build/run conditions. A traced success cannot repair or explain an earlier
untraced failure or establish performance acceptance. Root spelling, collector,
production reader, fixture bytes, alias multiplicity, all 44,367 inputs,
5,576 aliases, 148 gitlinks, oracles, accepted errors and resource ceilings
remain unchanged. Historical evidence is not rewritten.

## Qualification

Cause remains unknown. Compare the actual failed reader syscall/path with its
same-invocation vector and retained binary/source root. Report observed exact
main/release gates separately from this PR execution; a green standalone probe
or older candidate does not qualify a new source. The release owner controls
the fresh official candidate and its complete exact-head gates. This diagnostic
grants no cause, repair, retry-as-fix, issue/P0 closure or publication credit.
All upstream repositories remain read-only.
