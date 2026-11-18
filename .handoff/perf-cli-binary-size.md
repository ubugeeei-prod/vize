# perf: CLI binary size (handoff 2026-11-18)

Goal: release CLI (ci-opt, stripped) <= 79,818,008 B (the v0.779.0 source, 6030db68). Main 403207f2 builds 80,382,616 B on Linux (A/A run 44293733318). The per-stage instruction counts must not regress.

## Findings

Symbol diff of ci-opt arm64 builds; v0 mangling names the crate that instantiates each generic.

- **#13307** (+264,640 B on Linux): `vize_patina/src/markup/l2.rs` and `vize_atelier_vapor/src/l3.rs` call the generic `lower_with_caps_and_comment_policy::<Vec<ProvenanceRecord>>` directly. That compiles a whole extra copy of the L1->L2 lowering in `vize_patina` (~103 KB arm64) and in `vize_atelier_sfc` (~100 KB). The table-lowering change itself is small.
- **Copies that predate it**: a recording-lowering copy in `vize_atelier_dom` (capture path, 105 KB); `NoProvenance` lowering copies in `vize_l4` and `vize_atelier_dom` (~62 KB each); and pass copies (vslot, vmodel, legalize) in `vize`, `vize_curator`, `vize_l4` and `vize_atelier_dom` (~130 KB in all).
- **#13248** (+237,952 B on Linux): its own code is only ~60-80 KB arm64, of which ~45 KB is stable-sort instantiations over Digest, Finding, Located and &Diagnostic. The rest is ThinLTO/CGU repartition noise spread across crates (oxc_ast_visit, hashbrown and others move by ±).
- **Biggest cold offset: the LSP CSS catalog** (`vize_maestro/src/ide/style/data`). It holds about 30k `&'static str` and slice pointers. On Linux each costs 16 B plus a 24 B `.rela.dyn` entry, so about 1.2 MB of the binary. The v0.781.0 linux-gnu release has 1.9 MB of `.rela.dyn` and 1.58 MB of `.data.rel.ro`.

## Branches

| branch | content | state |
| --- | --- | --- |
| perf/cli-binary-size (variant A) | all lowering and pass instances in `vize_l1_to_l2`; non-generic entries plus a sealed `PassSink` | PR #13345 **closed**. Linux -937,344 B (79,445,272), but 38 of 107 instruction stages got worse (dom compile +0.1-0.5%, vapor +0.4-1.3%, patina +0.59%) |
| perf/l1-to-l2-cold-instances (variant B) | cold copies only: `lower_recorded_with_caps_and_comment_policy` for capture, `pass/recorded.rs` for `run_transform_with_pass_hook` | pushed, **no PR**. arm64 -164 KB, CLI output byte-identical. Instruction run 44299606196 and perf/size run 44299633426 still running |
| perf/lsp-css-catalog-compact | catalog as u16 text ids, `Run(u16,u16)` and flat tables; the generator `tools/commands/fixtures/generate-css-catalog.ts` is rewritten | WIP commit a840dcdb52. Custody semantic export byte-identical to main; clippy and tests not run yet |

## Next steps

1. Variant B: compare run 44299606196 with main run 44293727660, using `iccmp.py` logic: diff `runs[].<stage>.instructions` in each artifact's measurement.json. If every stage is <= main, open PR `perf(l1-to-l2)`. Read the Linux size from run 44299633426 (`sourceBuildReceipts` binaryBytes).
2. CSS catalog: run `cargo clippy -p vize_maestro --profile ci -- -D warnings -D clippy::wildcard_imports`, fix the maestro tests under `ide/style/tests`, then run `node --test tests/tooling/lsp-css-catalog.test.ts` and the storage, source-length and assertion-lint gates. Build ci-opt and check the size: Linux should drop by about 1 MB. Open PR `perf(lsp)`. maestro is linked into no instruction benchmark, so it carries no instruction risk.
3. Byte-identity harness, to recreate on the new machine: build, lint and dump for dom/ssr/vapor on misskey, vuefes and the repo test SFCs with `RAYON_NUM_THREADS=1`, normalizing timings only. Add the #13248 digest fixtures from `crates/vize/tests/diagnostic_digest_hints_cli.rs` (build strict and check, with `CORSA_PATH`).
4. Hot copies (patina, vapor, DOM `NoProvenance`): moving them costs instructions, so leave them.
