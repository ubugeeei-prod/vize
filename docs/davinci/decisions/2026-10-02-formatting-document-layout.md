# Formatting document layout (2026-10-02)

Issue: [#6875](https://github.com/ubugeeei-prod/vize/issues/6875).

The first reviewable layer adds a neutral Doc IR and a separate printer in
`vize_glyph::native_doc`. It uses only the existing L0 dependency and has no
parser, L1, semantic-IR or product-route prerequisite. Actual syntax consumers
will build Doc in their own layer. It does not switch an existing formatter API.

Doc borrows source text and composes arena-resident concatenation, group,
indentation, breakable space/empty lines and hard lines. Cached flat widths avoid
re-reading flat subtrees. The printer uses iterative reusable stacks, including
lookahead through the first pending continuation line, so a group reserves
width for closing syntax outside its own boundary. Width counts Unicode scalar
values; it does not claim terminal/font display width. Generated newlines use
LF or CRLF while source text keeps its exact authored bytes. Unbreakable source
may exceed the requested width and is never truncated.

Seven actual output laws cover exact Unicode/group width, scoped indentation,
nested-group choices, continuation width, hard versus authored newlines,
unbreakable source at zero width and 20,000 iterative nested groups. Production
and test strict Clippy and Rust formatting are checked on the actual frozen
source using retained dependencies. These are bounded ordinary-module laws,
not a whole Cargo dependency build, hosted Actions, differential acceptance,
performance budget, history closure or merge proof.

A prior combined-source whole Glyph typecheck against old retained dependencies
was held by absent current ImportSortGroup/SortImportsSetting APIs in old L0.
Its actual failure remains in the preserved original handoff; no replacement
provider or successful whole-product claim follows from the layout laws.

Remaining: genuine retained L1 template/directive/embed consumers (#6847),
all Vue dialects and JS/TS/JSX/TSX, SFC/options, shared source/version edits
(#6876), full-source Actions and formatter-specific instruction-count evidence.
The complete #6882 fix-history gate still precedes any default product switch.

## Canonical inventory (2026-10-03)

The first hosted required and full-source runs rejected the stale Glyph consumer
migration shard: module registration moves the existing lib.rs L0 site and the
Doc/printer/layout-law files add three real L0 sites. The unchanged canonical
Node generator writes the actual 62-line shard; every other generated shard
remains byte-exact. The correction changes no product source, expectation, gate,
fixture or performance ceiling. Actual canonical inventory/storage/ledger laws
are checked locally; fresh exact-head Actions and protected queue remain required.
