# Command behavior contract

Normative state × input → outcome table for `@vizejs/ui/command`. Every row is
exercised by `src/command*.test.ts`; compile-only assertions live in
`src/command.types.test-d.ts`.

| #   | State                     | Input                                 | Outcome                                                                 | Proven by             |
| --- | ------------------------- | ------------------------------------- | ----------------------------------------------------------------------- | --------------------- |
| C1  | registered command        | `execute` with its identifier         | `run` receives a frozen execution context and the dispatch reports it   | dispatch test         |
| C2  | registered command        | `execute` with a payload and source   | payload and source thread through to the handler and observers          | dispatch test         |
| C3  | unknown identifier        | `execute`                             | dispatch reports `not-found` and no handler or observer runs            | routing test          |
| C4  | disabled command          | `execute` while `when` resolves false | dispatch reports `disabled` and `run` never fires                       | enablement test       |
| C5  | disabled router           | `execute` while `isDisabled` is true  | every command reads as disabled until the gate clears                   | enablement test       |
| C6  | registered identifier     | second registration of the same id    | stable conflict diagnostic rejects the duplicate, keeping the original  | conflict test         |
| C7  | released registration     | re-registration of the freed id       | registration succeeds and help metadata updates                         | conflict test         |
| C8  | any registrations         | `commands` read                       | reactive frozen help metadata lists id, title, keywords, and enablement | help-metadata test    |
| C9  | observer configured       | dispatch that found its command       | `onDidExecute` receives the frozen dispatch after the handler completes | observer test         |
| C10 | handler throws            | `execute`                             | the failure surfaces to the caller and no dispatch is reported          | failure test          |
| C11 | invalid options           | malformed definition or option values | stable runtime diagnostics reject the misuse                            | diagnostics test      |
| C12 | active router             | dispose or Vue scope stop             | registrations clear and imperative calls become terminal                | lifecycle test        |
| C13 | concurrent SSR requests   | identical consumers                   | byte-identical markup with no request-global registration state         | SSR test              |
| C14 | SSR followed by hydration | dispatch from a hydrated control      | host identity remains and the command runs without warnings             | hydration test        |
| C15 | DOM, SSR, and Vapor lanes | authored consumer compiles            | every renderer accepts the same router surface                          | renderer gate         |
| C16 | root and subpath consumer | only command is retained              | equal CSS-free bundles exclude unrelated component families             | tree-shaking gate     |
| C17 | public TypeScript API     | unknown identifier or invalid options | compile-only assertions reject misuse                                   | type declaration test |

## Accessibility obligation

The router is headless plumbing for palettes, menus, and shortcut layers; it
never owns semantics itself. Consumers must reflect enablement onto the
controls that trigger a command (`disabled` or `aria-disabled`), keep every
command reachable through visible focusable controls rather than shortcuts
alone, and use the help metadata (title, description, group) as the
accessible names of palette and menu items so the spoken and visible labels
stay identical.
