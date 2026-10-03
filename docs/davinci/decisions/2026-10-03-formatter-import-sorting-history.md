# Import-sorting public API history (#6882 / #7258)

The merged #7258 feature is supplementary to the immutable original formatter
fix audit. Register its public API behavior in a separate source-built pack;
keep all original 300 declarations, source pins, goldens and receipts unchanged.
The new pack supplies legacy reference execution only. No default route changes,
native handling/equivalence, configuration, binding or Vite history credit follows.

## Real API and option observations

`formatter_observe` calls the existing `resolve_sort_imports` and then either
`GlyphFormatter::new_with_vue_version(...).with_sort_imports(...).format(...)`
or `format_script_with_sort_imports(..., SourceType::ts(), ...)`. The dedicated
API flags do not relabel the old `format_sfc` or `format_script` observations.
The existing exhaustive `FormatOptions` model is unchanged. The new option
probe reports complete effective ordinary options, whether sorting was supplied,
the exact raw setting and the complete pinned native resolver result. Omitted
settings and explicit false have separate source states even though both resolve
to `None`. Invalid settings expose the complete actual typed resolver error.

The independent 14-case manifest records input/output/option-probe SHA256 pins
and the complete current Rust test owner with its named source law. Its success
references reuse the authored whole-output assertions and UserCard reproduction
from the merged feature. Direct-script cases extract the same authored script
body; they are supplementary public observations, not original historical inputs.
The three error references were observed from the real source-built resolver.
No synthetic printer failure or source-registration count supplies execution.

Eleven successful cases cover UserCard enabled/false/omitted, ordinary TS/TSX,
custom groups and boundaries, side-effect/comment partitions, descending/internal
patterns, newline partitions and direct TypeScript enabled/false. Each compares
all output bytes in three actual formatting calls and retains SFC change verdicts.
Three invalid cases cover a leading boundary, unsupported order and explicit true;
they compare complete `ScriptFormatError` debug bytes, stderr and process status.
Normal validators reject altered API identity, missing option sources, probe drift,
source-owner/function drift, duplicate/omitted rows and invented native credit.

## Validation and source qualification

On private source `d1a5257a02fee898a9373907c29b50493c2c021f`, the normal build
selected the actual Cargo JSON example artifact, froze it, and retained raw build
logs, source/lock/toolchain/executable hashes and complete repeated process bytes.
The original 300 cases passed the unchanged audit. The separate new report passed
all 14 cases: 11 full-byte/fixed-point successes and three real typed errors.
Its SHA256 is `1ddaeceb8126f27a1356d427f8dab36339bcbb09117c992c1ed6610b439f24d4`.
Twelve focused contract/selection tests and strict observer Clippy also passed.
This local receipt qualifies only that source, not a later public or queue head.

The existing mandatory T1 formatter execution runs the additional pack using the
same frozen observer, with its own `import-sorting-report.json`. The original
300-case report set still enters the original validator separately. Existing
Actions uploads retain the full source/build/report bytes; PR selection keeps
real Cargo execution in T1 and the pure contract in T0. Exact public-head Actions,
protected composed execution and literal merge are pending.

## Remaining gates

Register the actual #7258 config/CLI, native/WASM and Vite+ feature behavior with
complete effective options and outputs; the earlier eight CLI observations remain
partial. Applicable Glyph instruction measurements and the real OXC printer-error
runtime arm also remain unfinished. #6882 stays open, all product defaults remain
unchanged and this pack's native handled/equivalent/paired counts remain zero.
