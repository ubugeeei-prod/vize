# FormWizard behavior contract

Normative state x input -> outcome table for `form-wizard.vue`,
`form-wizard-step.vue`, `form-wizard-next.vue`, `form-wizard-back.vue`, and
`form-wizard-progress.vue` (`@vizejs/ui/form-wizard`). `steps` infers the step-id
union used by `v-model`, gates, events, and the expose API. Every row is proven
by the named test in `form-wizard.test.ts` or `form-wizard-ssr.test.ts`;
inference is pinned in `form-wizard.types.test-d.ts`.

| #   | State           | Input                             | Outcome                                                                                                                                                                    | Proven by                                                                           |
| --- | --------------- | --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| W1  | first step      | render                            | labelled `role="group"`; every panel renders (`hidden` unless current, `aria-current="step"`); Back disabled; native progress "Step 1 of 3"                                | `renders every step panel with only the current one visible`                        |
| W2  | any             | Next / Back                       | Next advances (or emits `complete` on the last step), Back returns; `change(step, previous, direction)` fires and focus moves to the new panel                             | `next and back move between steps, focus the panel, and complete on the last step`  |
| W3  | gated           | Next                              | the gate receives `{ step, target, signal }`; while pending the root is `aria-busy` and Next is disabled; `false` keeps the step and emits `blocked`; Back never validates | `validation gates block forward moves and report the blocked step`                  |
| W4  | linear          | `goTo`                            | backward jumps are free; forward jumps need visited targets (unless `linear=false`) and validate every step in between                                                     | `goTo jumps back freely, forward only through visited steps, validating in between` |
| W5  | draft store     | mount / change / complete / reset | the draft loads after mount (unknown steps ignored), saves `{ step, visited }` after each change, and clears on completion and reset                                       | `drafts restore after mount, save every change, and clear on completion or reset`   |
| W6  | controlled      | Next                              | emits the request; the rendered step follows `modelValue`                                                                                                                  | `controlled steps win until the parent accepts them`                                |
| W7  | invalid setup   | empty `steps` / missing root      | throws `VIZE_UI_FORM_WIZARD_STEPS` or `VIZE_UI_CONTEXT_MISSING: FormWizard`                                                                                                | `rejects empty step lists and parts outside a FormWizard`                           |
| W8  | SSR / hydration | isolated requests                 | the server renders the default step; drafts apply only after hydration, so markup matches and no mismatch is reported                                                      | `renders the default step on the server and applies drafts only after hydration`    |

## Public extension contract

| Surface         | Contract                                                                                                                       |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Parts           | `root`, `step` (panel), `next`, `back`, `progress`.                                                                            |
| Data attributes | root `data-state` (`in-progress`/`validating`/`complete`) and `data-step`; panel `data-state` (`active`/`visited`/`upcoming`). |
| CSS properties  | `--vize-form-wizard-progress` on the progress element (0%–100%).                                                               |
| Integration     | Gate each step with `useForm` validation (for example `validate: () => form.validate().then((r) => r.valid)`).                 |

The subpath is tree-shakable and ships no CSS; those package contracts are
pinned by `distribution.test.ts`, `check:size`, and `check:tree-shaking`.
