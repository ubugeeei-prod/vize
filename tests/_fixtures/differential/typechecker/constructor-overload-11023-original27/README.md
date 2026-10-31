# Original #11023 baseline

These are the original **27 distinct cases and 58 authored files**, not the later
26-case constructor admission campaign. `original-cases.ts` is the exact 5,155-byte
Git blob `d34ce4bf37ea26ac8a9120c8bd6f66c79346d86f` from
`190257dbe52a7d4942be4496403a55b32c24426f`. The framework and workflow fixtures
preserve the original runner at `0fb9c041bb5e4bfaa375a0d2dbc6e64508730ddb`.

`historical-public-summary.json` is the unchanged summary from run 40594743602,
artifact 12776280038 (archive SHA-256
`d379b27aa0c01297f52a4719fb5cb0d2126944e9673a96f900cea566592ecb1a`).
The input loader requires every reconstructed file's ordered size/hash record to
equal that summary, including its 772-byte `h()` control and three configurations.

Manually dispatch `canon-slot-constructor.yml` from the prepared harness revision
with an immutable `source_sha`. The initial current-main baseline is
`c34d95586eb9459560c9be4990ff9ebdba3bde61`. The workflow builds that source, retains
the complete original CLI/mapper vectors and all four-frame protocol packets,
and separately checks the untouched public 0.602.0 controls and VueTSC/TS6 oracle.
The source package uses the nine physical declarations from the named source;
only its mapper command and source-labelled package version are supplied by the
harness. Original public dependencies and executables must match the historical
identities. The newly resolved install lock is retained and explicitly compared
with the historical lock hash; lock equality is never assumed.

The output is a semantic baseline. Existing failures remain visible, and a green
capture job does not claim a repair, release acceptance, or performance result.
