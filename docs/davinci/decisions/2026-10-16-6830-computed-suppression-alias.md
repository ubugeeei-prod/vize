# Preserve the ESLint computed-side-effect suppression alias

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The ReKa lint row in [App E2E run 38866347055](https://github.com/ubugeeei-prod/vize/actions/runs/38866347055)
newly reports `script/no-side-effects-in-computed-properties` on line 75 of
`NavigationMenuContentImpl.vue`. Its authored line 74 already disables
`vue/no-side-effects-in-computed-properties` for that next line. The pinned
source is [ReKa c23e0ed](https://github.com/unovue/reka-ui/blob/c23e0ed8a38c66cd4f9350e7a46c6b502a8491c2/packages/core/src/NavigationMenu/NavigationMenuContentImpl.vue#L74).

The committed ESLint-to-Patina rule catalog maps that Vue rule to the script
rule. Suppression normalization covered only the reserved-key and duplicate-key
renames, so the new Composition API diagnostic bypassed its existing directive.
Add this one explicit alias to the same normalization. Keep the computed-write
detector, severity, diagnostic content, and unrelated rule identities unchanged.

Preserve the whole original ReKa SFC in the differential fixture collection.
Require no finding with its authored directive and the exact original finding
after removing only that directive. Cover ESLint/Oxlint next-line and inline
scope, canonical IDs, plugin-prefixed IDs, region disable/enable across the
alias, unrelated rules, wrong lines, and directive text inside a string.

This fixes a product suppression regression; it does not authorize a ReKa
snapshot refresh or suppress unsanctioned computed mutations. All four
`computed_suppression_history` integration tests pass after recompiling Patina
from the repair worktree. The complete App E2E lint row remains required.
