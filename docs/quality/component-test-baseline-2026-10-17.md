# Component test baseline — 2026-10-17

Static inventory: `c83021d545c76cd5cf5ee7e15e6ebdeaf533f35a`, **30,810 declarations in 6,743 files**. These are recognized declarations, not executed tests. Loops, parameterized cases and fixture harnesses can execute more checks; unsupported test declaration syntax can be missed. Vendor declarations are listed separately from project components.

Measured Rust coverage: [successful scheduled Check run 38871318436](https://github.com/ubugeeei-prod/vize/actions/runs/38871318436), commit `b4be3b70977b80a689c8e5ada62d05283e86b868`, artifact `rust-source-coverage-summary/source-summary.json`. This older measurement is **not coverage of the inventory commit**. Counts sum instrumented file totals, including instrumented test code; they are not production-only coverage. Missing metrics mean unmeasured, not 0%. LLVM branch totals in this artifact are zero, so it provides no branch-coverage conclusion.

## Reproduce

From a checkout containing the updated inventory tool, use `--root` to scan a separate checkout at the inventory SHA (omit it to scan the current checkout). Downloaded coverage is joined only when its full measured SHA is supplied explicitly.

```sh
gh run download 38871318436 --name rust-source-coverage-summary --dir /tmp/vize-source-coverage
node tools/benchmarks/scripts/test-inventory.ts \
  --root /path/to/checkout-at-c83021d545c76cd5cf5ee7e15e6ebdeaf533f35a \
  --source-coverage /tmp/vize-source-coverage/source-summary.json \
  --source-sha b4be3b70977b80a689c8e5ada62d05283e86b868 \
  --json /tmp/component-tests.json --markdown /tmp/component-tests.md
```

The existing `report-test-inventory` Actions action runs this same tool, so its JSON artifact and job summary now include component counts without adding another CI job. Scheduled/manual `source-coverage` supplies the separate LLVM artifact. Component names use the first two path segments; integration suites remain assigned to their containing suite. Empty workspace components are retained.

## Volume and measurement

| Component                       | Test files | Static declarations |       Measured lines |     Measured regions |
| ------------------------------- | ---------: | ------------------: | -------------------: | -------------------: |
| crates/davinci_harness          |          0 |                   0 |     395/590 (66.95%) |     666/977 (68.17%) |
| crates/vize                     |        463 |                1481 | 24617/28252 (87.13%) | 37088/42506 (87.25%) |
| crates/vize_armature            |         30 |                 190 |   3293/3468 (94.95%) |   5021/5296 (94.81%) |
| crates/vize_atelier_core        |         85 |                 472 | 13677/16839 (81.22%) | 21958/27233 (80.63%) |
| crates/vize_atelier_dom         |        126 |                 343 |   2489/2817 (88.36%) |   3242/3622 (89.51%) |
| crates/vize_atelier_jsx         |         76 |                 431 |   5506/5724 (96.19%) |   8140/8549 (95.22%) |
| crates/vize_atelier_sfc         |        222 |                1040 | 20124/23449 (85.82%) | 30339/35782 (84.79%) |
| crates/vize_atelier_ssr         |         41 |                 210 |   8965/9715 (92.28%) | 14733/16209 (90.89%) |
| crates/vize_atelier_vapor       |        142 |                 500 | 13704/15860 (86.41%) | 21107/24214 (87.17%) |
| crates/vize_canon               |        593 |                2186 | 59078/65582 (90.08%) | 85542/96111 (89.00%) |
| crates/vize_carton              |         32 |                 165 |   2248/2622 (85.74%) |   3420/3934 (86.93%) |
| crates/vize_croquis             |        117 |                 526 | 24498/28337 (86.45%) | 35271/40935 (86.16%) |
| crates/vize_croquis_cf          |         77 |                 257 | 14145/15738 (89.88%) | 20181/22325 (90.40%) |
| crates/vize_curator             |         16 |                  53 |   2062/2238 (92.14%) |   3540/3782 (93.60%) |
| crates/vize_doctor              |         20 |                  95 |   3239/3617 (89.55%) |   4275/4760 (89.81%) |
| crates/vize_fresco              |         48 |                 261 |   5398/6713 (80.41%) |   7907/9847 (80.30%) |
| crates/vize_glyph               |        199 |                 783 |  9792/10477 (93.46%) | 15857/17072 (92.88%) |
| crates/vize_incremental         |          8 |                  34 |   1423/1559 (91.28%) |   2238/2464 (90.83%) |
| crates/vize_maestro             |        423 |                1683 | 43950/52028 (84.47%) | 66693/79635 (83.75%) |
| crates/vize_marquette           |         21 |                  87 |   2627/2997 (87.65%) |   3428/3851 (89.02%) |
| crates/vize_musea               |         25 |                 115 |   3238/3684 (87.89%) |   5008/5626 (89.02%) |
| crates/vize_patina              |        614 |                3955 | 56148/61553 (91.22%) | 84690/93217 (90.85%) |
| crates/vize_relief              |          9 |                  70 |   1151/1261 (91.28%) |   1632/1819 (89.72%) |
| crates/vize_vitrine             |         38 |                 139 |   3539/4215 (83.96%) |   5615/6791 (82.68%) |
| davinci/vize_dialect_moonbit    |          8 |                  34 |     811/960 (84.48%) |   1213/1445 (83.94%) |
| davinci/vize_extension_contract |          1 |                   1 |    990/1100 (90.00%) |   1695/1930 (87.82%) |
| davinci/vize_extension_host     |         16 |                  59 |     196/492 (39.84%) |     348/777 (44.79%) |
| davinci/vize_guest              |          0 |                   0 |     195/213 (91.55%) |     360/373 (96.51%) |
| davinci/vize_l0                 |         74 |                 402 |   7924/8568 (92.48%) | 12155/13200 (92.08%) |
| davinci/vize_l0_derive          |          2 |                   9 |     224/232 (96.55%) |     339/349 (97.13%) |
| davinci/vize_l1                 |        134 |                 766 | 12210/12767 (95.64%) | 17672/18710 (94.45%) |
| davinci/vize_l1_to_l2           |        166 |                 937 | 24583/26995 (91.07%) | 38089/42421 (89.79%) |
| davinci/vize_l2                 |        165 |                 578 | 11474/12243 (93.72%) | 16785/18713 (89.70%) |
| davinci/vize_l2_to_l3           |         24 |                  79 |     613/647 (94.74%) |   1004/1072 (93.66%) |
| davinci/vize_l3                 |         49 |                 192 |   5367/5827 (92.11%) |   7682/8530 (90.06%) |
| davinci/vize_l4                 |         57 |                 152 |   3593/4037 (89.00%) |   5918/6798 (87.06%) |
| docs/theme                      |          3 |                  22 |           unmeasured |           unmeasured |
| examples/vite-musea             |          1 |                   4 |           unmeasured |           unmeasured |
| npm/builder                     |        188 |                 723 |           unmeasured |           unmeasured |
| npm/cli                         |         12 |                  58 |           unmeasured |           unmeasured |
| npm/compose                     |        202 |                1093 |           unmeasured |           unmeasured |
| npm/devtools                    |          2 |                   7 |           unmeasured |           unmeasured |
| npm/extension-sdk               |          0 |                   0 |           unmeasured |           unmeasured |
| npm/framework                   |         47 |                 261 |           unmeasured |           unmeasured |
| npm/fresco                      |          7 |                  46 |           unmeasured |           unmeasured |
| npm/fresco-native               |          0 |                   0 |           unmeasured |           unmeasured |
| npm/marquette                   |         10 |                  44 |           unmeasured |           unmeasured |
| npm/mcp-musea                   |          7 |                  17 |           unmeasured |           unmeasured |
| npm/native                      |          0 |                   0 |           unmeasured |           unmeasured |
| npm/oxlint                      |         40 |                 185 |           unmeasured |           unmeasured |
| npm/plugin-sdk                  |          0 |                   0 |           unmeasured |           unmeasured |
| npm/ui                          |        514 |                2826 |           unmeasured |           unmeasured |
| npm/wasm                        |          0 |                   0 |           unmeasured |           unmeasured |
| playground/e2e                  |         26 |                 149 |           unmeasured |           unmeasured |
| playground/src                  |         30 |                 103 |           unmeasured |           unmeasured |
| tests/_fixtures                 |         16 |                 161 |           unmeasured |           unmeasured |
| tests/_helpers                  |          1 |                   1 |           unmeasured |           unmeasured |
| tests/app                       |          9 |                  47 |           unmeasured |           unmeasured |
| tests/davinci_test_support      |          4 |                  28 |           unmeasured |           unmeasured |
| tests/external-consumers        |          1 |                   1 |           unmeasured |           unmeasured |
| tests/fixtures                  |         46 |                1286 |           unmeasured |           unmeasured |
| tests/performance               |          3 |                   3 |           unmeasured |           unmeasured |
| tests/snapshots                 |         84 |                 120 |           unmeasured |           unmeasured |
| tests/tooling                   |       1257 |                4709 |           unmeasured |           unmeasured |
| tests/vize_test_runner          |          3 |                   3 |           unmeasured |           unmeasured |
| tools/benchmarks                |         24 |                 143 |           unmeasured |           unmeasured |
| tools/commands                  |         12 |                  35 |           unmeasured |           unmeasured |
| tools/support                   |         36 |                 139 |           unmeasured |           unmeasured |
| vendor/lightningcss             |         13 |                 154 |           unmeasured |           unmeasured |
| vendor/oxc_formatter            |          6 |                  53 |           unmeasured |           unmeasured |
| vendor/oxc_parser               |         14 |                  92 |           unmeasured |           unmeasured |
| vendor/oxlint_html_renderer186  |          1 |                   1 |           unmeasured |           unmeasured |
| vendor/parcel_sourcemap         |          3 |                  11 |           unmeasured |           unmeasured |

## Priorities from the evidence

Glyph has 783 static declarations and 93.46% measured line coverage; Maestro has 1,683 declarations and 84.47%. Test volume alone does not support treating the formatter as the least tested component. LSP has substantially larger measured gaps. These percentages are historical signals for investigation, not current acceptance gates.

| Surface                           | Measured lines | Behavioral follow-up                                                                                                                                |
| --------------------------------- | -------------: | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| LSP implementation navigation     |           0/94 | Real initialized checker request, authored location mapping, and unavailable checker response.                                                      |
| LSP lexical completion visibility |           0/75 | Generated helper exclusion, authored shadowing, and member-access completion retention.                                                             |
| LSP linked editing handler        |           0/20 | Matching opening/closing component tags; mismatched names and non-template positions return no range.                                               |
| LSP workspace folder events       |           0/15 | Add/remove folders, refresh roots, and retain open-document ownership.                                                                              |
| LSP CSS module completions        |           6/67 | Module names and class keys, missing module and Unicode positions.                                                                                  |
| Formatter CSS engine boundary     |            3/9 | Panic conversion and recovery on a subsequent formatting request. Added inline regression tests exercise success and borrowed/owned panic payloads. |

The focused formatter tests cover a concrete stability boundary: an upstream CSS engine panic must become `StyleFormatError`, and the next request must still succeed. They do not establish coverage gains until fresh instrumented Actions results are available.

For LSP follow-ups, prefer real protocol/provider assertions that exercise the measured route and reject false positives. A zero measured line count can also indicate a disabled feature or a route absent from that campaign; inspect the current feature selection before claiming the feature has no tests.
