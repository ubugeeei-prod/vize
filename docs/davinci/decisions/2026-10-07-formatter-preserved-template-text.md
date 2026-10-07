# Preserve template text whitespace (#7871)

The formatter config snapshot discards `compiler.whitespace` after evaluating
project configuration. The ordinary template layout condenses text boundaries,
so formatting the reported button changes its rendered text when the compiler
uses `preserve`.

Retain the explicit compiler mode from the same snapshot evaluation and forward
it to an additive `GlyphFormatter` builder. With `preserve`, copy authored bytes
outside markup syntax, including text boundaries, newlines, interpolation source,
comments and raw regions. Reuse the existing tag parser, attribute normalization,
sorting and multiline writer; positive controls require shorthand normalization
and configured multiline attribute formatting. No extra parse or pipeline stage
is added. The default condensed route and non-HTML template route remain as
before. Script/style/document formatting continues. Interpolation syntax remains
source owned in this mode; further interpolation formatting is separate work.

The corpus retains the original complete App.vue, JSON config and complete
reference. Rust public API laws exercise three passes, inline/nested boundaries,
Unicode/entities, comments, raw/pre/v-pre, LF/CRLF and Auto endings. CLI laws
exercise discovered and explicit JSON/TypeScript configurations through check,
three writes and check, with no-config and explicit-condense controls. The existing
source-bound CLI observer validates its exact binary receipt, retains complete
raw process streams, compares complete output bytes, and uses the pinned stock
Vue DOM/SSR runtime oracle over the original, shorthand and inline controls.

Local stock Vue execution independently confirms the three authored runtime
contracts, including escaped Unicode attributes and exact horizontal whitespace.
This is reference evidence only. Fresh exact-source Actions must execute the
actual formatter and CLI laws, all existing fixtures/history and unchanged
protected instruction caps before actual merge. Installed release replay and
Issue closure remain pending. No native migration or measured speed claim is made.

The first exact-source Check 37590175249 passes the affected Rust build and all
four Rust test shards. Tooling rejects a visibility-only change to the immutable
whitespace-significant source witness and two new consumer inventory entries.
Restore the original witness bytes instead of changing historical pins; reuse its
existing raw-region writer and remove only that writer's complete-close layout
newline. Unclosed and incomplete raw tails retain every authored byte, including
an unmatched outer element with a complete nested close. Regenerate the consumer
inventory from source, retain all original300 histories and qualification rules,
and require fresh successor Actions; the earlier green Rust evidence does not
qualify the changed successor.
