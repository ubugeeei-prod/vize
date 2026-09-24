/** Compile-only assertions for useVisualViewport. */

import type { ComputedRef, Ref } from "vue";

import { useVisualViewport } from "./use-visual-viewport.ts";
import type { VisualViewportControls } from "./use-visual-viewport.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const viewport = useVisualViewport({ keyboardThreshold: 80, overlaysContent: true });

type _Controls = Expect<Equal<typeof viewport, VisualViewportControls>>;
type _KeyboardOpen = Expect<Equal<typeof viewport.keyboardOpen, ComputedRef<boolean>>>;
type _Height = Expect<Equal<typeof viewport.keyboardHeight, Readonly<Ref<number>>>>;

// @ts-expect-error thresholds are numbers.
useVisualViewport({ keyboardThreshold: "80" });

// @ts-expect-error hosts need a layout viewport height.
useVisualViewport({ host: { visualViewport: null } });
