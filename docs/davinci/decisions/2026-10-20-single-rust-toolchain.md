# One Rust toolchain for development and measurement

The maintainer decided that the repository keeps a single Rust pin
("rust-toolchain も2種類必要ない"). `rust-measurement-toolchain.toml` is removed;
instruction-count and paired-memory collectors read `rust-toolchain.toml`
(Rust 1.99.1). Any future toolchain bump re-baselines those measurements in the
same PR. The workspace MSRV (1.99.0), the SemVer lane and historical source or
capture replays keep their literal compilers.

## Instruction re-baseline

The level (100 rows) and formatter (4 rows) registries move their compiler identity
to `rustc 1.99.1 (58cf8f9c3 2026-10-13)` and their evidence to source
`e40813b3306bf85e5f61d96b19ebd432cae10334`, run
[39370420052](https://github.com/ubugeeei-prod/vize/actions/runs/39370420052).
Rows, inputs, windows, non-rustc method fields and every numeric cap are unchanged.

The 1.99.0 control is the merge-queue measurement of
`3ee8f6cdaf638792a065fc94ef44c3f87eb51074`, run
[39369172235](https://github.com/ubugeeei-prod/vize/actions/runs/39369172235).
The 1.99.1 source differs from it only in toolchain manifests, workflows and
measurement tooling; no measured Rust source or lockfile changes. All three
executions on each compiler are identical. 88 of 100 level rows and all four
formatter rows count the same instructions; twelve SSR/DOM compile rows move by
at most 0.08%, and every 1.99.1 count stays below its existing cap:

| Row                               |      Cap |  1.99.0 |  1.99.1 |
| --------------------------------- | -------: | ------: | ------: |
| atelier_ssr_compile_stress-deep   |  2552433 | 2475573 | 2477526 |
| atelier_ssr_compile_stress-interp | 10001618 | 9594984 | 9596447 |
| atelier_ssr_compile_medium        |  1804294 | 1681923 | 1682906 |
| atelier_ssr_compile_large         |  4151138 | 3964223 | 3965160 |
| atelier_ssr_compile_small         |   172042 |  166621 |  166728 |
| atelier_ssr_compile_stress-wide   |  3401389 | 3113317 | 3113388 |
| atelier_dom_codegen_stress-wide   |   745254 |  718407 |  718395 |
| atelier_dom_compile_medium        |  1125209 | 1108062 | 1108050 |
| atelier_dom_compile_large         |  4738050 | 3577012 | 3576980 |
| atelier_dom_codegen_large         |   913938 |  795948 |  795852 |
| atelier_dom_codegen_stress-deep   |   816016 |  366328 |  366232 |
| atelier_ssr_codegen_large         |   870694 |  811361 |  811265 |

The ratchet admits exactly this 1.99.0 to 1.99.1 step: the authenticated 1.99.0
registry, the same rows and input identities, and caps that only stay or tighten.
The complete 1.99.0 registries and 1.99.1 three-run packets are retained as
fixtures beside the 1.98 transition controls.

## Paired memory

The memory comparison has no committed numeric ceilings. Both arms now build
with the contributor pin, and the source judge compares the complete
`rust-toolchain.toml` blob of both arms. A retired measurement manifest in an
older arm no longer selects a compiler.
