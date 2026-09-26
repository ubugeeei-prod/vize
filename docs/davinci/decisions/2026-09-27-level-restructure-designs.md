# Level Restructure — Detailed Level Designs (2026-09-27)

This page is part of the canonical
[decision record](./2026-09-27-level-restructure.md). It retains the decisions
from the same maintainer design session.

## L3 is the decision layer

Tracked in [#6839](https://github.com/ubugeeei-prod/vize/issues/6839). The
detailed design is in the
[#6839 design comment](https://github.com/ubugeeei-prod/vize/issues/6839#issuecomment-5847890775).

- **L3 decides, L4 encodes.**
  - L3 owns the decisions: the static level of each L2 node, the set of
    dynamic bindings per element, `Hoist`/`Cache` placement, and control
    regions (if, for, slot).
  - L4 owns the encoding: DOM patch flags, `dynamicProps`, `_cache[n]`,
    stringification and the block tree; Vapor templates and effects; SSR
    strings.
- **Target-specific criteria are L3 policies**, one per target:
  `l3::policy::{dom, vapor, ssr}`.
- **L3 artifacts are split in two:**
  - decision tables keyed by L2 `NodeId`, always built;
  - the flat program, built only on demand (for Vapor).
- DOM and SSR read the L2 tree plus the L3 decision tables. Vapor reads the
  L3 program. This replaces charter row #9's "SSR thin path".
- **The work moves; it does not grow.** The L3 decision computation replaces
  the DOM `pass/hoist` analysis. Its instruction count must not exceed the
  current DOM lane
  ([#6868](https://github.com/ubugeeei-prod/vize/issues/6868)).
- **Migration:** tests compare the old DOM analysis with the L3 decisions.
  Once they agree, the old analysis is deleted. The comparison never runs in
  production.
- Remarks are recorded only while an observer is active.
- **Risk:** `_cache[n]` numbering must follow Vue's order. That is an L4
  encoding concern, not an L3 decision.

## L4: emission

Tracked in [#6840](https://github.com/ubugeeei-prod/vize/issues/6840). The
detailed design is in the
[#6840 design comment](https://github.com/ubugeeei-prod/vize/issues/6840#issuecomment-5847908963).

- One crate, `vize_l4`, laid out as:
  - `write/`: the writer, built on `EmitDocument` and source maps. Legacy
    codegen depends on it.
  - `expr/`: expression rewriting
  - `runtime/`: the runtime helper vocabulary
  - `module/`: module assembly
  - `target/{dom, ssr, vapor}`, later `ts` (the type-check projection) and
    other frameworks
- **L4 writes text directly.** There is no JS AST plus codegen step.
- **One append-only writer serves every target:**
  - span links cost nothing when not recording;
  - it handles indentation and tracks the set of used helpers;
  - the preamble is assembled last, so nothing is `insert_str`-ed into the
    middle of a string.

  This gives DOM structural source maps and removes DOM's source-map
  fallback to legacy.

- SSR runs natively, without legacy codegen or Croquis.
- **Vapor generates JavaScript directly from the L3 program, with no legacy
  IR.** The port keeps the current emission order. The existing L3→legacy IR
  adapter stays as the test oracle until parity, then is deleted.
  Architectural soundness takes priority over reuse, and output stays
  byte-identical.
- **Module assembly lives in L4 `module/`.** `vize_atelier_sfc` becomes
  lane selection only, and the other `vize_atelier_*` crates become thin
  shells that select a lane and fall back to legacy.

## Script side

Tracked in [#6844](https://github.com/ubugeeei-prod/vize/issues/6844). The
detailed design is in the
[#6844 design comment](https://github.com/ubugeeei-prod/vize/issues/6844#issuecomment-5847967507).

- **L1** parses each script block with the language provider (oxc AST plus
  spans).
- **Macros** (`defineProps`, `defineEmits`, …) go through a pattern table
  keyed by callee name, the same way directives do.
- **L2 core** holds scopes, symbols, imports/exports and references.
- **L2 `framework::vue`** holds binding kinds, the
  props/emits/model/slots contract and reactivity facts. It feeds the
  identifier-resolution table that L4 uses.
- **Scope and symbol analysis is one lightweight, Vue-focused walk** that
  every product shares. It does not use `oxc_semantic`. When a product needs
  more, extend that walk instead of adding a second analysis. Instruction
  counts gate it
  ([#6868](https://github.com/ubugeeei-prod/vize/issues/6868)).
- **Setup output is span-level rewriting** in L4 `module/`, following the
  MagicString model.
- **Cross-file type resolution is lazy.** Only types referenced by macros
  are resolved. They are resolved through the project model
  ([#6874](https://github.com/ubugeeei-prod/vize/issues/6874)) and cached in
  the resident tier.

## Dialects, languages, frameworks

Tracked in [#6841](https://github.com/ubugeeei-prod/vize/issues/6841),
[#6842](https://github.com/ubugeeei-prod/vize/issues/6842) and
[#6843](https://github.com/ubugeeei-prod/vize/issues/6843). The detailed
layout is in the
[#6841 design comment](https://github.com/ubugeeei-prod/vize/issues/6841#issuecomment-5847986623).

- **Per-level layout** (each level includes only the parts it needs):

  ```
  core
  container/vue
  markup/{html_core, vue}
  profile/{document, component}
  lang/{js, ts}
  framework/vue/{feature/*, version.rs, quirks.rs}
  registry.rs
  ```

- **Versions are const compositions of features** (`framework/vue/feature/*`
  combined in `version.rs`). They replace `LegacyCaps` and
  `LegacyDialectCapabilities`.
- **One registry per level** (`registry.rs`) is the only place that names
  variants.
- **Core never references axis modules.** An import-path test in the PR tier
  (T0) enforces this.
- **Quirks is a feature set orthogonal to versions** (`quirks.rs`), and
  `TemplateSyntaxMode` moves there. Vue 0.x and 1.x are implemented on
  Davinci.
- petite-vue gets L1/L2 support (document profile plus hooks), and lint and
  LSP read it. There are no L3/L4 targets for petite-vue.
- `vize_dialect_moonbit` dissolves into per-level `lang/moonbit` modules.

## Type check

Tracked in [#6849](https://github.com/ubugeeei-prod/vize/issues/6849) and [#6879](https://github.com/ubugeeei-prod/vize/issues/6879). The detailed design is in the
[#6849 design comment](https://github.com/ubugeeei-prod/vize/issues/6849#issuecomment-5848020063).

- **Parity is measured on diagnostics** (position, message, code), not on
  the bytes of the virtual TS.
- **The virtual TS shape may be redesigned, but every requirement from the
  fix history must still hold.** 271 of the 354 commits that touch
  `virtual_ts` are fixes. Before the generator is replaced, that history is
  turned into diagnostic fixtures ([#6879](https://github.com/ubugeeei-prod/vize/issues/6879), Stage 0).
- **`vize_l4::target::ts` writes the virtual TS through the single writer.**
  Its span links are the `ProjectionMapping` rows.
  - Canon's own scope analysis (9.3k lines) is replaced by the L2 scopes and
    the shared script walk.
  - Maestro's `virtual_code` is merged into it.
- **LSP sync to tsgo keeps the layout stable.** Only the ranges of edited
  blocks and embeds are sent.
- **The checker host** (tsgo / corsa sessions) lives in `vize_l4` behind a
  feature (for example `host`) that pulls in std and I/O. Level crates
  therefore stay `no_std + alloc` for portable builds.
