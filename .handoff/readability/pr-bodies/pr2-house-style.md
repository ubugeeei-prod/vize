## What

Lane L1 of the source-readability program: the house source style, written down as repository policy, plus the configuration that is already true today. **Held for the maintainer's review. Auto-merge is off.**

| File | Change |
| --- | --- |
| `docs/project/source-style.md` (new, English only, next to `redundancy.md`) | The house style for Rust, TS/JS, Vue, Pkl, shell, Nix, editor scripts and code in docs. It covers file size, naming, comments, hardcoding, vertical whitespace, tests and the enforcement matrix. |
| `docs/project/README.md` | Links the page. |
| `rustfmt.toml` (new, root) | `max_width = 100`, `newline_style = "Unix"`, `use_field_init_shorthand = true`, `use_try_shorthand = true` |
| `vite.config.ts` | oxlint `eslint/no-warning-comments` with `terms: ["fixme", "xxx", "hack"]` |

The 2026-11-24 decisions are written into the evidence classes (§ header) and the rules:
- **N0** (comments, blank lines, import order) passes on token identity, IR identity with panic-location line and column normalized, and an identical release size. No instruction tables are needed.
- **N1** splits may grow the release binary only by the file-path strings of moved panic sites (about 30–60 B per new file). Any other growth is rejected.
- **Issue numbers** leave comments, file names and test names. They stay only in fixture manifests, ledgers and `TODO(#n)`.
- **Byte custody**: pinned bytes are frozen as fixture copies under `tests/_fixtures`, and the hash checks point at the copies.

## Evidence (class T/N0: nothing changes today)

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` with the new `rustfmt.toml` (pinned rustfmt 1.11.0, toolchain 1.100.0) | clean, 0 diffs |
| `style_edition = "2024"` (from the draft) | **left out**: it reformats `upstream/forks/parcel_sourcemap` (edition 2021, 2 diffs). Each crate's edition already picks the style edition. |
| `vp lint` on the whole repo with `no-warning-comments` | 0 hits, exit 0; a probe file with `// FIXME` is reported |
| `vize` binary | no Rust source, manifest or lockfile change |

### Option names checked against the pinned versions

- **oxfmt 0.63** (`configuration_schema.json`): import sorting is `sortImports` (`newlinesBetween`, `order`, `ignoreCase`, `groups`, `customGroups`, `internalPattern`, `partitionByComment`, `partitionByNewline`, `sortSideEffects`), not `experimentalSortImports`. `sortPackageJson` defaults to `true` already. Nothing is turned on here: `sortImports` rewrites every file it reaches, so it lands per area with that area's sweep.
- **oxlint 1.78**: every proposed rule name resolves, while a made-up name fails config parsing. Hits in `npm/*/src`, `tools`, `tests`, `playground/src` and `docs/{theme,scripts}` with fixtures ignored:

| Rule | Hits |
| --- | ---: |
| `eslint/no-warning-comments` (fixme/xxx/hack) | **0**: enabled |
| `import/no-duplicates` | 27 |
| `unicorn/filename-case` (kebab) | 205 |
| `typescript/consistent-type-imports` | 315 |
| `eslint/max-depth` 4 | 338 |
| `eslint/max-nested-callbacks` 3 | 346 |
| `eslint/max-params` 4 | 531 |
| `eslint/max-lines-per-function` 120 | 730 |
| `eslint/no-nested-ternary` | 910 |

## For the maintainer's review

1. **Doc home.** `docs/project/source-style.md` is English-only, so no ja/zh-CN/pt-BR/fr copies are needed. The alternative is sections in `.github/CONTRIBUTING.md` and `architecture/source-guide.md`, which have translations.
2. **Nightly rustfmt for import groups** (`group_imports = "StdExternalCrate"`, `imports_granularity = "Module"`, `fmt-rust` only, date-pinned). Until it is approved, the doc says the readability ratchet's `mixed-imports` counter tracks import grouping.
3. **Advisory counters.** The ratchet from #13638 enforces file length and issue numbers. Long functions, restating comments, import groups and hardcoded limits are advisory, because a replay showed they would have blocked 3 of 5 ordinary fixes. Once this style is adopted they can become enforced (one line in `enforcedCounters`); that is a separate PR, after the fleet brief tells agents.
4. Open items not decided here:
   - renames that change `Debug` strings;
   - deep-import renames in packages;
   - rule-page `#L` anchors;
   - `tests/tooling` subdirectories;
   - the test-name budget;
   - moving `fmt-rust` to the pinned toolchain (a Pkl change, so it belongs to the workflow lane).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
