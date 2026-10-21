# Compiler fix-history map and diagnostic slice — #6880

The compiler fix-history ledger remains incomplete: its pinned census contains
626 non-merge fix subjects, while this change advances bounded semantic review
from eight to ten SFC fixture scopes. Whole-fix review, the semantic denominator,
native acceptance, other targets, and other dialects remain unchanged.

This slice registers two existing complete public SFC observations:

- `0494c409b07a58c6296cdc2f50739b8e947366f4` (#3604), whose original
  source-map-enabled Counter recipe retains the complete generated module,
  structured map, authored source content, bindings, and diagnostics; and
- `7b821f8d0c46c56ab48d0a954def515a9219838e` (#750), whose original
  props-destructure mismatch retains the complete compiler error code, message,
  and location. Its existing capture separately retains the document-relative
  located-validator result.

No output is regenerated or inferred. The differential manifest points at the
committed first observation from the repeated `72293b11` capture and its exact
per-case reference packets. Both references retain their original inputs,
options, entrypoints, fix provenance, build receipt, and hashes. The shared SFC
observer now executes all ten registered cases so an Actions-built binary can
prove current source behavior instead of validating synthetic packets alone.

The registered comparison remains the complete public SFC Result contract. It
does not grant bundler, editor, mounted-runtime, SSR, Vapor, alternate-syntax,
or native compiler credit. Every one of the 626 fix subjects still retains
whole-fix target and dialect review pending, so #6880 must remain open.
