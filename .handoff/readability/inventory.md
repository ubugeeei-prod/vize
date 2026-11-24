# Readability inventory (baseline)

The snapshot is `origin/main` `0a04b42431` (2026-11-24 18:28 JST), extracted with `git archive`, so only tracked files are counted.
Every number here is reproducible with the scripts in `scripts/` (see §9). The raw per-file rows are in `data/inv/files.tsv`: 25,740 rows, one per in-scope file, with 28 metrics each.

## 1. Scope

**In scope:** first-party code in `.rs .ts .tsx .mts .cts .js .mjs .cjs .jsx .vue .pkl .sh .nix .lua .scm .el .vim .css .scss`. Markdown is counted separately and has no comment metrics.

**Excluded:**
- `upstream/**` and `.github/**`. `.github` holds only generated YAML and shell: the 1,139 `.github/scripts/**/*.sh` files are rendered from `tools/config/pkl/scripts/**/*.sh.pkl`.
- Any path segment in `fixtures _fixtures __fixtures__ snapshots __snapshots__ expected generated test-fixtures testdata vendor i18n corpora typecheck-corpora dist target node_modules`.
- `tools/benchmarks/results/`, `tests/expected/`, `tests/fuzz/regressions/` and the translated docs (`docs/content/{ja,zh-CN,pt-BR,fr}`).
- The 38 files whose first lines say they are generated (`data/generated-files.txt`, 9,123 lines). The largest group is the CSS catalog in `crates/vize_maestro/src/ide/style/data/`.

**Result:** 23,080 code files, **3,483,845 lines**, of which 3,232,727 are non-blank. There are also 2,628 Markdown files (222,501 lines).

## 2. Headline numbers

| Metric | Value | How measured |
| --- | ---: | --- |
| Code files over 500 lines | **172**, 62 of them test files | line count; the repo gate counts more suffixes (`.json`, `.md`, `.yml`) |
| Code files over 1,000 lines | **27** | |
| Plain comment lines / doc-comment lines | 43,326 / 151,329 (1.3% / 4.7% of non-blank) | tokenizer in `inventory.py` (strings skipped) |
| Comment lines flagged redundant (union of the rows below) | **5,799** | heuristics, §8 |
| — issue-number lines (`#1234`, `issue 1234`, `PR 1234`) | 2,667 | exact |
| — history narration ("used to", "previously", "no longer", "for now", …) | 834 | about 50% precision |
| — restating the next code line (≤ 2 lines, ≥ 75% of content words appear in the next line) | 2,010 | about 75% precision |
| — section banners | 211 | exact |
| — commented-out code | 132 | low precision |
| Functions over 80 lines / of those with no blank line inside | 3,041 / **1,686** | brace matching on code-only text |
| Rust items directly after `}` with no blank line | 1,808 | |
| Lines over 100 columns | 29,100 | mostly prose strings and data |
| Code lines at ≥ 7 indent levels | 19,751 (Rust 10,135) | |
| Magic numbers in non-test code (literal not in {0,1,2,3,4,8,10,16,32,64,100,1000,…}, outside `const`/`static`) | 13,269 | |
| Repo-path string literals (`"crates/…"`, `"tools/…"`, …) | 16,361 | |
| Hash literals (40/64 hex) in code | 3,711 | |
| Files with an issue number in the name | **442** | `[-_]\d{3,6}` in the base name |
| Retired `.mjs` files | 657 (225 are `tests/tooling/*.test.mjs`) | |
| Byte-custody sources (current bytes pinned by hash) | **884** first-party files; about **480 enforced** by a test | `scripts/pins.py`, §5 |

### By language

| Language | Files | Lines | >500 | >1000 |
|---|---|---|---|---|
| rust | 10,964 | 1,932,388 | 81 | 15 |
| ts | 7,849 | 1,195,961 | 62 | 9 |
| md | 2,628 | 222,501 | 2 | 0 |
| js | 1,134 | 161,014 | 7 | 2 |
| vue | 1,079 | 119,052 | 22 | 0 |
| pkl | 1,846 | 71,775 | 0 | 0 |
| css | 45 | 6,441 | 1 | 1 |
| shell | 150 | 1,032 | 0 | 0 |
| lua / vim / nix / elisp / scheme | 45 | 3,845 | 0 | 0 |

(The language rows include the 38 generated files; the area tables below do not.)

## 3. Per-area tables

Column notes:
- **Comment %** is comment lines over non-blank lines, split into plain and doc.
- **Redundant est.** is the union of the flagged lines, as a share of comment lines.
- **Issue refs** and **History** count lines; **Restates**, **Banners** and **Commented code** count lines in flagged blocks.
- **Fns >80 (no blank)** counts functions over 80 lines, with the number that contain no blank line in parentheses.
- **Naming flags** are non-kebab TS files, non-snake Rust files, issue/version numbers in names, generic names, `.mjs`, numbered parts and stems over 48 characters. Codename paths are counted separately; the whole of `davinci/` counts because of the directory name.
- Magic numbers are 0 for test files by construction.

### By top-level group (code files, Markdown excluded)

| Area | Files | Lines | >500 | >1000 | Comment % (plain/doc) | Redundant est. (lines, % of comments) | Issue refs | History | Restates | Banners | Commented code | Magic nums | Repo-path lits | Hash lits | Fns >80 (no blank) | Longest fn | Naming flags | Codename paths |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `crates` | 8845 | 1,570,529 | 72 | 12 | 1.6 / 4.6 | 3,443 (4%) | 1,779 | 450 | 1,087 | 128 | 29 | 3,714 | 2,481 | 883 | 1644 (767) | 714 | 50 | 136 |
| `npm` | 4956 | 631,816 | 61 | 7 | 1.0 / 8.9 | 1,158 (2%) | 186 | 197 | 717 | 38 | 25 | 3,968 | 298 | 55 | 240 (69) | 547 | 196 | 0 |
| `tests` | 3403 | 546,632 | 8 | 1 | 1.2 / 0.7 | 471 (5%) | 356 | 72 | 4 | 3 | 42 | 0 | 8,736 | 1,798 | 364 (268) | 433 | 469 | 244 |
| `davinci` | 2048 | 326,161 | 1 | 0 | 1.1 / 7.5 | 334 (1%) | 120 | 63 | 130 | 19 | 3 | 732 | 143 | 9 | 301 (240) | 336 | 3 | 2048 |
| `tools` | 3203 | 317,256 | 25 | 6 | 1.3 / 1.9 | 319 (3%) | 208 | 41 | 38 | 10 | 28 | 4,175 | 4,316 | 930 | 398 (305) | 533 | 815 | 199 |
| `apps` | 501 | 74,274 | 5 | 1 | 0.8 / 1.2 | 45 (3%) | 4 | 4 | 26 | 7 | 5 | 364 | 317 | 7 | 70 (27) | 393 | 66 | 1 |
| `editors` | 121 | 16,984 | 0 | 0 | 2.3 / 1.1 | 27 (5%) | 12 | 7 | 8 | 6 | 0 | 316 | 58 | 29 | 24 (10) | 462 | 26 | 0 |
| `vite.config.ts` | 1 | 152 | 0 | 0 | 7.5 / 19.2 | 2 (5%) | 2 | 0 | 0 | 0 | 0 | 0 | 12 | 0 | 0 (0) | 0 | 0 | 0 |
| `flake.nix` | 1 | 33 | 0 | 0 | 0.0 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |
| `default.nix` | 1 | 8 | 0 | 0 | 0.0 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |

