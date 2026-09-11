# Toolchain testbed behavior contract

Normative state × input → outcome table for the UI source corpus used by
Atelier, Patina, Glyph, and Canon. Every row is proven by the named test in
`src/authoring/toolchain-testbed.test.ts` or by the package `check` script itself.

| #   | State          | Input                     | Outcome                                                                  | Proven by                                                              |
| --- | -------------- | ------------------------- | ------------------------------------------------------------------------ | ---------------------------------------------------------------------- |
| T1  | package script | `pnpm check`              | SFC lint and DOM/SSR/Vapor renderer compilation run before typecheck     | `package check keeps the UI source corpus on the toolchain gate`       |
| T2  | formatter gate | `pnpm fmt`                | source, scripts, and Vite config stay on the same package-local fmt lane | `package check keeps the UI source corpus on the toolchain gate`       |
| T3  | renderer gate  | authored Vue SFC files    | DOM, SSR, and Vapor lanes compile every source fixture                   | `renderer conformance script owns the DOM, SSR, and Vapor lanes`       |
| T4  | linter gate    | authored Vue SFC files    | opinionated type-aware Patina authoring rules run on every source SFC    | `sfc lint script owns the Patina opinionated authoring lane`           |
| T5  | typecheck gate | catalogued type contracts | compile-only public contracts remain included by the static check lane   | `static typecheck keeps catalogued public contracts on the Canon lane` |
