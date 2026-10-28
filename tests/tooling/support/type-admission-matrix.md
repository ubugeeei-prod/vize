# Independent type admission

`canon-type-admission.test.ts` and `lsp-type-admission.test.ts` use original public
fixtures, independently specified authored coordinates, and arrays that retain
duplicate diagnostics. Performance samples are separate from these untimed
correctness probes. Passing them is one admission layer, not a claim that the
whole typechecker is correct.

The 30-mutation ref suite proves exact values, absence of unintended `any`, `unknown`,
and `never`, nullability, readonly, and exposed component argument/return/property
types. Every mutation has its own SFC filename and authored diagnostic token.
Stages are valid, invalid, repaired, poisoned proof constraints, and repaired
constraints. Poisoning `Assert<true>` detects files whose script proof declarations
were skipped or suppressed. LSP additionally queries typed hover and requires
diagnostics at the changed document version before accepting repair.

Vue's control explicitly configures the composable aliases `r` and the raw
Unicode-escaped callee spelling, because its parser matches configured names. Vize's subject must resolve the
actual Vue import binding and preserve local shadowing. Explicit type arguments
retain the authored Vue helper contract, including arbitrary keys and intentional
type overrides; inferred template refs retain DOM/component/generic inference.

| Axis              | Initial executable coverage                                                                                                                                                               | Further admission required                                                                                                  |
| ----------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Ref type evidence | Explicit/inferred DOM, aliases, Unicode escapes, auto-imported globals, two type arguments, override, local shadow, nested calls, mixed calls, exposed instance, generic payload, `$refs` | Namespace/destructured binding variants, callback refs, v-for arrays, nested scopes and updates combined                    |
| Source ownership  | Unique invalid member/argument/write/nullability at an independently computed source token; proof liveness                                                                                | Every emitted diagnostic feature and suppressed unowned copy, projection AST and semantic-map audit                         |
| Features          | Props instantiate the generic exposed ref; explicit exposed signatures                                                                                                                    | Independent props/emits/slots/flow/import/global/directive mutations with repair, typed queries and type witnesses          |
| Dependency graph  | Direct original child and generic imports                                                                                                                                                 | Diamond, barrel/reexport, SCC, deep chain, cross-package aliases and project references; independent and mixed              |
| Single-file size  | Unicode before a same-line probe, CRLF input                                                                                                                                              | Real authored 128 KiB, 1 MiB and 10 MiB SFCs with script/template probes near start/middle/end and mapping boundaries       |
| Type complexity   | Generic payload inference and exact method parameter/return witnesses                                                                                                                     | Conditional/mapped/distributive/generic/union/recursive cases; preserve legitimate compiler depth-limit diagnostics         |
| Corpus admission  | Unique mutations in every initial ref SFC                                                                                                                                                 | Untimed unique script/template mutation coverage in every canonical DAG cluster, then every SFC; private corpus stays local |
| Delivery surfaces | CLI and Vue control tests; actual LSP diagnostic/hover test                                                                                                                               | Content mapper with the identical native backend and public installed consumer acceptance at the release identity           |

The scale matrix must vary these axes independently before combining them. A
small-file count result, clean output, matching diagnostic set, initialized LSP,
or a hot cache does not satisfy the 2,500-SFC cold full-diagnostic target. Neither
skipped branches nor widened types may be used to meet the performance budget.
