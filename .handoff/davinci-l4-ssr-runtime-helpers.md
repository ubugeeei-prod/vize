# Handoff: lane B (L3/L4 backends), paused 2026-11-19 ~21:30 JST

Delete `.handoff/` before merging. The measurement tools from the session scratchpad are copied to
`.handoff/laneb-tools/`. Their hard-coded paths point at the old scratchpad, so edit `IC`/`S` first.

## Rule in force (maintainer, 2026-11-19)

- Instruction counts are judged on the **release profile** (fat LTO, 1 CGU), strictly: every row must be ≤ base. ci-opt counts are informational.
- Size is judged on the release binary.
- "Release-shaped" means `[profile.ci-opt] lto = "fat", codegen-units = 1`. `lbrelcommit.sh` makes a measurement commit on top of any commit.

## Branches and PRs

| Branch | PR | State |
| --- | --- | --- |
| `davinci/l3-shared-decisions` | #13398 | merged (L3-1 record) |
| `davinci/l4-one-emit-document` | #13390, open, auto-merge off; leave as is | head `8567dd13a1` = variant v6 |
| `davinci/l4-one-emit-document-v8` | none | WIP variant v8 (`a7a33270e2`), the candidate to put on #13390 once the release rows are clean |
| `davinci/l4-ssr-runtime-helpers` (this branch) | none | L4-2, stacked on L4-1 v6; rebase it onto the final L4-1 |
| `davinci/sourcemap-js-line-terminators` | none | listed-change PR, stacked on L4-1 v6 (`4d3b7cdb9a`); open it after L4-1 merges |

## L4-1 (#13390) release-profile state

The release base is `9cf1ccf4b1`: run 44481946281, twin 44483432053, A/A 0 on all 101 rows. Every variant was grafted onto `6be122088b` (old head) so that all share that base.

| Variant | Release rows up | Note |
| --- | --- | --- |
| v6 (PR head) | 5, ≤ +0.11% | run 44492579701 |
| v7 (`35b30eb935`) | 2: `atelier_ssr_compile_medium` +226, `atelier_dom_compile_stress-deep` +125 | run 44495575103 |
| v8 (`a7a33270e2`) = v7 + prefilter in the `push_escaped` text loop | not measured yet | runs below |

Root causes found from the v7 disassembly probe (runs 44510157047 base / 44510162253 v7; `lbasm.py`):

1. `SsrCodegenContext::flush_push` is real code. The inlined escape loop costs 13 instructions per byte against base's 12.
   - v8 skips to the next byte that can open a pattern (`opens_escape` + `position`) instead of trying every pattern at every byte. That should cut the loop well below base.
2. `lower_children_guarded` (+125, untouched `vize_l1_to_l2`) and `decode_html_entities_inner` (+180) are **identical code in base and head**. Only RIP-relative addresses differ.
   - Their self cost still changed, so the extra instructions are address-dependent. Most likely cause: fixture alignment.
   - The harness embeds fixtures with `include_str!` (`tools/benchmarks/crates/davinci_harness/src/fixtures.rs`), so any `.rodata` layout change moves them. core's `from_utf8`, `memchr` and `str::find` have alignment-dependent prologues; Croquis showed `core::str::from_utf8` +24 the same way.
   - This cannot be fixed in #13390's code. Proposed fix, which needs a harness-owner/maintainer decision because it re-records baselines: embed each fixture 64-byte aligned (`#[repr(align(64))]` wrapper around `include_bytes!`), or copy it to a heap buffer outside the measured window.

In-flight runs (dispatched before the pause, from `measure/laneb-probe2-*`). Both run the harness patched by `ic-patched2.mjs` (release-shaped), which also uploads `disasm/*.s` and `disasm/*.fixtures.json` (each fixture's file offset and offset mod 64):

- **44518541629**: base (`measure/laneb-probe2-base`)
- **44518545933**: v8 (`measure/laneb-probe2-v8`)

What to read from them:

- `lbic.py 44481946281 44518545933 44483432053 44518545933` gives the v8 release rows.
- Compare the `fixtures.json` `mod64` values base vs v8. If they differ while the +125 / +180 rows persist, the alignment hypothesis is confirmed. Take that to the maintainer as evidence for the harness fix.
- If v8 leaves only those alignment rows up: release size probe (base `vize` 57,219,936 B, vitrine napi 80,706,640 B at `a74306cee7`; build the head under `/private/tmp/claude-501/vize-release-build.lock`, only with ≥ 60 GB free). Then push v8 to #13390, update the body, and ask for a ruling on the alignment rows before enabling auto-merge.

## L4-2 (this branch)

- Byte-identical output on the 2,133-file corpus (DOM/SSR/Vapor). 170 test targets pass, and clippy plus the assertion lint are clean.
- ci-opt against its v5 parent: 12 `atelier_ssr_*` rows fall (up to −3.2%), 89 identical. The release-shaped run has not been done.
- Next: rebase onto the final L4-1, run release base/head and twins, write the PR body, and enable auto-merge if every release row is ≤ base.

## Source-map line terminators (`davinci/sourcemap-js-line-terminators`)

- Maintainer ruling: every backend breaks lines at LF, CR, CRLF, U+2028 and U+2029.
- Done in `4d3b7cdb9a`:
  - deleted atelier_core's `\n`-only `source_map.rs`, `source_map_anchor.rs` and `document_source_map`;
  - legacy DOM/SSR/Vapor, Vitrine plugin maps and atelier_jsx now use `vize_l4::write::source_map` (`line_starts` is now public);
  - the SFC module-map decoder uses JS line terminators;
  - added the SSR fixture `source_map_lines_break_at_every_javascript_line_terminator` and the SFC fixture `module_maps_break_lines_at_every_javascript_line_terminator`;
  - added ledger rows W-MAP-1..3 in `docs/davinci/plan/compile-waivers.md`.
- Local suites pass, except the env-only `vize_vitrine` `oxlint_html_profile` failure in git worktrees.
- Still owed:
  - a pinned compiler differential fixture under `tests/_fixtures/differential/compiler`, needed for a `fix(...)` title;
  - a Vapor-specific fixture;
  - CI clippy;
  - release measurement;
  - opening the PR after L4-1 merges.

## Later slices

- L4-3: native SSR still parses with armature first and runs the legacy transform, because callers read its errors. v-pre spans come from the legacy parser (`l4/frozen_slots.rs`). First do a parity slice that produces the SFC-visible errors from L1/L2 diagnostics.
- Then L4-4, L3-2 onward per #13398, L4-5 and L4-7.

## Open decisions

- Harness fixture alignment (above). Without it, any change that moves `.rodata` can raise untouched rows on the release profile.
