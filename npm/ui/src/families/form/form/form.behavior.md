# Form behavior contract

Normative state x input -> outcome table for the Standard Schema form foundation
(`@vizejs/ui/form`). Every row is proven by the named test in
`src/families/form/form/form.test.ts`; compile-only assertions live in
`src/families/form/form/form.types.test-d.ts`.

| #   | State        | Input                                | Outcome                                                       | Proven by                                                           |
| --- | ------------ | ------------------------------------ | ------------------------------------------------------------- | ------------------------------------------------------------------- |
| F1  | any          | Standard Schema issue path           | path formats into a deterministic HTML field name             | `formats Standard Schema paths as form field names`                 |
| F2  | failed       | Standard Schema issues               | field errors preserve messages, names, and unwrapped paths    | `normalizes Standard Schema failures into field and summary errors` |
| F3  | failed       | duplicate field errors               | summary fields keep the first error per document id           | `deduplicates summary fields while preserving all field errors`     |
| F4  | any          | sync or async schema validation      | result normalizes into success or field-summary failure state | `validates sync and async Standard Schemas`                         |
| F5  | any          | malformed schema, result, or options | throws a stable form diagnostic                               | `rejects malformed schemas, results, and options`                   |
| F6  | valid        | field receives a matching error      | `useFormField` supplies an invalid flag for field wiring      | `wires field invalid state from normalized errors`                  |
| F7  | invalid      | field errors feed summary composable | `useFormErrorSummary` exposes existing error-summary fields   | `feeds normalized errors into an error summary controller`          |
| F8  | public types | mismatched schema input or mutation  | compilation rejects misuse                                    | `src/families/form/form/form.types.test-d.ts`                       |
| F9  | native form  | constraint-invalid controls          | errors normalize through the same field and summary pipeline  | `normalizes native constraint validation failures`                  |
| F10 | editing      | field visit, change, and blur events | dirty, visited, and touched state are tracked per field       | `tracks field state and focuses the first registered invalid field` |
| F11 | invalid      | submitted form has registered fields | focus moves to the first invalid field in current error order | `tracks field state and focuses the first registered invalid field` |
| F12 | validating   | a slower validation finishes last    | stale results return to their caller but cannot replace state | `discards stale async validation results`                           |
| F13 | submitting   | latest validation succeeds           | `onSubmit` runs once for the latest valid result              | `submits only the latest valid result`                              |
