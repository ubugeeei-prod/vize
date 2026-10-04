# Public Node formatter line endings (#7736)

Issue: [#7736](https://github.com/ubugeeei-prod/vize/issues/7736).
Configuration roadmap: [#6098](https://github.com/ubugeeei-prod/vize/issues/6098).

At main `ac1675fe`, schema/generated config types, Rust, CLI, editor config and
WASM carry `endOfLine`, but public Node `formatSfc` omits that field. JavaScript
silently receives LF and TypeScript cannot pass the existing formatter option.
Expose just this option through a NAPI string enum with the same four lowercase
values. NAPI validates the provided value before entering formatting and forwards
it to the existing Glyph options. Omission keeps LF; `auto` uses the source
resolver completed by #7704. Input/options stay immutable. No printer, pipeline,
range/source-map/result shape or new profile is introduced.

The package's existing generated declaration and loader carry the new enum and
optional field. A real consumer checks interoperability with generated config
types and rejection of unsupported values. Public exported Node calls cover all
terminators, Vue 2/2.7/3, Auto inference, explicit precedence, fallback, three-pass
fixed points, raw-body fidelity and `InvalidArg` for unsupported values.

The lower core fix #7745 adds the shared CLI corpus case with script/template/style
and authored LF raw text within explicit CRLF layout. This child tests Auto against
the same immutable input and original independently authored 170-byte reference;
CLI config remains explicit CRLF, so the core behavior is proven independently.
No duplicate case or reference recapture is added. Preserve original CLI inputs,
captures, the original 300 API history plans and nine public NAPI history plans.
The complete NAPI owner has one explicit transition from its original hash;
both original named functions remain byte-identical and independently pinned.
Unregistered owner drift or rewriting the original identity fails admission.
Current-source execution must still prove every original Node result/error.

This bridge is the upper slice of a verified native Stack above #7745. Both exact
heads, all 104 unchanged instruction ceilings and a protected queue merge are
required before scoped completion. No native provider acceptance credit
is granted by these legacy formatter results. Hold new queue admission until the
active release publication freeze is lifted.

#6098 remains open for profiles, arbitrary configuration combinations and complete
range/source-map acceptance. Separate audit TODO: `quoteProps` and `jsxSingleQuote`
exist in schema/config types, but the current Rust-to-Oxc option projection omits
their fields; prove and address those independently rather than expanding this
Node entrypoint repair.
