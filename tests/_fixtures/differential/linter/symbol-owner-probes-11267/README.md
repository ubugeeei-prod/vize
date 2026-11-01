These are causal reductions for #11267, not the reporter's unavailable 1027-file
application. Each real TypeScript declaration is queried in a fresh connection,
both through an imported function and an inline declaration. The same-session
reference explicitly fetches full parameter symbols before querying their types.

The native test preserves the cold result, complete signatures, parameter symbols,
types, reference result, and close result before asserting whole probe equality.
It does not classify a cold failure as the reported owner error unless the real
checker returns that error. Required, optional, rest, explicit-this, overload,
generic and zero-parameter forms retain declaration order and all returned types.

The CLI also consumes App.vue and the typed/untyped dependency declarations in
four phases: typed, untyped, repaired, restored. Complete authored diagnostic
objects and raw process results are retained before assertions. A strict-boolean
error stays present throughout, so removing the owner error cannot hide other
typed findings. This CLI check is independent of the initial direct-probe proof.

Actions uses the existing stable checker and the explicitly reported
tsrs@0.20.1-ts7.1.0-beta independently. This is not an original-application,
macOS, public-package, or large-project qualification.
