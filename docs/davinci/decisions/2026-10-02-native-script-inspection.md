# Explicit native script Program inspection

Tracked with [#6836](https://github.com/ubugeeei-prod/vize/issues/6836).
This consumer depends on the actual ordinary Program provider and the pinned
parser recovery repair; its branch starts at the provider's real head.

The standalone command is:

```sh
vize dump --level l1 --script tsx --roundtrip component.tsx
vize dump --level l1 --script js --script-goal script --roundtrip input.js
```

`--script` explicitly selects JS, TS, JSX or TSX. `--script-goal` selects
Module or Script, with Module as the default. Both require the existing L1
roundtrip mode; a different level is rejected before reading a file.
Language selection does not depend on the filename and no unambiguous parse
or semantic-analysis pass is introduced.

The command constructs one validated authored source and calls the native
Program provider once. It prints actual direct statement/directive kinds and
source spans, retained comments and full diagnostic metadata/labels, then
checks that the retained authored bytes equal the input. It does not walk a
recursive AST merely to generate this inspection. Syntax holes are printed
with their real diagnostics; source fidelity can still succeed for malformed
input. The command preserves all input bytes, including bytes the parser did
not reach after a fatal syntax error. The comment view contains the comments
that the parser actually retained.

The provider owns its actual OXC Program, source/profile identity and ordinary
Drop diagnostics until inspection ends. The consumer copies the byte-fidelity
result and facts into L0 strings, drops that owner before its arena, and never
reparses or serializes between levels. Existing template, L2/L3 dump and
all-level product feed routes retain their contracts.

Three scoped inspection laws cover real JS/TS/JSX/TSX statement children above
the former Program scaffold, malformed Unicode and every UTF-8 prefix, and
ordinary diagnostic retention. Separate source-built CLI laws cover explicit
language/goal options, observable syntax holes, exact authored-file bytes and
invalid level/options. Complete expected stdout/stderr and the updated help
snapshot come from the actual source-built hosted capture.

The bounded `Native Script CLI Capture` workflow can be opted into on the PR
with the `native-script-capture` label before its first merge, or dispatched
once registered on the default branch. It retains the exact head, authored
input, complete actual help and module/script inspection outputs. Hosted
capture is the source of updated complete CLI fixtures; local inspection laws
are not a substitute for a full source-built CLI check.

The same opt-in capture records complete stdout, stderr and exits for the four
actual language fixtures, malformed Script input and rejected option/level
combinations. Its bounded lossless file exporter preserves raw bytes, source
identity and per-file/pack SHA256 in framed logs plus a separate gzip artifact.
The original complete ZIP artifacts remain retained; no partial-match oracle
or help reconstruction is admitted when the artifact download route is blocked.

The successful [hosted capture](https://github.com/ubugeeei-prod/vize/actions/runs/36919093494)
retained 34 files and 5,185 exact source/output bytes. Its validated log frames
recovered the identical 4,220-byte gzip pack, SHA256
`19d243925785e48b5f118398534d684469b3f7657822dc871167acb2a2d2ad25`.
The signed PR merge checkout `6e79572df5729e80861ab2d93f56e65b529d27ee`
has the reviewed consumer `712afb0895322cd5b2ddb2e56a15e8317f205071` as
its second parent. The retained 1,454-byte help agrees with the independent
earlier full-CI observation, and the previous 1,126-byte help remains beside
the active fixture. `dump_script_cli/provenance.json` records complete case
hashes, exits and source identity. CLI laws compare the entire four-language
and recovering output, including diagnostics, and all three rejected errors;
only the single authored temporary input path is substituted. Fresh full
source-built checks and strict instruction gates remain required.

TODO: retain the wrapped-shape admission limits, then connect real file
language/container identity and typed native script Documents through actual
provider APIs. This standalone Program observation is not a product path
replacement or full native end-to-end completion. The issue remains open.
