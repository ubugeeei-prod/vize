# Musea VRT identical decoded pixels

An unchanged screenshot previously calculated YIQ distance and grayscale output
for every pixel and allocated a full RGBA diff buffer, even though no diff file
would be written. After decoding both PNGs and validating their dimensions,
compare their complete RGBA buffers. Exact equality returns the same zero-diff
statistics without color calculations or a diff allocation. A 1920×1080 match
avoids 8,294,400 bytes of temporary diff storage.

This shortcut deliberately follows decoding and dimension validation. Identical
corrupt files must reject. Equal byte sequences with different image shapes must
still produce the existing size-mismatch image. Different encoded metadata can
describe the same decoded pixels; alpha compositing, tolerance, and anti-aliasing
remain the existing comparator's responsibility for unequal RGBA buffers.

The regression suite checks complete statistics and diff pixels for first/last
pixel changes and equal-data/different-shape images. It also covers metadata
differences, transparent/translucent/opaque pixels, alpha comparison on/off,
threshold endpoints, anti-aliasing settings, and identical invalid PNGs.

## Measurement

The package's `src/vrt/runner-comparison.bench.ts` accepts baseline and candidate
comparison modules. It creates deterministic PNGs, performs three warmup pairs,
then alternates implementation order for twelve measured pairs. Timing includes
both PNG reads/decodes and comparison, plus diff encoding/writing when needed.
Each pair must produce identical statistics; the changed-image case must also
produce byte-identical encoded diff PNGs.

Run from the Musea package, with dependencies installed for both source trees:

```sh
node --import tsx src/vrt/runner-comparison.bench.ts \
  /absolute/baseline/src/vrt/runner-comparison.ts \
  ./src/vrt/runner-comparison.ts
```

Repeat with `node --cpu-prof --import tsx ...` for source-level CPU sampling.
Keep sampling separate from the timing receipt to avoid profiler overhead in the
reported timings. The baseline was `64dd10a0ea62778435417c2321580dccb88abae3`.
The [receipt](../../../tools/benchmarks/results/musea-vrt-identical-images.json)
retains every timing sample, environment, source hash, and source-profile summary.

On Apple M2 Max, Node 24.14.0, macOS arm64:

| PNG pair                      | Before median | After median | Speedup |
| ----------------------------- | ------------: | -----------: | ------: |
| Identical 375×667             |     20.206 ms |    16.854 ms |  1.199× |
| Identical 1920×1080           |    156.782 ms |   132.550 ms |  1.183× |
| Last pixel changed, 1920×1080 |    271.119 ms |   271.457 ms |  0.999× |

The separate profile attributed 628 baseline self samples to `compareImages`
and 294 candidate self samples across the same three cases. These are local
end-to-end measurements and sampled source attribution; they do not establish
the Linux instruction-ceiling gate or browser capture throughput.
