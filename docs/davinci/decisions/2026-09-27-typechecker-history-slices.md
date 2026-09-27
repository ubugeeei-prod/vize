### Type-checker history fixture slices

[#6879](https://github.com/ubugeeei-prod/vize/issues/6879) retains 271 directory fix requirements; native/full-range admission and Actions/full T1 proof remain open.

| Slice                                                                        | Required T1 diagnostic contract                                                    | Remaining scope                                                     |
| ---------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| [Event handlers](./2026-09-27-typechecker-fix-history.md)                    | Three #4996 inputs, every batch row without filtering or normalization             | Whole bundled patch, shared/native comparison and #6849 replacement |
| [Definite assignment](./2026-09-27-template-definite-assignment-fixtures.md) | 18 original SFCs and one TSX SFC; all seven diagnostics and an empty project       | Whole bundled patch, full-range and native equivalence              |
| [Required props](./2026-09-27-required-props-diagnostic-fixtures.md)         | All 24 original identities, full messages/order; actual astral-prefix UTF16 column | Whole bundled patch, full-range and native equivalence              |
| [Unicode values](./2026-09-27-unicode-value-diagnostic-fixtures.md)          | Japanese/emoji literal diagnostics; production byte-corruption fault fails         | Broader template/comment paths, full-range and native equivalence   |
| [Reserved expressions](./2026-09-27-reserved-expression-diagnostics.md)      | Shorthand, keys, members and literal boundaries; production shorthand fault fails  | Broader scanner inputs, full-range and native equivalence           |
