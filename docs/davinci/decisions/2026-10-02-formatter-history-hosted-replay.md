# Formatter history: original-source hosted replay

Issue: [#6882](https://github.com/ubugeeei-prod/vize/issues/6882).
Scope: immutable original source `331f64feb9c6b9c0058789fcdedc94daec474e8c`,
not execution of current main or the reviewed consumer `712617c6`.

The current replay also makes one runtime-neutral TypeScript correction:
requirement IDs use `Set<string>` so strict checking accepts membership tests.
Strict mixed TS/MJS checking enables `allowJs` to resolve existing JavaScript
interfaces; legacy `checkJs` behavior and every original fixture, expectation,
receipt and source pin remain unchanged. This adds no execution credit.

## Actual execution

Two distinct Check workflow dispatches ran sequentially: the second started
after the first whole workflow reached its terminal state. Both `test-scripts`
and `clippy-and-test` jobs succeeded. Both whole historical workflows failed
separate security/check-js jobs; these runs grant no whole/current/protected
acceptance.

| Source run                                                                    | Mandatory formatter job | Workspace Rust job    | Artifact    | Read-only transport run/job         |
| ----------------------------------------------------------------------------- | ----------------------- | --------------------- | ----------- | ----------------------------------- |
| [36936617526](https://github.com/ubugeeei-prod/vize/actions/runs/36936617526) | 110618272797, success   | 110618272975, success | 11198604100 | 36939365768 / 110627015401, success |
| [36939449372](https://github.com/ubugeeei-prod/vize/actions/runs/36939449372) | 110627281069, success   | 110627281634, success | 11200102871 | 36942303199 / 110636400172, success |

Each actual source-built observer matched all 300 registered API cases:
276 complete public outputs with fixed points, 21 typed errors and three
internal observations. The 14 Vue-version cases include 13 explicit selectors
and one default-version control.
Each run retained 852 actual formatting passes and 300 complete options probes.
Five historical CLI scenarios retained 20 complete check/dry/write/recheck
process observations per run. Across both runs, every full API/CLI row agrees,
including argv, options, input/output, stdout/stderr, exit status, signal and
process error. Native states remain explicitly unsupported.

The original 87-commit / 56-fix / 150-requirement denominator is unchanged.
Original331 schema1 manifests and original pure validators are authoritative
for these raw reports. Their full 300 executable input/options/expected/error
cases independently equal the reviewed compact712 schema2 declarations.
Neither raw manifest/report hashes nor frozen corpus/observer/matrix pins were
rewritten to make this comparison pass.

## Evidence binding

The temporary transport head is `b3a9d7d6ddd87badd4d98738bda7e6d7848f09c9`.
It runs no product or Cargo command. Authenticated source/transport metadata and
complete job logs bind each artifact, frame order, gzip/JSON length and hash,
every transported file and the extracted observer hash. Offline audit
re-extracts each authenticated log into a new directory before validating the
full reports against original manifests and source/build/Cargo/process receipts.
An independent verifier repeated the complete audit and obtained the same nine
full-row/raw-report hash tuples.

| Retained item                                  | SHA256                                                             |
| ---------------------------------------------- | ------------------------------------------------------------------ |
| First artifact, service-reported               | `8efb6ccd3b4d54ae2bbdc66e6118d8b6af2774eaa73d27b2658a7120b64e79a3` |
| Second artifact, service-reported              | `49a31c8f9c9092e9887cea321e9fba7f978fe307fe16b4c9befac8c3cc8f3f70` |
| First authenticated transport log              | `203140c4a90b72015c5a4bf600a45c4ec53066829d3ce4c24445425319b34c1f` |
| Second authenticated transport log             | `ae5da6483d3192feb7b639e07deed0356b44edf4b22d9f746b07fb23f329dc4f` |
| Two-run audit receipt                          | `67b16600a5bd3e7da4b78de207fe27f9be7a51c565b7616cb901ab7ec710f0c3` |
| Independent two-run audit receipt              | `f8557906cd3f3148d4190ddc062630dc97eaed1782adf06babffa8ce380b8271` |
| Authenticated complete second-run job metadata | `d617dab699324392f933ce4f8c8ffa08c59b8ea840d4a56a54507e378f11e840` |

Each original artifact contains 12 complete nonbinary files; the transport
preserves all of them through 27 frames and rechecks the frozen observer binary
hash without executing it. The uploader retained only
`target/differential/formatter-api`. Historical CLI reports carry their actual
embedded build receipt. The standalone CLI receipt and ordinary shared CLI raw
report were not uploaded, are not reconstructed and receive no raw-report
acceptance. Artifact digests above come from authenticated GitHub metadata;
literal ZIP bytes were not obtained or independently hashed.

The complete first workspace Rust log additionally maps 118 retained-law
references to 110 unique target/module-qualified passed tests. Exact original
cc87/source331/reviewed712 raw signatures and bodies agree. This is historical
named-test evidence. Selected-target Cargo JSON and compiled binary hashes were
not recovered for that broad workspace run, and the controlled current campaign
recipe was not executed. Engineering controls receive zero acceptance from it.

## Remaining gate

Exact current-head API/CLI execution, new #7258 import-sorting API/config/CLI
and Vite obligations, independent engineering controls, selected current law
receipts, protected merge-group validation and actual publication/merge remain
unfinished. #6882 stays open. Native handled/equivalent counts remain zero;
these historical observations do not admit a native formatter route.
