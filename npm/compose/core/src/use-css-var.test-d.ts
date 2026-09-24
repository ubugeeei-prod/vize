/** Compile-only assertions for the `use-css-var` type contracts. */

import type { Ref } from "vue";

import { useCssVar } from "./use-css-var.ts";
import type { CssVarTarget } from "./use-css-var.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const accent = useCssVar("--accent");
type _ValueIsWritableString = Expect<Equal<typeof accent.value, Ref<string>>>;

declare const element: HTMLElement;
element satisfies CssVarTarget;
useCssVar("--gap", () => element);

// @ts-expect-error variable values are strings.
accent.value.value = 1;

// @ts-expect-error targets need a style declaration.
useCssVar("--gap", {});