### crates/*

| Area | Files | Lines | >500 | >1000 | Comment % (plain/doc) | Redundant est. (lines, % of comments) | Issue refs | History | Restates | Banners | Commented code | Magic nums | Repo-path lits | Hash lits | Fns >80 (no blank) | Longest fn | Naming flags | Codename paths |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `crates/vize_canon` | 1931 | 325,908 | 11 | 5 | 1.7 / 4.0 | 868 (5%) | 649 | 94 | 124 | 2 | 5 | 722 | 522 | 736 | 388 (181) | 666 | 8 | 3 |
| `crates/vize_patina` | 1620 | 297,871 | 5 | 1 | 1.3 / 6.3 | 416 (2%) | 182 | 47 | 117 | 68 | 2 | 576 | 295 | 15 | 169 (107) | 374 | 5 | 3 |
| `crates/vize` | 1019 | 197,439 | 6 | 2 | 0.9 / 2.3 | 264 (5%) | 216 | 40 | 8 | 2 | 0 | 205 | 894 | 33 | 323 (131) | 392 | 7 | 6 |
| `crates/vize_maestro` | 1048 | 191,865 | 10 | 0 | 1.5 / 2.8 | 326 (4%) | 172 | 50 | 103 | 0 | 5 | 773 | 125 | 57 | 193 (75) | 368 | 4 | 19 |
| `crates/vize_atelier_sfc` | 587 | 104,033 | 15 | 1 | 1.8 / 3.8 | 224 (4%) | 107 | 39 | 57 | 9 | 14 | 69 | 288 | 16 | 138 (89) | 714 | 6 | 25 |
| `crates/vize_croquis` | 378 | 73,647 | 9 | 1 | 1.8 / 6.3 | 198 (4%) | 27 | 20 | 146 | 6 | 0 | 210 | 16 | 0 | 66 (21) | 280 | 4 | 3 |
| `crates/vize_glyph` | 382 | 54,658 | 2 | 0 | 1.7 / 3.8 | 126 (4%) | 97 | 20 | 10 | 2 | 0 | 30 | 86 | 5 | 61 (45) | 340 | 1 | 0 |
| `crates/vize_atelier_core` | 226 | 43,346 | 5 | 1 | 4.9 / 8.0 | 256 (5%) | 87 | 47 | 107 | 15 | 3 | 95 | 19 | 1 | 61 (14) | 425 | 6 | 5 |
| `crates/vize_atelier_vapor` | 278 | 43,276 | 1 | 0 | 1.8 / 4.2 | 113 (5%) | 23 | 11 | 79 | 0 | 0 | 78 | 69 | 0 | 48 (30) | 372 | 1 | 55 |
| `crates/vize_croquis_cf` | 187 | 32,905 | 0 | 0 | 2.0 / 5.0 | 88 (4%) | 3 | 5 | 64 | 16 | 0 | 86 | 1 | 0 | 48 (8) | 423 | 3 | 2 |
| `crates/vize_atelier_jsx` | 160 | 28,600 | 0 | 0 | 1.9 / 7.3 | 150 (6%) | 124 | 16 | 10 | 7 | 0 | 7 | 49 | 0 | 20 (10) | 305 | 0 | 0 |
| `crates/vize_vitrine` | 157 | 24,434 | 2 | 0 | 1.5 / 3.8 | 66 (5%) | 26 | 8 | 34 | 0 | 0 | 67 | 38 | 11 | 38 (18) | 290 | 0 | 0 |
| `crates/vize_carton` | 138 | 24,200 | 1 | 0 | 1.4 / 4.7 | 16 (1%) | 6 | 6 | 4 | 0 | 0 | 44 | 21 | 3 | 17 (11) | 187 | 3 | 1 |
| `crates/vize_atelier_dom` | 159 | 22,650 | 1 | 0 | 1.5 / 5.1 | 44 (3%) | 19 | 12 | 13 | 0 | 0 | 0 | 11 | 0 | 7 (3) | 182 | 0 | 4 |
| `crates/vize_atelier_ssr` | 128 | 22,605 | 2 | 0 | 1.6 / 5.2 | 51 (4%) | 18 | 10 | 23 | 0 | 0 | 15 | 21 | 1 | 25 (15) | 176 | 1 | 7 |
| `crates/vize_fresco` | 109 | 20,706 | 0 | 0 | 2.0 / 10.5 | 107 (5%) | 0 | 6 | 101 | 0 | 0 | 203 | 0 | 0 | 8 (1) | 324 | 0 | 0 |
| `crates/vize_armature` | 74 | 14,301 | 2 | 1 | 2.0 / 4.8 | 40 (4%) | 20 | 7 | 12 | 1 | 0 | 243 | 2 | 5 | 8 (3) | 145 | 0 | 2 |
| `crates/vize_doctor` | 69 | 13,566 | 0 | 0 | 0.1 / 8.3 | 6 (1%) | 0 | 2 | 4 | 0 | 0 | 43 | 0 | 0 | 2 (1) | 91 | 0 | 0 |
| `crates/vize_musea` | 44 | 8,695 | 0 | 0 | 2.1 / 9.6 | 48 (5%) | 0 | 5 | 43 | 0 | 0 | 77 | 5 | 0 | 4 (0) | 124 | 0 | 1 |
| `crates/vize_incremental` | 54 | 8,638 | 0 | 0 | 1.0 / 8.0 | 15 (2%) | 0 | 2 | 13 | 0 | 0 | 28 | 3 | 0 | 2 (1) | 105 | 0 | 0 |
| `crates/vize_marquette` | 46 | 7,924 | 0 | 0 | 0.1 / 11.9 | 5 (1%) | 0 | 2 | 3 | 0 | 0 | 40 | 13 | 0 | 12 (2) | 252 | 1 | 0 |
| `crates/vize_curator` | 28 | 4,970 | 0 | 0 | 0.8 / 6.2 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 63 | 3 | 0 | 6 (1) | 199 | 0 | 0 |
| `crates/vize_relief` | 23 | 4,292 | 0 | 0 | 2.0 / 13.4 | 16 (3%) | 3 | 1 | 12 | 0 | 0 | 40 | 0 | 0 | 0 (0) | 75 | 0 | 0 |

