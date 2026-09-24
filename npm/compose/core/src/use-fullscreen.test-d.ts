/** Compile-only assertions for the `use-fullscreen` type contracts. */

import { useFullscreen } from "./use-fullscreen.ts";
import type { FullscreenElementLike, FullscreenHost } from "./use-fullscreen.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const element: HTMLElement;
const fullscreen = useFullscreen(() => element);
type _ToggleResolvesState = Expect<Equal<Awaited<ReturnType<typeof fullscreen.toggle>>, boolean>>;

element satisfies FullscreenElementLike;
document satisfies FullscreenHost;

// @ts-expect-error navigation UI is a closed union.
useFullscreen(undefined, { navigationUI: "never" });

// @ts-expect-error the state is read-only.
fullscreen.isFullscreen.value = true;
