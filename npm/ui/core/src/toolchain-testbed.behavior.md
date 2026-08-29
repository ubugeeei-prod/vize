# Toolchain testbed behavior contract

Normative state × input → outcome table for the UI source corpus used by
Atelier, Patina, Glyph, and Canon. Every row is proven by the named test in
`src/toolchain-testbed.test.ts` or by the package `check` script itself.

| #   | State          | Input                  | Outcome                                                              | Proven by                                                        |
| --- | -------------- | ---------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------------- |
| T1  | package script | `pnpm check`           | SFC lint and DOM/SSR/Vapor renderer compilation run before typecheck | `package check keeps the UI source corpus on the toolchain gate` |
| T2  | renderer gate  | authored Vue SFC files | DOM, SSR, and Vapor lanes compile every source fixture               | `pnpm lint:sfc`                                                  |
| T3  | linter gate    | authored Vue SFC files | opinionated SFC authoring rules run on every source fixture          | `pnpm lint:sfc`                                                  |
