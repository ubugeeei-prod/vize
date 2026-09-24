/** Compile-only assertions for the `use-style-tag` type contracts. */

import type { Ref } from "vue";

import { useStyleTag } from "./use-style-tag.ts";
import type { StyleTagElement } from "./use-style-tag.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const style = useStyleTag("a {}", { id: "links" });
type _CssIsWritable = Expect<Equal<typeof style.css, Ref<string>>>;

declare const element: HTMLStyleElement;
element satisfies StyleTagElement;

// @ts-expect-error css is text.
useStyleTag({ color: "red" });

// @ts-expect-error the loaded flag is read-only.
style.loaded.value = true;