### davinci/*

| Area | Files | Lines | >500 | >1000 | Comment % (plain/doc) | Redundant est. (lines, % of comments) | Issue refs | History | Restates | Banners | Commented code | Magic nums | Repo-path lits | Hash lits | Fns >80 (no blank) | Longest fn | Naming flags | Codename paths |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `davinci/vize_l2` | 492 | 78,237 | 0 | 0 | 0.5 / 6.7 | 52 (1%) | 17 | 8 | 19 | 8 | 0 | 86 | 23 | 0 | 79 (77) | 214 | 0 | 492 |
| `davinci/vize_l1` | 393 | 67,075 | 1 | 0 | 1.1 / 6.7 | 65 (1%) | 15 | 13 | 32 | 5 | 0 | 144 | 15 | 0 | 59 (45) | 253 | 0 | 393 |
| `davinci/vize_l1_to_l2` | 384 | 60,357 | 0 | 0 | 1.8 / 6.5 | 49 (1%) | 26 | 15 | 3 | 2 | 3 | 54 | 64 | 9 | 43 (26) | 208 | 0 | 384 |
| `davinci/vize_l4` | 315 | 51,283 | 0 | 0 | 1.2 / 5.3 | 30 (1%) | 14 | 7 | 8 | 1 | 0 | 157 | 5 | 0 | 69 (56) | 220 | 3 | 315 |
| `davinci/vize_l0` | 247 | 38,052 | 0 | 0 | 1.3 / 16.4 | 124 (2%) | 47 | 16 | 59 | 3 | 0 | 269 | 12 | 0 | 13 (4) | 336 | 0 | 247 |
| `davinci/vize_l3` | 162 | 22,891 | 0 | 0 | 0.6 / 6.7 | 13 (1%) | 1 | 3 | 9 | 0 | 0 | 20 | 9 | 0 | 25 (21) | 154 | 0 | 162 |
| `davinci/vize_l2_to_l3` | 47 | 6,556 | 0 | 0 | 0.3 / 2.6 | 1 (1%) | 0 | 1 | 0 | 0 | 0 | 0 | 15 | 0 | 12 (11) | 161 | 0 | 47 |
| `davinci/vize_l0_derive` | 8 | 1,710 | 0 | 0 | 0.5 / 8.7 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 (0) | 91 | 0 | 8 |

### npm/*

| Area | Files | Lines | >500 | >1000 | Comment % (plain/doc) | Redundant est. (lines, % of comments) | Issue refs | History | Restates | Banners | Commented code | Magic nums | Repo-path lits | Hash lits | Fns >80 (no blank) | Longest fn | Naming flags | Codename paths |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `npm/ui` | 2640 | 309,417 | 29 | 1 | 0.7 / 10.0 | 335 (1%) | 4 | 80 | 238 | 6 | 7 | 2,412 | 15 | 0 | 64 (5) | 547 | 1 | 0 |
| `npm/builder` | 876 | 121,322 | 12 | 3 | 1.9 / 3.0 | 274 (5%) | 158 | 44 | 46 | 21 | 10 | 562 | 70 | 19 | 76 (29) | 383 | 63 | 0 |
| `npm/compose` | 659 | 90,894 | 14 | 3 | 0.9 / 18.1 | 386 (2%) | 0 | 52 | 331 | 0 | 3 | 561 | 0 | 0 | 29 (0) | 416 | 0 | 0 |
| `npm/linter` | 170 | 30,284 | 1 | 0 | 1.1 / 1.3 | 9 (1%) | 5 | 4 | 0 | 0 | 0 | 129 | 118 | 32 | 27 (20) | 376 | 38 | 0 |
| `npm/fresco` | 179 | 22,552 | 1 | 0 | 1.0 / 5.8 | 65 (5%) | 0 | 4 | 58 | 3 | 0 | 121 | 0 | 0 | 7 (0) | 428 | 73 | 0 |
| `npm/framework` | 148 | 20,399 | 2 | 0 | 1.6 / 5.3 | 10 (1%) | 2 | 2 | 3 | 0 | 3 | 30 | 6 | 0 | 3 (0) | 341 | 8 | 0 |
| `npm/cli` | 98 | 14,481 | 0 | 0 | 1.4 / 6.8 | 36 (3%) | 14 | 4 | 18 | 0 | 0 | 13 | 27 | 1 | 8 (4) | 151 | 2 | 0 |
| `npm/marquette` | 42 | 5,619 | 0 | 0 | 0.4 / 14.5 | 6 (1%) | 0 | 1 | 4 | 0 | 1 | 19 | 1 | 0 | 8 (0) | 162 | 0 | 0 |
| `npm/mcp-musea` | 30 | 4,043 | 1 | 0 | 0.3 / 2.5 | 9 (9%) | 0 | 0 | 0 | 8 | 1 | 26 | 0 | 0 | 4 (0) | 216 | 0 | 0 |
| `npm/native` | 19 | 2,731 | 0 | 0 | 1.2 / 1.2 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 36 | 35 | 0 | 3 (3) | 124 | 9 | 0 |
| `npm/typescript-plugin` | 11 | 2,262 | 0 | 0 | 3.7 / 0.5 | 1 (1%) | 0 | 1 | 0 | 0 | 0 | 12 | 0 | 0 | 7 (5) | 311 | 0 | 0 |
| `npm/plugin-sdk` | 15 | 2,260 | 0 | 0 | 1.2 / 9.9 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 15 | 0 | 0 | 1 (1) | 139 | 0 | 0 |
| `npm/volar` | 11 | 1,488 | 1 | 0 | 1.2 / 0.5 | 3 (12%) | 3 | 0 | 0 | 0 | 0 | 16 | 19 | 3 | 3 (2) | 186 | 0 | 0 |
| `npm/devtools` | 10 | 1,169 | 0 | 0 | 0.7 / 4.7 | 3 (5%) | 0 | 3 | 0 | 0 | 0 | 7 | 0 | 0 | 0 (0) | 46 | 1 | 0 |
| `npm/formatter` | 8 | 971 | 0 | 0 | 1.0 / 1.2 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 5 | 6 | 0 | 0 (0) | 54 | 1 | 0 |
| `npm/wasm` | 3 | 721 | 0 | 0 | 0.0 / 44.1 | 20 (7%) | 0 | 1 | 19 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 23 | 0 | 0 |
| `npm/create-fresco` | 10 | 703 | 0 | 0 | 0.6 / 7.9 | 1 (2%) | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 37 | 0 | 0 |
| `npm/fresco-native` | 3 | 310 | 0 | 0 | 1.4 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 4 | 1 | 0 | 0 (0) | 39 | 0 | 0 |
| `npm/entry` | 24 | 190 | 0 | 0 | 3.8 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |

