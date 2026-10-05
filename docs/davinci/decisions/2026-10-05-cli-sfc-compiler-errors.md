# Build rejects SFC results carrying compiler errors

Decision for [#7879](https://github.com/ubugeeei-prod/vize/issues/7879).

The SFC compiler can return `Ok` with partial script code and a non-empty
`errors` vector after a template failure. The build adapter counted that result
as compiled, wrote a component without its render function and exited zero.
Use a private result check to join every compiler error message and enter the
existing compile-failure path before output, dump capture or success accounting.
Both emitted and stats-only paths use it; the latter retains failure caching.
The compiler API result itself stays unchanged, including its diagnostic vector.

Normal failed builds emit no artifact and exit one. Explicit
`--continue-on-error` retains the existing template-only failure contract:
exit one, an empty JS failure artifact or a JSON failure object carrying every
message. It never treats the partial component as compiled. Successful output,
warnings, profiling and stage order retain their existing paths.

Preserve the complete three original SFCs and original report bytes. Real Rust
compiler calls supply complete error messages and valid module bytes to actual
CLI stdio laws across JS/JSON/stats and continue-on-error. Only the validated
four-decimal finite duration is normalized; exit, full stdout/stderr, source and
all emitted artifact bytes are compared. Derived Options API, unsupported Vapor
memo, repeated invalid stats sources and valid script/template/DOM-memo controls
remain distinct.

Initial hosted execution exposed a fixture expectation missing the existing
`./` prefix for glob-discovered files. Correct only that full stderr oracle;
retain all forty subprocess cases. Replace the extra diagnostic substring
assertion with complete frozen error vectors and regenerate the source-derived
Croquis inventory for the new private `SfcError` import. No gate is relaxed.

The independent official Vue 3.5.35 SFC compiler supplies complete pinned parse
and template error vectors, including source positions, for seven whole inputs.
Actual source-built NAPI calls compare full script/template-only error vectors.
DOM accepts the memo control; Vize's current Vapor backend still refuses it.
No official rc.10 byte parity, native compiler migration, new Vapor support or
complete fix-history credit is claimed.

Exact-source Actions, unchanged protected suites/ceilings, actual merge and
release remain required. #6880 stays open.
