# Native transport refusal custody (#6879)

The protected queue for #8632 retained one native failure in the full Rust
suite: `unavailable_native_semantic_transport_cannot_report_empty_clean`.
The native CLI still returned exit 1, empty stderr, and the exact original
Hoppscotch declaration TS1128 at line 1294. Semantic completion also refused;
the test had expected its fallback to repeat the CLI syntax refusal, while
the actual API session returned a closed JSON-RPC connection refusal.

The complete observed execution message is preserved as a second exact
expectation alongside the original complete CLI fallback message. The law
requires an actual nonempty denied `--api`/`--lsp` trace, rejects every other
record or incomplete line, and binds the API connection refusal to a recorded
`--api` invocation. Each short shell append records the invocation before
the wrapper exits 41; all CLI calls still delegate to the genuine native
executable. No production fallback policy or diagnostic oracle changes.

The retained packet includes the entire wrapper and transport trace, genuine
native path, generated configuration, CLI exit/stdout/stderr and whole error.
The ordinary semantic control keeps its entire TS2322 vector and requires
zero denied transport invocations. The original broken/repaired syntax and
semantic pair and pinned Element Plus control remain required.

PR native phases run the complete `native_syntax_semantics` integration suite
with required native execution before the existing source CLI/editor controls,
using the same `ci-opt` build profile and dependencies. Its complete packets
occupy the dedicated `native-source-recipe-transport-refusal` directory, whose
existing `native-source-recipe*` artifact glob retains every recursive member.
No gate, fixture, budget,
timeout or original source recipe is removed or relaxed.

Hosted exact-head source/native qualification and actual protected merge are
required. The first failed queue remains failure evidence. This test-only
repair grants no compiler/typechecker behavior, performance or public-release
acceptance credit.

The first hosted source `c206e4b` passed all three actual native runtime laws;
the full suite refused the uninitialized Element Plus gitlink rather than
mistaking the outer checkout for its pinned upstream repository. The existing
immutable gitlink must be initialized before this complete integration suite;
the exact `20298a21` pin/source/config checks remain intact. Its source-plan
gate also retained the shifted Carton-use line in the generated inventory;
regenerating only that row preserves the same dependency classification.
Fresh complete exact-head Actions remain required.

The `ba1d928` source passed all four native laws and the original CLI/editor
runtime tests, then correctly failed the Vue helper's exact six-case receipt
because the new capture directory shared that original case namespace. Move
only the new evidence to the dedicated wildcard-selected sibling directory;
the six original cases, receipt and all diagnostic/projection oracles remain
unchanged. The immutable artifact collector must retain all whole new packets
from this location, and fresh complete Actions remain required.

The `c06485f` source's tooling gate rejected the suffix assertion used to
check trace termination. Require every complete newline-inclusive record to
equal exactly `--api\n` or `--lsp\n` instead; this also rejects partial and
CRLF records directly. The nonempty trace and whole two-message error oracle
remain mandatory. No assertion allowlist entry or original oracle changes.
