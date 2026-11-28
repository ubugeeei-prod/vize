# Actual L2 text rebuild instruction probe

This temporary diagnostic compares `vize_l2::analysis::text_facts::rebuild_source`
through its public API, compiled from two immutable product commits. It is not
a replacement for shipped-corpus, native correctness, wall-time or memory
qualification. No product default or acceptance rule changes.

The four authored inputs rebuild to 12 and 24 inline bytes, and 43 and 103 heap
bytes. They contain ordinary static text, expressions and Japanese text. These
are new Linux x86-64 client windows, not the historical aarch64 measurements in
the unsafe decision record. The source bundle is fixed and retained unchanged
for every build; it must not be adjusted after observing candidate deltas.

A separate 1,040-case native oracle compares the actual product function with
plain string appends over four UTF-8 widths, 65 lengths and all four flag
combinations, including `raw_html=true, dynamic=false`. The measured results
must preserve exact output, length, heap representation and capacity.

The driver independently builds A/B/A/B using the same cleared package and
target paths, the source lock's dependency identities, the repository release
profile and existing pinned measurement toolchain. All four actual ELFs,
compiler metadata, native oracles, three raw Callgrind executions per ELF,
window identities, dump hashes and source hashes are retained. Both A/A ELF
hashes and counts must be exact, and any positive instruction delta fails.
The instruction client is byte-identical to the repository harness; allocator,
guest environment, libc dispatch, compiler identity and build timestamp use
the same existing protocol.

Only the temporary branch's `shipped-profile-measure.yml` dispatch runs this
probe. Supply full `base_sha` and `head_sha`. The workflow's producer SHA,
source SHAs, run ID and attempt bind the evidence. Inspect retained raw data
and all required product checks before crediting any result.
