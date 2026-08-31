# Pointer grace behavior contract

Normative state × input → outcome table for `@vizejs/ui/pointer-grace`. Every
row is exercised by `src/families/interaction/pointer-grace/pointer-grace*.test.ts`;
compile-only assertions live in
`src/families/interaction/pointer-grace/pointer-grace.types.test-d.ts`.

| #   | State          | Input                                   | Outcome                                           | Proven by                                                              |
| --- | -------------- | --------------------------------------- | ------------------------------------------------- | ---------------------------------------------------------------------- |
| G1  | idle           | origin and target set                   | polygon is the origin plus extreme target corners | polygon test                                                           |
| G2  | armed          | pointer inside the target               | pending timer is cleared                          | move inside test                                                       |
| G3  | armed          | pointer inside the safe triangle        | pending timer is cleared                          | move inside triangle test                                              |
| G4  | tracked        | pointer outside polygon                 | `onGraceEnd` fires after `delay`                  | leave delay test                                                       |
| G5  | pending        | dispose                                 | timer is released and further moves throw         | dispose test                                                           |
| G6  | any            | `usePointerGrace` outside a scope       | setup diagnostic is thrown                        | setup test                                                             |
| G7  | concurrent SSR | identical trees                         | byte-identical markup contains no timers          | SSR test                                                               |
| G8  | public types   | invalid delay or mutating readonly refs | compilation rejects misuse                        | `src/families/interaction/pointer-grace/pointer-grace.types.test-d.ts` |
