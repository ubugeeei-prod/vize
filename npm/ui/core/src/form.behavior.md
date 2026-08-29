# Form behavior contract

Normative state x input -> outcome table for the Standard Schema form foundation
(`@vizejs/ui/form`). Every row is proven by the named test in
`src/form.test.ts`; compile-only assertions live in
`src/form.types.test-d.ts`.

| #   | State        | Input                                | Outcome                                                       | Proven by                                                           |
| --- | ------------ | ------------------------------------ | ------------------------------------------------------------- | ------------------------------------------------------------------- |
| F1  | any          | Standard Schema issue path           | path formats into a deterministic HTML field name             | `formats Standard Schema paths as form field names`                 |
| F2  | failed       | Standard Schema issues               | field errors preserve messages, names, and unwrapped paths    | `normalizes Standard Schema failures into field and summary errors` |
| F3  | failed       | duplicate field errors               | summary fields keep the first error per document id           | `deduplicates summary fields while preserving all field errors`     |
| F4  | any          | sync or async schema validation      | result normalizes into success or field-summary failure state | `validates sync and async Standard Schemas`                         |
| F5  | any          | malformed schema, result, or options | throws a stable form diagnostic                               | `rejects malformed schemas, results, and options`                   |
| F6  | valid        | field receives a matching error      | `useFormField` supplies an invalid flag for field wiring      | `wires field invalid state from normalized errors`                  |
| F7  | invalid      | field errors feed summary composable | `useFormErrorSummary` exposes existing error-summary fields   | `feeds normalized errors into an error summary controller`          |
| F8  | public types | mismatched schema input or mutation  | compilation rejects misuse                                    | `src/form.types.test-d.ts`                                          |
