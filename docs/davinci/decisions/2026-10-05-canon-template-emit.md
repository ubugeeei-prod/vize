# Template emits from authenticated setup macros (#7820)

The original unassigned type-only `defineEmits` SFC and complete authored
TS2345 event-name/payload vector are retained without editing reporter bytes.
The template context currently reads the unconstrained public instance `$emit`,
which loses the component's macro contract.

Use the same setup macro return type (`__EmitFn`) for type-only declarations
inside the setup lexical scope. This preserves local aliases and generic type
parameters. Runtime declarations infer from the same setup helper overloads.
The existing semantic helper plan excludes an authored/shadowed `defineEmits`;
absent macros continue to use their public instance contract. No generated text
rewrite, extra pipeline stage, native-stage claim or blanket diagnostic mask is
introduced.

Required source-built CLI and official TypeScript native 7.0.2/Vue declaration
oracles compare complete output/status for the exact original, valid calls,
runtime array names and unconstrained payloads, renamed local call signatures,
shadowed macros and absent macros. The real editor service compares both complete
original diagnostics and the exact typed `$emit` hover/range. Raw original and
control bytes, full outputs and official binary/package/source hashes are kept
in the native-phase Actions artifact. Expectations are independently authored;
actual execution remains pending. The generated consumer surface inventory is
regenerated from source.

Fresh exact-head strict/source/native Actions and unchanged instruction ceilings
are required before readiness. The release owner's first-cut publication hold
continues to govern queue entry; protected full/104-platform suites, actual
signed merge, issue closure and public consumer qualification remain separate.

Qualification shares the existing pinned `npm/cli` real Vue dependency through
normal ancestor lookup; workspace root has no direct Vue dependency. Isolated
case paths use that package without installation or symlinks. The editor binary
is pinned to the same authenticated official oracle; complete editor fixture,
config and raw response bytes are retained before full assertions. Original
source and all independently authored expectations remain unchanged.