### tools/*

| Area | Files | Lines | >500 | >1000 | Comment % (plain/doc) | Redundant est. (lines, % of comments) | Issue refs | History | Restates | Banners | Commented code | Magic nums | Repo-path lits | Hash lits | Fns >80 (no blank) | Longest fn | Naming flags | Codename paths |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `tools/support` | 769 | 129,853 | 15 | 1 | 1.5 / 1.7 | 179 (4%) | 131 | 19 | 4 | 8 | 20 | 1,816 | 1,907 | 637 | 205 (159) | 533 | 94 | 87 |
| `tools/benchmarks` | 476 | 98,079 | 3 | 1 | 0.7 / 3.5 | 84 (2%) | 47 | 14 | 16 | 2 | 6 | 1,360 | 466 | 211 | 162 (131) | 374 | 117 | 15 |
| `tools/config` | 1839 | 68,594 | 0 | 0 | 1.7 / 0.1 | 48 (4%) | 27 | 4 | 18 | 0 | 0 | 816 | 1,812 | 77 | 0 (0) | 24 | 594 | 65 |
| `tools/commands` | 95 | 20,006 | 7 | 4 | 1.0 / 1.9 | 8 (1%) | 3 | 4 | 0 | 0 | 2 | 181 | 131 | 5 | 31 (15) | 398 | 10 | 32 |
| `tools/nix` | 22 | 651 | 0 | 0 | 13.1 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 (0) | 0 | 0 | 0 |
| `tools/scheme` | 2 | 73 | 0 | 0 | 8.8 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |

### tests/*

| Area | Files | Lines | >500 | >1000 | Comment % (plain/doc) | Redundant est. (lines, % of comments) | Issue refs | History | Restates | Banners | Commented code | Magic nums | Repo-path lits | Hash lits | Fns >80 (no blank) | Longest fn | Naming flags | Codename paths |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `tests/tooling` | 2984 | 475,668 | 6 | 0 | 1.2 / 0.6 | 406 (5%) | 309 | 59 | 1 | 1 | 39 | 0 | 8,125 | 1,269 | 259 (192) | 433 | 427 | 190 |
| `tests/differential` | 120 | 20,034 | 0 | 0 | 1.7 / 0.3 | 31 (8%) | 25 | 4 | 0 | 0 | 2 | 0 | 179 | 351 | 36 (32) | 331 | 24 | 0 |
| `tests/performance` | 75 | 13,127 | 0 | 0 | 0.9 / 1.2 | 4 (2%) | 3 | 1 | 0 | 0 | 0 | 0 | 270 | 119 | 23 (15) | 269 | 1 | 0 |
| `tests/_helpers` | 64 | 11,024 | 2 | 1 | 1.6 / 1.1 | 8 (3%) | 5 | 0 | 1 | 2 | 0 | 0 | 36 | 11 | 19 (11) | 274 | 5 | 0 |
| `tests/davinci_test_support` | 53 | 10,906 | 0 | 0 | 0.6 / 5.5 | 7 (1%) | 6 | 1 | 1 | 0 | 0 | 0 | 33 | 11 | 11 (11) | 224 | 0 | 53 |
| `tests/typescript-plugin` | 20 | 4,526 | 0 | 0 | 0.7 / 0.0 | 1 (3%) | 1 | 0 | 0 | 0 | 0 | 0 | 35 | 3 | 7 (6) | 168 | 0 | 0 |
| `tests/app` | 29 | 4,480 | 0 | 0 | 3.4 / 1.3 | 2 (1%) | 0 | 1 | 0 | 0 | 1 | 0 | 0 | 3 | 3 (0) | 178 | 0 | 0 |
| `tests/editor-conformance` | 22 | 2,751 | 0 | 0 | 5.7 / 1.5 | 1 (1%) | 0 | 1 | 0 | 0 | 0 | 0 | 25 | 0 | 5 (1) | 187 | 1 | 0 |
| `tests/native` | 10 | 1,607 | 0 | 0 | 0.8 / 0.2 | 3 (19%) | 3 | 0 | 0 | 0 | 0 | 0 | 29 | 31 | 0 (0) | 68 | 6 | 0 |
| `tests/fuzz` | 10 | 608 | 0 | 0 | 20.4 / 3.4 | 6 (5%) | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 29 | 0 | 1 |
| `tests/vize_test_runner` | 6 | 524 | 0 | 0 | 1.3 / 2.1 | 1 (6%) | 0 | 0 | 1 | 0 | 0 | 0 | 1 | 0 | 1 (0) | 235 | 0 | 0 |
| `tests/external-consumers` | 7 | 463 | 0 | 0 | 1.2 / 0.5 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 57 | 5 | 0 |
| `tests/scripts` | 1 | 433 | 0 | 0 | 0.9 / 3.7 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 37 | 0 | 0 |
| `tests/consumers` | 1 | 273 | 0 | 0 | 1.5 / 0.0 | 1 (25%) | 0 | 1 | 0 | 0 | 0 | 0 | 3 | 0 | 0 (0) | 50 | 0 | 0 |
| `tests/editor-host` | 1 | 208 | 0 | 0 | 3.7 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |

