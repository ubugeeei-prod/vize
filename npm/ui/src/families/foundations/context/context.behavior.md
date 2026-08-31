# Typed context behavior contract

Normative state x input -> outcome table for `@vizejs/ui/context`. Every row is
proven by `src/families/foundations/context/context.test.ts`; compile-only
assertions live in `src/families/foundations/context/context.types.test-d.ts`.

| #   | State              | Input                     | Outcome                                                                                  | Proven by                                                  |
| --- | ------------------ | ------------------------- | ---------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| CT1 | valid name         | create context            | name is trimmed, the injection key keeps that description, and the contract is immutable | `creates an immutable context with an integration key`     |
| CT2 | app provider       | read required or optional | the exact provided value and generic type are returned                                   | `resolves application-provided values`                     |
| CT3 | explicit undefined | read required             | `undefined` is accepted as a deliberate provided value                                   | `distinguishes an explicit undefined value...`             |
| CT4 | missing provider   | read optional             | `undefined` is returned without throwing                                                 | `distinguishes an explicit undefined value...`             |
| CT5 | missing provider   | read required             | stable `VIZE_UI_CONTEXT_MISSING` diagnostic identifies the context name                  | `distinguishes an explicit undefined value...`             |
| CT6 | blank name         | create context            | stable `VIZE_UI_CONTEXT_NAME` diagnostic rejects unactionable names                      | `rejects names that cannot produce...`                     |
| CT7 | public type API    | incorrect key/value use   | compile-only assertions reject generic drift and key mutation                            | `src/families/foundations/context/context.types.test-d.ts` |

The context primitive is headless and SSR-safe because it only uses Vue's
provide/inject mechanism in the active app or component scope. Rendering,
accessible relationships, and lifecycle ownership remain with the concrete
component family that consumes the context.
