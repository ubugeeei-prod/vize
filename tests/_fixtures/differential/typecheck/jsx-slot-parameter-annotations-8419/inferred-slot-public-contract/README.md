# Original inferred-slot oracle with explicit JSX

These five input byte sequences are copied unchanged from the existing
`canon-template-public-contracts.test.ts` inferred-slot case. The wrong
`isActive: string` callback retains its original `@ts-expect-error`; all five
files must have complete empty diagnostics. The explicit Vize config enables
JSX without changing the original input or diagnostic oracle. The compiler
options and program membership are asserted in the complete report.

Before the native repair the original config Actions reports unused
`@ts-expect-error` TS2578; after the repair the genuine source CLI/public Corsa
7.0.2 returns the complete clean packet. Source Actions exercise this same
fixture as a required native CLI law.
