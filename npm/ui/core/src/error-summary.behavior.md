# Error summary behavior contract

Normative state × input → outcome table for `error-summary.vue`
(`@vizejs/ui/error-summary`). Every row is proven by the named test in
`src/error-summary.test.ts` or `src/error-summary-ssr.test.ts`; compile-only
assertions live in `src/error-summary.types.test-d.ts`.

| #   | State        | Input                                | Outcome                                                | Proven by                                          |
| --- | ------------ | ------------------------------------ | ------------------------------------------------------ | -------------------------------------------------- |
| E1  | valid        | render                               | host renders, summary region stays out of the tree     | `stays out of the tree while every field is valid` |
| E2  | valid        | fields become invalid                | labelled group renders the invalid fields as links     | `lists invalid fields and takes focus`             |
| E3  | valid        | fields become invalid                | summary takes focus and captures the prior element     | `lists invalid fields and takes focus`             |
| E4  | invalid      | link click                           | focus moves to the named control, `fieldFocus` fires   | `moves focus to an invalid control from its link`  |
| E5  | invalid      | every field repaired                 | focus returns to the captured element, `restore` fires | `restores focus when every field is repaired`      |
| E6  | invalid      | focus moved elsewhere, then repaired | focus is not stolen and the capture is dropped         | `does not steal focus after a repair`              |
| E7  | invalid      | `autoFocus` false                    | summary renders without moving focus                   | `respects autoFocus false`                         |
| E8  | invalid      | `focusField` unknown id              | returns null and focus stays put                       | `moves focus to an invalid control from its link`  |
| E9  | any          | duplicate field ids                  | throws a stable option diagnostic                      | `rejects invalid fields options`                   |
| E10 | no scope     | `useErrorSummary`                    | throws a stable setup diagnostic                       | `rejects composable use outside an effect scope`   |
| E11 | disposed     | any controller call                  | throws a stable disposed diagnostic                    | `rejects use after dispose`                        |
| E12 | SSR          | render with fields                   | byte-identical labelled group and anchor list          | SSR test                                           |
| E13 | public types | malformed field                      | compilation rejects misuse                             | `src/error-summary.types.test-d.ts`                |