### editors/*

| Area | Files | Lines | >500 | >1000 | Comment % (plain/doc) | Redundant est. (lines, % of comments) | Issue refs | History | Restates | Banners | Commented code | Magic nums | Repo-path lits | Hash lits | Fns >80 (no blank) | Longest fn | Naming flags | Codename paths |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `editors/vscode` | 87 | 13,763 | 0 | 0 | 1.8 / 1.3 | 15 (4%) | 9 | 6 | 0 | 0 | 0 | 316 | 58 | 29 | 24 (10) | 462 | 26 | 0 |
| `editors/nvim` | 9 | 1,318 | 0 | 0 | 4.9 / 0.0 | 4 (7%) | 3 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |
| `editors/vim` | 7 | 934 | 0 | 0 | 2.6 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |
| `editors/zed` | 16 | 717 | 0 | 0 | 6.3 / 0.0 | 8 (22%) | 0 | 0 | 8 | 6 | 0 | 0 | 0 | 0 | 0 (0) | 23 | 0 | 0 |
| `editors/emacs` | 2 | 252 | 0 | 0 | 6.8 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |

### apps (playground, docs code, examples)

| Area | Files | Lines | >500 | >1000 | Comment % (plain/doc) | Redundant est. (lines, % of comments) | Issue refs | History | Restates | Banners | Commented code | Magic nums | Repo-path lits | Hash lits | Fns >80 (no blank) | Longest fn | Naming flags | Codename paths |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `playground/src` | 180 | 28,261 | 4 | 0 | 1.1 / 1.2 | 33 (6%) | 2 | 2 | 23 | 5 | 1 | 89 | 0 | 0 | 23 (0) | 393 | 63 | 1 |
| `docs/scripts` | 148 | 23,172 | 0 | 0 | 0.4 / 0.4 | 6 (4%) | 2 | 1 | 1 | 0 | 3 | 69 | 270 | 5 | 17 (13) | 257 | 0 | 0 |
| `playground/e2e` | 35 | 4,580 | 0 | 0 | 0.8 / 0.2 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 16 | 17 | 2 | 2 (1) | 105 | 0 | 0 |
| `docs/previews` | 31 | 4,485 | 0 | 0 | 0.1 / 0.8 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 131 | 26 | 0 | 17 (7) | 285 | 0 | 0 |
| `docs/theme` | 21 | 4,358 | 1 | 1 | 2.4 / 0.3 | 5 (5%) | 0 | 0 | 2 | 2 | 1 | 13 | 1 | 0 | 5 (1) | 171 | 0 | 0 |
| `examples/poc-svelte` | 6 | 1,434 | 0 | 0 | 0.1 / 4.2 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 (2) | 98 | 0 | 0 |
| `examples/vite-musea` | 18 | 1,321 | 0 | 0 | 0.9 / 1.4 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 11 | 1 | 0 | 0 (0) | 37 | 0 | 0 |
| `examples/poc-angular` | 5 | 1,116 | 0 | 0 | 0.0 / 4.5 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 68 | 0 | 0 |
| `examples/poc-python` | 7 | 968 | 0 | 0 | 0.0 / 2.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 (1) | 88 | 0 | 0 |
| `examples/poc-support` | 3 | 860 | 0 | 0 | 0.0 / 7.1 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 (1) | 83 | 0 | 0 |
| `examples/poc-solid` | 5 | 692 | 0 | 0 | 0.2 / 4.7 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 63 | 1 | 0 |
| `examples/poc-astro` | 5 | 671 | 0 | 0 | 0.2 / 5.8 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 (0) | 172 | 0 | 0 |
| `examples/poc-moonbit` | 4 | 489 | 0 | 0 | 0.4 / 5.3 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 (1) | 99 | 0 | 0 |
| `examples/poc-coffeescript` | 4 | 426 | 0 | 0 | 0.3 / 6.1 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 (0) | 58 | 0 | 0 |
| `examples/oxlint-vize` | 5 | 295 | 0 | 0 | 1.4 / 13.0 | 1 (2%) | 0 | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 (0) | 37 | 0 | 0 |
| `examples/cli` | 5 | 212 | 0 | 0 | 0.0 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 (0) | 3 | 0 | 0 |
| `playground/scripts` | 1 | 197 | 0 | 0 | 18.5 / 8.2 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 37 | 0 | 0 |
| `docs/vite.config.ts` | 1 | 189 | 0 | 0 | 1.8 / 2.9 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 (0) | 35 | 0 | 0 |
| `examples/jsx-tsx` | 4 | 155 | 0 | 0 | 0.0 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 (0) | 9 | 2 | 0 |
| `docs/acceptance` | 1 | 123 | 0 | 0 | 0.8 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 19 | 0 | 0 | 0 (0) | 0 | 0 | 0 |
| `playground/vite.test.config.ts` | 1 | 59 | 0 | 0 | 3.6 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |
| `examples/html-conformance` | 5 | 51 | 0 | 0 | 0.0 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |
| `playground/playwright.config.ts` | 1 | 45 | 0 | 0 | 4.8 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 (0) | 0 | 0 | 0 |
| `playground/vite.app.config.ts` | 1 | 43 | 0 | 0 | 7.5 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 2 | 1 | 0 | 0 (0) | 0 | 0 | 0 |
| `docs/browser-path.js` | 1 | 32 | 0 | 0 | 3.8 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 18 | 0 | 0 |
| `playground/vite.config.ts` | 1 | 22 | 0 | 0 | 0.0 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |
| `playground/vite.node.config.ts` | 1 | 15 | 0 | 0 | 0.0 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |
| `docs/public` | 1 | 3 | 0 | 0 | 0.0 / 0.0 | 0 (0%) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (0) | 0 | 0 | 0 |


## 4. Hotspots by category

### 4.1 Oversized files

- **172 code files over 500 lines: 110 product and 62 test.** The full list, with custody status, is in `data/over-500.tsv`.
  - Product files by area: `npm/ui` 21, `tools/support` 15, `npm/compose` 13, `crates/vize_atelier_sfc` 9, `npm/builder` 8, `tools/commands` 7, `crates/vize_maestro` 6, `crates/vize_croquis` 6, `crates/vize_atelier_core` 4, `playground/src` 4. `davinci/` has one file over 500.
  - `tools/commands/**` and `tools/support/**` are **exempt from the gate today** (`source-file-lengths.ts:58-59`). They hold 22 files over 500 lines, and the largest is `tools/commands/davinci/davinci_fpfn.rs` at 1,498.
