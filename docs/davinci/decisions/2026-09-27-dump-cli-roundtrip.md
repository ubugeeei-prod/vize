# Level dump CLI roundtrip slice

Issue: [#6832](https://github.com/ubugeeei-prod/vize/issues/6832).
Integrated on the canonical Dump graph; the original stage path/type branch is preserved.

Expose only `vize dump --level l1|l2|l3 --roundtrip FILE` in this slice.
Both flags are required. This reads existing UTF-8 input, binds a real typed
level API and checks byte identity after the roundtrip. It never writes the
input, executes a compiler pipeline, or replaces compiler output.

- L1 uses the actual Vue-template surface parser and lossless render. Its
  recoverable diagnostics/holes do not prevent fidelity success. This is
  not a claim that the source is semantically valid.
- L2 and L3 use their typed dump page's parse and Full print APIs. Success
  means canonical Full bytes, not semantic IR verification. Dump documents
  are never replayed as inputs to another compiler level.
- Exit 0 means byte identity; exit 1 means I/O, UTF-8, parse or mismatch
  failure; exit 2 means CLI usage failure. Mismatches report the first
  differing line and input/printed byte counts. Only success writes its
  summary to stdout; failures use stderr.
- Level selectors L0/L4 and unimplemented pipeline/all-levels/JSON options
  are rejected by the argument parser. They never return empty success.

The CLI uses existing dependencies and canonical
`vize_davinci::dump::{Dump, Error, Mode}` with per-level `dump::Page` aliases.
The original branch is preserved. No new level-to-legacy dependency or
runtime/serialized protocol change belongs in this slice.

Source-built CLI fixtures use `CARGO_BIN_EXE_vize`. They cover real L1
Unicode/CRLF/recoverable input, the committed L2 Full reference, a live L3
program snapshot, canonical mismatches, malformed/wrong-level/Display L2
input, missing/invalid UTF-8 files, required flags and unsupported options.
L3's derived page currently prints identical Full and Display bytes; mode
origin cannot be inferred from identical text. Validation checks actual
Full canonical bytes, rather than claiming to detect that origin.

Source-built CLI fixtures and the renamed library/binary compile gates must
pass. Fresh source CI remains mandatory; local formatting and pure
metadata/source checks are separate evidence.

## Exact CLI output oracles

The original CLI slice is preserved as `355731309e4057f209e5ad29b7a7c5d22fe6dfe7`.
Its clean replay is `b699c44faaccc08c229d107e3b355314f8cdf90d`. A temporary
test-only probe at `526d013d0cdd6e88a5d673555c45ec0738bd8148` ran the unchanged
eight tests and retained all 20 actual invocations, inputs, output bytes and exits.
The selected default-feature Darwin ARM CLI had SHA256
`b2e11ebf9096d92be167d7a7ef48d10029e9f0b0a7e38540fa505a9d307365b6`.

Thirteen complete observed output bodies replace nine substring assertions.
Five error bodies map only their one owned temporary input path to the next
run's path. Messages, spaces, newlines, byte counts and exit codes stay exact.
The probe is removed. Its raw JSONL and source/build/execution receipts remain
separate from the final oracles in the local CLI replay evidence archive.

Actual `vize --help` and the existing unit's `write_long_help` have different
option layouts. Their captures stay separate. Only the actual unit's failing
`.snap.new` updates its snapshot; CLI help is never reformatted into that oracle.
Fresh composed Actions and other platform output checks remain required.

The existing optimizer debug binary stays available. Its Croquis, budget,
remarks and pipeline migration, shared production capture and playground
generator remain unfinished. #6832 stays open; #6833 remains blocked until
#6832 is complete.

## Replay after repaired CI and canonical Dump APIs

This locally prepared layer follows `9672e648a`. The CLI source patch has the
same stable patch id as its original (`67a4b3d9c1d29427e107def7cbffdffcc6e2c2e8`
for `crates/vize`); the only conflict preserves all links in the central
record. Observation and final oracle patches retain their complete stable
patch ids `2cfa60e25281c218fc9323689dc78e68254d59a4` and
`e5fe8906128687e85b42cbba2e9f7c09dcf7dbdc`.

At the prepared composition, 34 existing focused dependency, workflow,
module, storage, stability and Dump-boundary contracts pass. Locked/offline
metadata has 38 workspace packages and 783 targets and registers exactly one
`dump_roundtrip_cli` integration test with no required feature. Cargo format,
Markdown format, whitespace and changed-source 350-line checks pass. Neither
the source-built CLI tests nor fresh Actions were run on this composition;
the earlier historical binary and output captures remain separate evidence.

The pipeline/all-level CLI and shared production capture are active follow-up
slices. Their current native availability must be reported explicitly;
roundtrip success does not certify native product completion. These decisions
are mirrored in the [#6832 record](https://github.com/ubugeeei-prod/vize/issues/6832#issuecomment-5854158632).
