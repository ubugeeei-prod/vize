# Original File position queries (2026-10-03)

Decision for [#6871](https://github.com/ubugeeei-prod/vize/issues/6871).

`FileArtifact` provides `reference_at_offset`, `binding_at_offset` and
`scope_at_offset` over the original script-family rows. Each receives a
file-absolute UTF-8 byte offset and returns a borrowed result or a typed error.
`ReferenceRef` retains its actual File and resolves only its original target;
`ScopeRef` performs name lookup only in that same owner. Equal numeric IDs in
different Files do not establish ownership.

Queries first require `FileArtifact::is_complete()`. Successful canonical
`finish()` alone does not admit unsupported, unresolved, invalid-profile or
interrupted observations. Original issues and interruption records remain
unchanged. Invalid source offsets and non-UTF-8 boundaries are distinct errors.
Valid EOF and zero-width sites match nothing because spans are half-open.

The covering original Program unit must be unique. Source outside real unit
spans returns no script result; the synthetic File root never grants scope to
opaque SFC blocks or template expressions. Multiple recorded units or name
sites are ambiguous, independently of insertion order. Local import names and
export-local uses retain their original namespaces and targets; remote import
spellings, public export aliases and re-exports never acquire invented bindings.

At an original declaration/use site, scope lookup returns that row's actual
scope. A declared function name therefore belongs to the parent, while its
parameters/body belong to the recorded child. Elsewhere, lookup selects the
deepest containing recorded scope through its original ancestry; incomparable
overlapping scopes are refused. Recorded introduction spans do not imply
unrecorded lookup extents.

The provider scans existing arrays and parent links. It adds no parser, binder,
serialization, pipeline stage or query allocation. It does not claim an index
or a measured performance gain. Public queries lend immutable same-File views;
they do not grant native SFC or product admission.

Focused laws use actual admitted JS/TS observations and the real File producer.
They cover source origins, Unicode boundaries, half-open sites, nested scopes,
forward references, import/export aliases, type namespaces, reverse unit order,
overlapping original observations, canonical-only text, empty units, unsupported
profiles and script/template incompleteness. Compile-fail laws preserve private
constructors and the live File borrow. Exact-head Actions, unchanged instruction
ceilings and an actual protected merge remain required before delivery.

## Remaining implementation

| Slice | Required native behavior | Actual prerequisite |
| --- | --- | --- |
| File template queries | Resolve original retained template occurrence positions to the same File binding and scope | Actual completed template construction and authored span maps; no fabricated script unit |
| Maestro consumer | Reuse the shared position API and retain real current-snapshot artifacts per block/embed | This provider plus genuine artifact retention; default route replacement still requires #6883 |
| Document/petite | Apply case folding, HTML self-closing rules, implied end tags and table semantics in the native tree builder | Existing Document lexer profile; its declared constants alone are not implemented tree semantics |
| Vapor target | Write complete JS directly from the L3 program without the legacy IR generator | Genuine L3 decisions/schedule and runtime helper vocabulary; current L4 target is a skeleton |
| Product integration | Exercise the native producer/decision/emitter through complete public results | Real providers and per-product fix-history gates; opt-in `compile_sfc_native` is a consumer, not whole-product completion |

#6871 remains open for broader semantic queries and genuine product consumers.
The independent provider can merge without waiting for unrelated whole-Issue
closure; later consumers use a registered native Stack if they depend on it.