- **Long functions:**
  - 666 lines: `crates/vize_canon/src/virtual_ts/generator.rs`, which is custody-pinned.
  - 637 lines: `crates/vize_atelier_sfc/src/compile.rs`, custody-pinned.
  - 714 lines: one test in `crates/vize_atelier_sfc/src/compile_script/tests.rs`.
  - 533 lines: `tools/support/compat/v1/support-matrix.ts`.
  - 547 lines: `npm/ui/src/families/date-time/scheduler/scheduler-runtime.ts`.
  - 423 lines, no blank line: `crates/vize_croquis_cf/src/diagnostics/formatting.rs` `format_kind_details`.
  - 373 and 310 lines, no blank line: `crates/vize_atelier_sfc/src/script/define_props_destructure/collector.rs` `collect_from_expression` / `collect_from_statement`.
  - About 12 test-support runtimes in `tests/tooling/support/native-input-model-*.ts` have one 400–430-line function each, and they are near-copies of one another.

### 4.2 Comments

- Comment density is low: 1.3% plain and 4.7% doc. The waste is concentrated, not spread.
- **Issue-number lines** by area: crates 1,779 (`vize_canon` 649, `vize_patina` 182), tests 356, tools 208, npm 186 and davinci 120.
  - Worst files: `davinci/vize_l0/src/expression_guard/analyze.rs` (16), `crates/vize_patina/src/markup/differential/battery/census.rs` (13), `crates/vize_atelier_jsx/tests/babel_compat/verdicts.rs` (11) and `crates/vize_atelier_core/src/codegen/tests.rs` (10).
- **Restating comments** are densest in older product crates:
  - `vize_croquis` (146), `vize_canon` (126), `vize_patina` (117), `vize_atelier_core` (107), `vize_maestro` (103) and `vize_fresco` (101).
  - In TS: `npm/compose` (331 JSDoc lines such as `/** Serial number. */`) and `npm/ui` (238).
- **Banners**: 211 lines, 68 of them in `vize_patina`.
- **Doc comments that are product text** are excluded from every sweep: clap derives in 28 files, `#[napi]` in 47 and `#[wasm_bindgen]` in 12.

### 4.3 Naming

| Outlier | Count | Where |
| --- | ---: | --- |
| Issue number in the file name | 442 | `crates/vize_canon` 242, `tests/tooling` 74, `crates/vize` 35, `tools/benchmarks` 29, `crates/vize_patina` 21, `crates/vize_atelier_core` 17 |
| TS/JS file not kebab-case | 222 | `npm/fresco` 73, `playground/src` 61, `npm/builder` 39, `tools/support` 33 |
| Rust file not snake_case | 23 | `tools/support/compat/**/*.rs` (`frozen-10682-observer.rs`), `tools/commands/davinci/*.rs` |
| Generic file name (`helpers`, `utils`, `common`, `shared`, …) | 54 | 24 `helpers.rs`, 7 `utils.ts`, 7 `utils.test.ts`, 5 `shared.rs` |
| File stem over 48 characters | 652 | mostly Pkl step scripts and `tests/tooling` |
| Retired `.mjs` | 657 | `tests/tooling` 225, `tools/**`, `npm/linter/oxlint/scripts` |
| Public Rust functions over 40 / 60 characters | 245 / 34 | `vize_atelier_dom/src/compile/custom_elements.rs` (12 over 50), `vize_atelier_vapor/src/compile/entry.rs` (8), `davinci/vize_l4/.../budget/slot_scope.rs` |
| Rust tests over 60 / 90 characters, or with an issue number | 8,757 / 63 / 77 of 24,410 | `vize_canon` 1,768, `vize_maestro` 1,194, `vize_patina` 1,113 |
| `#[expect(clippy::too_many_arguments)]` | 143 | |
| `#[expect]` reasons copied verbatim (`"fixtures use std strings"`) | 376 files | |
| `tests/tooling` flat directory | 1,832 test files in one directory | prefixes `lsp-` 194, `native-` 158, `davinci-` 144, `typecheck-` 92, `lint-` 69 |

### 4.4 Hardcoding

| Hotspot | Evidence |
| --- | --- |
| Inline recursion and depth limits | 64 inline comparisons such as `>= 128` (`crates/vize_croquis/src/types/world/props.rs:47,143`, `emits.rs:34`, `crates/vize_glyph/src/native_doc/template.rs:123`), against 36 named `*DEPTH*` constants with different values (`MAX_DEPTH = 64` twice, `MAX_RESOLUTION_DEPTH = 256`, `MAX_ELEMENT_NESTING_DEPTH = 4096`) |
| Help text in control flow | `crates/vize_croquis_cf/src/diagnostics/formatting.rs:38-460` |
| Scattered tool versions | `@pkl-community/pkl@0.27.2` in 8 Pkl files, while the catalog pins 0.30.2; 585 version-shaped literals across `tools/config/pkl` |
| Repo paths in tests and tools | 16,361 literals. Worst: `tests/tooling/tooling-test-selection.test.ts` (166), `tests/tooling/native-selected-setup-sfc-dom-capture-workflow.test.ts` (159), `tools/config/vite-plus/tooling-test-scopes.ts` (155), `tools/support/compat/github/run-lsp-config-diagnostics.ts` (144) |
| Re-implemented helpers | 1,548 local `const root`/`repoRoot` definitions, 69 local sha256 helpers and 57 separate `git ls-files` spawns in `tools` and `tests` |
| Hash literals in code | `tests/differential/typechecker-registry.ts` (101), `crates/vize_canon/tests/fix_history_diagnostics.rs` (97), `tools/support/compat/github/native-beta-dto-current-custody.ts` (70) |
| Hand-rolled argv parsing | 564 files read `process.argv`; 82 use `parseArgs` |
| Duplicated exclusion lists | `tools/commands/ci/source-file-lengths.ts:51-95` against the `vite.config.ts` ignore lists |
| Large literal tables mixed into `src/` | `npm/compose/core/src/catalog-utilities.ts` (3,213 lines, 267 entries) and `catalog-entries.ts` (1,599); `npm/ui/.../shape-d3-vectors.ts` (1,285) and `scale-d3-vectors.ts` (584). The vectors are legitimate data, but they belong in a `vectors/` or `data/` file and not as `src` modules. |

