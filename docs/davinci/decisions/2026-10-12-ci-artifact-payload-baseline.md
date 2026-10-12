# CI artifact payload baseline

This investigation preserves every qualification check and complete observation.
The measured artifacts belong to main source
`a49936db24d9540e0630b42474654190d575294b`, Check run `38155233389`.
That run was subsequently cancelled after main advanced; these measurements
never establish a successful full Check or authorize a release.

Authenticated artifact metadata and complete ZIP central directories establish:

| Artifact                                |     ZIP bytes | Entries | Declared decoded bytes |
| --------------------------------------- | ------------: | ------: | ---------------------: |
| Formatter corpus `11709550349`          | 1,758,920,917 | 142,033 |          7,807,255,030 |
| Original oxlint locations `11708299152` | 1,860,300,449 |  20,238 |         13,644,002,299 |

The respective API digests are
`8919e837cb987136b73c249e2f6c5bed9f4f265f1c14b5df98cbf16edf6039c3`
and `8bec20ddbc446ebfe19642c92ac70a0435cb3e5b17cad62e68e7631a35980e35`.
Reading the complete directories required 19,935,208 and 3,449,169 bytes,
without downloading every file body. Directory sizes and CRCs are metadata;
they do not cryptographically authenticate equality of the whole contents.

The formatter directory contains 90 TypeScript native `lib/tsc` files, each
24,101,026 bytes, with identical declared CRC `0cdb754a` and compressed size
9,126,861 bytes. Together they occupy 821,417,490 compressed bytes and
2,169,092,340 decoded bytes. Three complete files were independently decoded,
CRC checked and SHA256 hashed, with identical
`4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4`:

- `differential/lsp-bind-style-code-actions/cli-project-zs6RmL/node_modules/@typescript/typescript-linux-x64/lib/tsc`
- `differential/lsp-bind-style-code-actions/invocation-2S5W2R/cli-project-HhBfP2/node_modules/@typescript/typescript-linux-x64/lib/tsc`
- `differential/lsp-bind-style-code-actions/invocation-2S5W2R/original-project-MlSPne/node_modules/@typescript/typescript-linux-x64/lib/tsc`

Across the complete formatter directory, repeated nonempty CRC/size signatures
suggest 1,296,353,453 compressed bytes beyond one copy per signature. This is a
candidate inventory, not a secure deduplication result or a measured speedup.
Every actual deduplication must hash and compare complete source bodies.

The oxlint directory instead retains nine distinct `source-native.node` bodies,
with different sizes and CRCs. They total 8,609,322,554 decoded bytes and
1,704,643,776 compressed bytes. Historical native builds cannot be treated as
one identical binary or discarded because their names are similar.

The next slice must measure lossless packing and restoration on the same source,
including CPU, wall time, upload/download, memory and total retained bytes. It
must preserve complete streams, source and binary identities, path inventory and
mutable-file independence. Missing, corrupt, substituted or incomplete evidence
must refuse restoration. Packing remains opt-in until its actual consumers and
whole original qualification succeed; no current default artifact is removed.

The live queue ruleset `24013395` has `ALLGREEN`, eight builds in flight, maximum
one merge, minimum one merge and zero wait. GitHub documents that merge limits
batch successful merges but do not combine merge-group builds. Raising the merge
limit alone therefore does not remove duplicate CI builds; it is not applied as
an unmeasured optimization. See [GitHub's merge queue documentation](https://docs.github.com/en/enterprise-cloud%40latest/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue).
