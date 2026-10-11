# Type-checking performance reader flow

Refs: #8365 (More guides) and #8372 (natural Japanese).

## Decision

Add a short purpose and CLI next-step link to the English worker/profile guide,
and author its complete Japanese counterpart. Keep the entire former English
body intact after the introduction. The Japanese input is marked as reviewed
so the existing translation collector preserves it on regeneration.

Both original shell recipes, every inline option/API token and the #3905
issue URL remain exact. Worker and server settings retain their distinct
meanings. The unset checker count stays one; explicit CLI selection overrides
VIZE_CHECKERS, and None restores environment/default selection for Rust callers.
The diagnostic limitation is the historical comparison described by #3905,
not a new all-project or installed-release conclusion.

Current source authority is the CLI CheckArgs nonzero checker field,
crates/vize_canon/src/batch/executor/cli/checkers.rs, its existing count tests,
CorsaExecutor::set_checker_count and cli/shard_sizing.rs. The guide explains
profiling and worker selection. Its final cache paragraph describes path
aliases, matching commands/check/imports_alias_scaling_tests.rs and
imports_registration.rs; it does not promise a measured speedup or
claim the type checker's performance target is achieved.

## Delivery boundary

The finite More coverage entry now includes the real authored Japanese page
and retains every other route and classification. Existing collector and More
checks, native rendered reading flow and stable public source acceptance are
required before counting this bounded guide as delivered. The site-wide
Japanese and More issues remain open.

Root reviewed all four concrete files and the preservation packet; the first
Japanese Corsa mention now identifies it as the type-checking engine.
Prepared independently while the casing repair qualifies. No casing product,
browser-helper or active foundation line is edited by this preparation.