### 4.5 Layout and whitespace

**Rust unit tests come in five shapes:**

| Shape | Files |
| --- | ---: |
| Inline `#[cfg(test)] mod tests { … }` | 1,301 |
| `tests.rs` | 705 |
| Sibling `*_tests.rs` | 639 |
| `tests/` directories under `src` | 189 |
| `tests_*.rs` | 47 |

Integration-test files: 2,728. `mod.rs` files: 78.

**Rust imports** (`scripts/rust-import-groups.py`):

| Style | Files |
| --- | ---: |
| Grouped `std` / external / local with blank lines | 2,900 |
| Mixed in one group | 4,089 (3,828 of them are one alphabetical block where `crate`/`super` lands between external crates) |
| Single kind | 3,429 |

**TS imports** (`scripts/ts-import-groups.py`):

| Style | Count |
| --- | ---: |
| Grouped by kind | 2,188 files |
| Mixed in one group | 3,450 files |
| `import type` statements | 3,942 |
| Inline `type` specifiers | 1,464 |

**Vue:** 979 of the 991 SFCs that have both a script and a template put the script first. All 575 SFCs with props use type-based `defineProps`.

**Vertical whitespace:**
- 1,808 Rust items follow a `}` directly. The worst file is `davinci/vize_l4/src/targets/dom/artifact/buf.rs` with 28.
- 1,686 functions over 80 lines have no blank line at all.
- **Formatter configuration:** there is no `rustfmt.toml`. `fmt-rust` runs `cargo fmt --all -- --check` on a floating `toolchain = "stable"` (`tools/config/pkl/parts/check/fmt-rust.pkl:10`). oxfmt sets only `quoteProps: "preserve"`, and oxlint sets only `typeAware: true`.

## 5. Byte custody (pinned sources)

`scripts/pins.py` hashes every first-party file (sha256 and git blob id) and searches the whole tree for those digests. A match means the file's **current bytes** are recorded somewhere. Results are in `data/pins/pinned.tsv` (5,135 files, fixtures included) and `data/pins/pinned-classified.tsv` (884 first-party sources).

| Class | Files | Meaning |
| --- | ---: | --- |
| `direct-in-test` | 55 | A test or custody/transition source spells the digest. An edit fails a test. |
| `record-read-by-test` | 425 | A JSON/TSV record names the path and digest, and a test loads that record. An edit very likely fails a test, often through a "transition" function that reconstructs the predecessor bytes from the current file (`tests/tooling/script-consumed-slots-custody.test.ts`). |
| `record-only` | 404 | Only a ledger or provenance record carries the digest. An edit leaves a stale record but probably fails nothing. |

Enforced pins by area (direct plus record-read-by-test, with record-only after the slash): `tests/tooling` 80/38, `crates/vize_canon` 68/17, `crates/vize_maestro` 43/6, `crates/vize_patina` 42/6, `tools/support` 42/47, `crates/vize` 25/8, `crates/vize_glyph` 24/52, `npm/cli` 18/28, `crates/vize_atelier_vapor` 15/0, `crates/vize_atelier_sfc` 14/5, `crates/vize_croquis_cf` 14/0, `crates/vize_croquis` 11/0, `tests/typescript-plugin` 11/0, `crates/vize_atelier_jsx` 9/0, `npm/volar` 9/2, `crates/vize_vitrine` 7/4, `davinci/vize_l0` 7/7. `npm/builder` has 2 enforced and 98 record-only.

**In Rust:** 288 enforced files under `crates/` and `davinci/`. 106 are test files and **182 are product sources (36,610 lines)**. That includes `crates/vize_canon/src/virtual_ts/generator.rs`, which the fleet brief already names, and `crates/vize_atelier_sfc/src/compile.rs`. Only 4 of the 172 over-limit files are enforced pins, so splitting big files is mostly unblocked.

**Two other line-number pins:**
- Generated rule pages embed `#L<line>` anchors into `crates/vize_patina/src/rules/**`: 1,864 anchors in `docs/content/**`, across 5 locales.
- `vize_l0` micro-profiler sites embed `file!():line!()`. They exist only under the `micro-profiling` feature, which is off in release.

The classification is static. The authoritative check is still the full `tests/tooling` suite plus the Rust suites on a branch.

## 6. Open-PR overlap (2026-11-24 evening)

`data/open-pr-areas.tsv` maps the 26 open PRs to areas.

