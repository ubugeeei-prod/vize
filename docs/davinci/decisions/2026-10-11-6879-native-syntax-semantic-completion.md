# Complete native semantics after a parser abort (#6879)

Whole actual native output at original producer
`605429c660b328bb03bec5bbfef15961e59fcaf0` proves that the injected Element Plus
and Hoppscotch roots were included and read. Native CLI returned parser errors
instead of the injected TS2322. Its diagnostic-shaped output was accepted as
a completed answer, then selected-file mapping produced an empty result.

Before mapping or suppression, a nonzero CLI result now checks complete actual
reported JS/TS file bodies through the existing OXC parser/source-type
authority. Each distinct body is parsed at most once. A confirmed parser
failure refuses CLI semantic completion and uses the existing session/file
diagnostic route attached to the same generated configuration. The session's
CLI degradation meets the same refusal, and failed completion preserves both
the original whole native error body and the session failure. Ordinary scoped
semantic findings retain their existing fast path; result success is not
blindly propagated across unrelated selected files.

The independently authored finite native fixture retains the entire original
malformed Hoppscotch declaration, the entire injected SFC and original generated
module, their producer/upstream/provider hashes and MIT license. It requires
the actual native CLI to reproduce syntax-only abort, then the source-built
checker to return original TS1128 plus injected TS2322 at authored 2:7. Repair
changes only the injected assignment and retains TS1128. An unavailable
API/LSP control still uses the actual CLI and must fail explicitly with the
original body rather than reporting empty clean.

No source member, config field, declaration suffix, syntax error, native
diagnostic or original consumer deadline is removed. The original whole
TS5102 packet remains retained and its existing option-probe policy is
unchanged. This is the top dependent slice after quoted setup-anchor and JSX
spread-root generator repairs. Source-built Actions, complete native controls,
original full consumer qualification, protected merge and release are pending.
