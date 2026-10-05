# Preserve expression grouping while removing the formatter wrapper

Issue: [#7923](https://github.com/ubugeeei-prod/vize/issues/7923).

The expression formatter prints a reusable `void (…)` wrapper, then formerly
removed a leading and final parenthesis without knowing which groups they
belonged to. Grouped member calls became invalid source, and unparenthesized
sequences changed the argument seen by Vue's interpolation/binding consumers.
A later parse refusal could disguise the broken output as a stable fixed point.

Retain the existing parsed wrapper AST, arena reset, source buffer, single
formatter stage and quote policies. Its sole unary argument determines whether
a printed parenthesis pair belongs to the removable wrapper: call/member/chain,
non-null/instantiation roots retain their authored grouping; root sequences
retain the grouping required by the consumer. Existing lower-precedence wrapper
removal remains intact. A wrapped multi-statement input refuses extraction.
No second parsing pass, textual group scanner or new pipeline stage is added.

The sixteen-row legacy corpus retains the complete original SFC and all named
condition/event/as/satisfies/IIFE/sequence reproductions, plus nested sequences,
non-null/optional/computed members, quoted parentheses, and ordinary binary,
call, object and regexp controls. Public API and actual source-built CLI compare
whole SFC bytes over three passes and check/write process custody. The existing
pinned official Vue compiler/runtime parses both whole templates, executes DOM
and SSR across every complete state, checks independently written DOM results
and click updates, and compares all before/after rendered states. Raw source
receipt, complete process streams, compiler modules and runtime observations
are retained in the existing always-uploaded differential evidence directory.

This corrects legacy behavior without native-stage or performance credit.
Historical capture bytes, instruction ceilings and source limits stay intact.
Exact-head Actions, protected full suites, actual merge and release inclusion
remain required; a stable but semantically invalid output is not acceptance.

The strict current source-owner witness hash tracks the changed script wrapper
file. Every original capture SHA, revision, retained Rust function body hash
and expected byte remains unchanged; the new helper does not waive source custody.