- **Busiest areas:**
  - `crates/vize_atelier_sfc`: 8 PRs (#13539, #13571, #13601, #13617, #13620, #13623, #13628, #13632)
  - `davinci/vize_l4`: 6
  - `crates/vize_patina`: 6
  - `davinci/vize_l0`: 5
  - `tests/tooling`: 10
  - `tools/config`: 11
  - `docs`: 15
- **Areas with no open PR right now:**
  - Rust: `vize_glyph`, `vize_fresco`, `vize_musea`, `vize_doctor`, `vize_marquette`, `vize_curator`, `vize_relief`, `vize_armature`, `vize_croquis_cf`, `vize_atelier_jsx`
  - npm: `npm/ui`, `npm/compose`, `npm/fresco`, `npm/devtools`, `npm/mcp-musea`, `npm/plugin-sdk`, `npm/wasm`
  - also `editors/*`, `playground` and `examples`
- **Not open as PRs, but in flight:**
  - The codename hard rename (`refactor/level-codename-rename`, which renames `davinci/**` paths).
  - The `L1Ir` entry-point renames.
  - The workflow program (#13633 and its successors).
  - The tools move.

## 7. Worst 30 files

Ranked by `scripts/worst.py`, a composite of lines over 500, longest function, functions with no blank line, flagged comments, hardcoded paths and hashes, magic numbers and naming flags. The why column is a hand review.

| # | File | Lines | Why |
| ---: | --- | ---: | --- |
| 1 | `crates/vize/tests/check_cli.rs` | 3,750 | One test binary for the whole `vize check` surface. It has 15 functions over 80 lines (up to 228), `use`/`mod` interleaved with a brace-root `use {…}` (lines 5-21), and copied expect reasons. Split it by behaviour into `tests/check_cli/*.rs`. Test-only. |
| 2 | `crates/vize_atelier_sfc/src/compile/tests.rs` | 3,202 | Every SFC compile behaviour in one file. Split it by feature (props, emits, styles, model, …). |
| 3 | `npm/compose/core/src/catalog-utilities.ts` | 3,213 | 267 metadata literals in a source module. It belongs in a data file (JSON or TS data) with a schema, or one file per category. |
| 4 | `crates/vize_canon/src/batch/type_checker/tests.rs` | 2,214 | A monolithic test file with a 148-line test and 6 issue-number comment lines. |
| 5 | `crates/vize_canon/src/virtual_ts/tests.rs` | 2,046 | 2,046 lines in one test module; 98 plain comment lines, 7 of them issue references. |
| 6 | `tests/_helpers/apps.ts` | 1,997 | An app-E2E helper that mixes per-app setup, such as Misskey and Nuxt UI, with generic runners. It has a 204-line function. Split it one file per app. |
| 7 | `tools/commands/davinci/source_harness.rs` | 1,197 | Exempt from the length gate. It has a 398-line function, 24 items with no blank line, a codename in the path and 11 hardcoded repo paths. |
| 8 | `npm/compose/core/src/catalog-entries.ts` | 1,599 | A data table, with 320 numeric literals, inside `src/`. |
| 9 | `crates/vize_armature/src/parser/tests.rs` | 1,966 | A monolithic parser test with "Regression (#1065/#1090)" style doc headers. |
| 10 | `crates/vize_croquis/src/script_parser/tests.rs` | 1,806 | Monolithic, with a 120-line test. |
| 11 | `npm/ui/src/families/charts/chart-shape/shape-d3-vectors.ts` | 1,285 | Legitimate d3 reference vectors, but in `src/` as code. Move them to `vectors/*.json`, or name the module `*.vectors.ts` outside the package entry. |
| 12 | `tools/support/compat/v1/support-matrix.ts` | 944 | A 533-line builder function, 73 repo paths and 7 hashes inline. It is the source of `docs/release/v1-supported-matrix.json`. |
| 13 | `crates/vize_atelier_sfc/src/compile_script/tests.rs` | 826 | One test function of 714 lines. |
| 14 | `crates/vize_atelier_core/src/codegen/tests.rs` | 1,510 | Monolithic; 10 issue-reference lines among 68 comment lines. |
| 15 | `crates/vize_canon/tests/fix_history_diagnostics.rs` | 1,291 | 97 inline hash literals. Move them into fixture manifests. |
| 16 | `crates/vize_canon/src/virtual_ts/generator.rs` | 861 | A 666-line function, and the file is **byte-custody pinned**. It needs the custody procedure before anything can be split out. |
| 17 | `npm/compose/core/src/treeshake.test.ts` | 1,615 | A monolithic test. |
| 18 | `npm/builder/vite/src/plugin/resolve.test.ts` | 1,570 | A monolithic test, beside a 1,274-line `resolve.ts` (#28). |
| 19 | `crates/vize_canon/src/sfc_typecheck/tests.rs` | 1,432 | Monolithic. |
| 20 | `tools/commands/davinci/davinci_fpfn.rs` | 1,498 | The largest tools file. Exempt from the gate, with a codename in the path. |
| 21 | `npm/compose/core/src/use-form.ts` | 980 | A 416-line composable body. |
| 22 | `crates/vize_canon/src/batch/virtual_project/tests.rs` | 1,272 | Monolithic; 9 issue-reference lines. |
| 23 | `crates/vize_atelier_sfc/src/script/define_props_destructure/collector.rs` | 931 | Two 300–370-line functions with no blank line. The largest product Rust file that is not pinned. |
| 24 | `crates/vize_atelier_sfc/src/compile.rs` | 699 | A 637-line function. **Custody-enforced** (`tests/native/sfc-empty-project-context-source.ts`), and touched by 8 open PRs. |
| 25 | `npm/fresco/src/app.ts` | 901 | A 428-line function, in the package with the most camelCase file names. |
| 26 | `tools/commands/davinci/assertion-lint.rs` | 1,092 | A rust-script with a 171-line function and a non-snake file name. |
| 27 | `npm/ui/src/families/date-time/scheduler/scheduler-runtime.ts` | 668 | A 547-line function and 25 magic numbers (time math). |
| 28 | `npm/builder/vite/src/plugin/resolve.ts` | 1,274 | Product code, the main Vite resolver. 60 plain comment lines, 3 of them issue references. Split it by resolution kind. |
| 29 | `npm/ui/src/families/charts/chart-scale/scale-d3-vectors.ts` | 584 | Same as #11. |
| 30 | `crates/vize_atelier_core/src/codegen/v_for/generate.rs` | 833 | A 365-line product function on the hot codegen path. Any split needs IR identity. |

Ranks 31–40 are in `data/worst.md`.

## 8. Heuristics and their precision

All the regexes are in `scripts/inventory.py`. I checked precision by hand on random samples with `scripts/sample-flags.py <root> <flag> <prefix> <n> <seed>`:

| Flag | Sample | Precision | Notes |
| --- | --- | ---: | --- |
| `issue` | 6 in `npm` | 100% | Every hit really is an issue reference. Whether to delete it is policy (style guide §1.3). |
| `restates` | 12 in `crates` | about 75% | False positives are section labels in long `match` tables ("Clipboard events"). |
| `history` | 10 in `crates` | about 50% | "used to decide", "previously captured" and "originally" are false positives. Review by hand. |
| `commented_code` | 6 in `tools` | about 30% | The TS tokenizer does not model regex literals. Never auto-delete these. |
| `banner` | — | about 100% | |

A rerun after each lane gives the trend. The ratchet tool (plan §5) will use the precise subset: issue numbers, banners and a history-phrase list tuned to about 90% precision.

## 9. Reproduce

```bash
S=<scratch>; git -C <repo> archive origin/main | tar -x -C $S/src
python3 -I scripts/inventory.py $S/src data/inv              # files.tsv + summary.json (~4 min)
python3 -I scripts/aggregate.py data/inv/files.tsv data/generated-files.txt > data/tables.md
python3 -I scripts/worst.py data/inv/files.tsv data/generated-files.txt data/pins/pinned-classified.tsv 40
python3 -I scripts/pins.py $S/src data/pins                   # byte-custody scan (~3 min, needs rg)
python3 -I scripts/rust-import-groups.py $S/src/crates $S/src/davinci
python3 -I scripts/ts-import-groups.py $S/src/npm $S/src/tools $S/src/tests
```
