# Recent published releases

Selected changes from actual public releases. The [complete release history](https://github.com/ubugeeei-prod/vize/releases) contains every public release and comparison.

## [0.439.0] - 2026-10-10

### Added

- Add documented HTML element hover and standard CSS completion and hover in the language server ([#8316](https://github.com/ubugeeei-prod/vize/pull/8316), [#8318](https://github.com/ubugeeei-prod/vize/pull/8318)).
- Add project-wide Musea preview toolbar controls ([#8331](https://github.com/ubugeeei-prod/vize/pull/8331)).

### Fixed

- Preserve binding identity during rename, search project references and unopened symbols, and report incomplete diagnostics after a native timeout ([#8166](https://github.com/ubugeeei-prod/vize/pull/8166), [#8121](https://github.com/ubugeeei-prod/vize/pull/8121), [#8124](https://github.com/ubugeeei-prod/vize/pull/8124)).
- Keep formatter comments attached to their blocks and honor print width for inline interpolations and continued closing lines ([#8108](https://github.com/ubugeeei-prod/vize/pull/8108), [#8267](https://github.com/ubugeeei-prod/vize/pull/8267), [#8307](https://github.com/ubugeeei-prod/vize/pull/8307)).
- Preserve Vapor static attribute modifiers and merge order, resolve filename recursion and setup directives, and retain destructured slot defaults ([#8172](https://github.com/ubugeeei-prod/vize/pull/8172), [#8183](https://github.com/ubugeeei-prod/vize/pull/8183), [#8170](https://github.com/ubugeeei-prod/vize/pull/8170)).
- Detect SSR browser-global usage in script setup, reject unknown configured lint rule IDs, and enable `no-with-defaults` in the opinionated preset ([#8094](https://github.com/ubugeeei-prod/vize/pull/8094), [#8110](https://github.com/ubugeeei-prod/vize/pull/8110), [#8340](https://github.com/ubugeeei-prod/vize/pull/8340)).
- Update `shell-quote` and the MCP SDK for their security advisories ([#8137](https://github.com/ubugeeei-prod/vize/pull/8137), [#8148](https://github.com/ubugeeei-prod/vize/pull/8148)).

[Release notes](https://github.com/ubugeeei-prod/vize/releases/tag/v0.439.0) · [Complete changes since 0.435.0](https://github.com/ubugeeei-prod/vize/compare/v0.435.0...v0.439.0)

## [0.435.0] - 2026-10-06

### Added

- Enable Vue 3 correctness rules in the default presets and add opt-in strict component attribute checks ([#8093](https://github.com/ubugeeei-prod/vize/pull/8093), [#8075](https://github.com/ubugeeei-prod/vize/pull/8075)).

### Fixed

- Select the nearest JavaScript project configuration, support jsconfig-only projects, preserve computed inlay types, and recover hover after canceled native work ([#8096](https://github.com/ubugeeei-prod/vize/pull/8096), [#8103](https://github.com/ubugeeei-prod/vize/pull/8103), [#8037](https://github.com/ubugeeei-prod/vize/pull/8037), [#8095](https://github.com/ubugeeei-prod/vize/pull/8095)).
- Format declared TypeScript generic arrows and correct Musea code panel width and theme ([#8077](https://github.com/ubugeeei-prod/vize/pull/8077), [#8084](https://github.com/ubugeeei-prod/vize/pull/8084)).
- Remove production npm security advisories ([#8080](https://github.com/ubugeeei-prod/vize/pull/8080)).

[Release notes](https://github.com/ubugeeei-prod/vize/releases/tag/v0.435.0) · [Complete changes since 0.434.0](https://github.com/ubugeeei-prod/vize/compare/v0.434.0...v0.435.0)
