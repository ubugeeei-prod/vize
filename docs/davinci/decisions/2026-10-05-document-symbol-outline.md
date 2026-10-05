# SFC document symbols (2026-10-05)

Decision paired with [#8006](https://github.com/ubugeeei-prod/vize/issues/8006).
This fixes the existing editor provider, independently of the native LSP
migration and [#6883](https://github.com/ubugeeei-prod/vize/issues/6883).

## User-visible contract

Retain the existing block names, discovery order, kinds, language details,
whole block ranges and tag-name selection ranges. Add children under script,
script setup and HTML/art template blocks. Existing style blocks stay intact.

Script declarations follow the original OXC AST order. Variables and patterns
select the declared identifier; functions, classes, interfaces and type aliases
select their names. Object-valued bindings carry statically named object
members, including methods and nested objects. Classes carry statically named
methods and fields. Dynamic computed names and spreads are not invented;
imports and anonymous default exports are outside this bounded outline.
Function parameters and local statements are not promoted to top-level symbols.

Template children follow the original untransformed element AST. Elements
carry their descendants, full source ranges and tag-name selection ranges;
components use Class and ordinary elements use Object. Directives stay on their
original elements; interpolation expressions, text and comments are not symbols.
Only an exact original `<tag` name span is selectable. Parser-inserted implicit
tbody/tr nodes are omitted while their authored descendants remain visible.
All positions use one original-document UTF-16 line index, including CRLF and
astral characters. Rejected child parses keep the existing block without
inventing a partial tree. Unknown template languages keep the block.

The resident SFC descriptor already supplies original block content and offsets.
The request uses existing OXC/template parser entry points once per requested
block, without a new pipeline stage, generated module, native IPC or per-node
type query. This is structural editor data, not a type-accuracy or latency claim.

## Original input and independently authored oracle

Retain the issue's complete `MySwitch.vue` (205 bytes, SHA256
`6d7fa96a27d5338e8c9f9c9f3368c2d5be78576756b70bd53e414f96d662fa66`)
and `Parent.vue` (429 bytes, SHA256
`116cac3641306e4bb0c53fc7ed5fd662b0a3a203868a6c648be24579f0f3c40c`).
The complete public issue body and original `vize.config.json` are retained;
`tsconfig.json` is authored from the issue's complete stated options/include.

The full expected symbol arrays are newly authored from those source positions
and the feature contract. They are not original raw server captures. Parent
expects template children MySwitch/p and script children store/get/set/on/toggle;
MySwitch expects span and the destructured items/checked bindings. The arrays
preserve every name, kind, range, selectionRange, child order and omitted field.
Rust whole-array controls and shared corpus real stdio sessions use the same
immutable expected bytes. Synthetic framed data only exercises comparator laws.

The new corpus requests hierarchical document symbols explicitly while leaving
all old case capability objects untouched. Both original Vue files and configs
are materialized; each corpus session opens its entry only, with the existing
editor-on/lint-off/typecheck-off observation profile. It does not replay the
reporter's complete typechecking/workspace-symbol session. Fresh source-built
RPC framing, executable/build identity, readiness, responses and shutdown must
be retained before runtime acceptance; no local build or RPC capture is credited.

## Validation and remaining work

Prepared Rust controls cover complete reported projects, destructuring/defaults,
exports, nested members/classes, TS/JSX, directives, UTF-16/CRLF, malformed and
empty content, art, and honest Pug fallback. Eleven local pure controls pass;
the original five highlight sessions are selected by their fixed IDs so new
outline cases cannot extend their original nine-request vector. Source and protected Actions are
pending; publication, queue entry and completion require actual terminal proof.

Pug has no original-span element tree in the current resident descriptor. Keep
its block-only template outline rather than substitute generated HTML offsets;
its script outline works. Original-span Pug elements, style selectors, richer
anonymous/default export structure and unresolved computed members remain TODO.
No native/default or whole-fix-history acceptance, speed, memory, ranking or
10x claim follows from this change. Keep #6883 open and instruction budgets
unchanged. The verified original reporter is included as a source Co-author;
final same-primary squash normalization is reported from the actual commit.

The source-qualified consumption and migration inventories record these legacy
SFC/parser and raw OXC imports. Their authoritative generators refresh only the
Maestro shards; the existing rows and fixture gates remain intact. These rows
record current source use and grant no native migration acceptance.
