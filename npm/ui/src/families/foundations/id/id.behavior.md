# Deterministic ID behavior contract

Normative state × input → outcome table for
`src/families/foundations/id/deterministic-id-provider.vue` and `@vizejs/ui/id`.
Every row is proven by the named test in
`src/families/foundations/id/id.test.ts`; compile-only public type assertions
live in `src/families/foundations/id/id.types.test-d.ts`.

| #    | State                     | Input                                                  | Outcome                                                     | Proven by                                                           |
| ---- | ------------------------- | ------------------------------------------------------ | ----------------------------------------------------------- | ------------------------------------------------------------------- |
| ID1  | root scope                | explicit prefix and string seed                        | namespace is `prefix-seed`                                  | `creates immutable request-local scopes with independent sequences` |
| ID2  | root scope                | repeated hint                                          | monotonically increasing unique IDs                         | `creates immutable request-local scopes with independent sequences` |
| ID3  | root scope                | child scope allocation                                 | child and parent ID counters remain independent             | `creates immutable request-local scopes with independent sequences` |
| ID4  | nested scope              | duplicate child seeds                                  | allocation index prevents collisions                        | `keeps duplicate nested provider seeds collision-free`              |
| ID5  | nested scope              | explicit prefix override                               | child prefix changes without losing parent namespace        | `supports a nested namespace prefix override`                       |
| ID6  | any scope                 | unsafe prefix, seed, hint, or numeric seed             | stable diagnostic rejects the value                         | `rejects namespace values that are unsafe to compose`               |
| ID7  | consumer ID               | valid punctuation or Unicode                           | exact consumer ID is accepted and branded                   | `validates explicit IDs and derives semantic parts`                 |
| ID8  | consumer ID               | empty, whitespace, or ASCII control                    | stable diagnostic rejects the value                         | `validates explicit IDs and derives semantic parts`                 |
| ID9  | component setup           | no provider                                            | Vue `useId()` supplies a hydration-stable seed              | `uses Vue's application ID sequence without a provider`             |
| ID10 | component setup           | provider                                               | nearest request-local scope allocates the ID                | `allocates descriptive IDs from the nearest provider`               |
| ID11 | component setup           | reactive explicit ID appears                           | explicit ID replaces the generated fallback                 | `preserves one fallback across reactive explicit ID changes`        |
| ID12 | component setup           | reactive explicit ID disappears                        | original generated fallback is restored without renumbering | `preserves one fallback across reactive explicit ID changes`        |
| ID13 | outside setup             | call composable                                        | stable setup diagnostic is thrown                           | `rejects composable use outside component setup`                    |
| ID14 | provider slot             | render                                                 | slot receives validated namespace and prefix                | `exposes the resolved namespace to its slot and public instance`    |
| ID15 | nested providers          | mount                                                  | parent and nested consumer IDs are unique                   | `keeps duplicate nested provider seeds collision-free`              |
| ID16 | SSR request               | repeat identical tree and seed                         | byte-identical IDs are rendered                             | `renders byte-stable IDs for repeated and concurrent SSR requests`  |
| ID17 | concurrent SSR            | different request seeds                                | request-local sequences never bleed across renders          | `renders byte-stable IDs for repeated and concurrent SSR requests`  |
| ID18 | SSR followed by hydration | identical provider tree                                | client retains server IDs without mismatch diagnostics      | `hydrates provider IDs without warnings or replacement`             |
| ID19 | nested provider insertion | sibling control allocation                             | child-scope counter cannot renumber sibling IDs             | `keeps ID and child-scope allocation sequences independent`         |
| ID20 | public types              | invalid seed, hint, prefix, or unbranded ID assignment | TypeScript rejects closed-contract misuse                   | `src/families/foundations/id/id.types.test-d.ts`                    |

## Assistive-technology notes

The primitive only produces IDs; components remain responsible for assigning
them to correct relationships such as `for`, `aria-labelledby`,
`aria-describedby`, `aria-controls`, and `aria-errormessage`. Stable IDs avoid
relationship loss during hydration, but do not make an invalid relationship
accessible.

## Escape hatches and constraints

- Pass an explicit `id` when an external document contract owns the value.
- Give separate SSR islands distinct provider seeds, or configure Vue's
  application `idPrefix`, when their DOM is combined into one document.
- Allocate generated IDs during setup. Calling a scope's `nextId()` from a
  render loop intentionally changes the sequence and is unsupported.
- User-owned IDs allow Unicode and punctuation. Escape them before using them
  in a CSS selector.
