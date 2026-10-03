# Native JS/TS local definitions and references

Decision for #6850, #6871 and #6883: an explicit
`experimental-source-navigation` feature adds `vize/nativeDefinition` and
`vize/nativeReferences` preview requests. They accept the standard definition
and references parameter shapes and return a local `Location` or location list.
The normal LSP routes and advertised capabilities retain their existing behavior.

The profile comes from the real open Document's `language_id`, captured with its
URI, process-global revision, client version and once-retained Arc source.
Only `javascript` and `typescript` are admitted. JSX/TSX, Vue, unknown languages,
recovered Programs and unsupported/incomplete native Files remain explicit
errors. Current TS variable annotations, destructuring and unresolved uses
remain refused. Imports resolve to the local import binding; external targets,
workspace indexing and type-definition lookup are unfinished.

On a cache miss the actual L1 producer parses the original full buffer once.
Its original admitted Program is checked against that same SourceRoot and
passed to the existing L2 FileProducer. Definition/reference projection uses
only this File's genuine BindingRef and resolved Reference targets, including
shadowed bindings. No name search, second parse, legacy semantic fallback,
additional core pipeline stage or serialization between levels is introduced.
The original syntax and File owners coexist during synchronous projection.

The project caches an owned response summary for that physical snapshot and
reuses it for both operations. Arena-owned AST/File owners do not cross an
await or remain in the response cache. Retention of all level artifacts/ASTs,
#6872 and the complete incremental product migration remain unfinished.
Equal foreign source copies and same-version reopenings cannot reuse the
summary: its snapshot identity and the host's global revision both matter.

UTF-8 spans map strictly to original LSP UTF-16 coordinates, preserving Unicode
and CR/LF/CRLF. Invalid lines, columns and surrogate interiors are refused.
Ready responses still pass SourceQueryProject's fresh currentness/cancellation
guard. Post-mutation lifecycle notification cancels only retired revisions,
preserving queries registered after the actual host update. Navigation
document/lifecycle/registry/cache locks never survive an await or a cancellation
wake. The change hook runs after the existing diagnostic guard is released;
fresh publication checks still reject edits before that notification.
Wire transport occurs after guarded publication.

Fifteen new source-authored laws cover local/shadowed/import bindings, physical
cache identity, native refusals, coordinate boundaries, host mutation/reopen,
stale/cancelled publication and complete JSON-RPC result/error envelopes through
the actual production service builder. They are unexecuted until fresh exact-head
Actions validates them. The full recipe adds named feature tests, minimal-feature
compilation and warning-denying Clippy while preserving all prior gates.
Protected queue and actual terminal merge are still required.

This is a bounded opt-in JS/TS capability, not native Vue admission, a default
product replacement or completion of the LSP fix-history gate. #6883 remains open;
original historical and actual merged #7466/#7467 evidence remains head-scoped.
