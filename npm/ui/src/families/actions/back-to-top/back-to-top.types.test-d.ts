/** Compile-only assertions for the public BackToTop contract. */

import type {
  BackToTopBehavior,
  BackToTopExpose,
  BackToTopSlotState,
  BackToTopState,
  BackToTopTarget,
} from "./back-to-top.ts";
import { BackToTop } from "./back-to-top.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const api: BackToTopExpose;
declare const slot: BackToTopSlotState;

type _Behavior = Expect<Equal<BackToTopBehavior, "auto" | "smooth">>;
type _State = Expect<Equal<BackToTopState, "hidden" | "visible">>;
type _Target = Expect<Equal<BackToTopTarget, HTMLElement | string | null>>;
type _Element = Expect<Equal<typeof api.element, HTMLButtonElement | null>>;
type _ScrollTop = Expect<Equal<typeof slot.scrollTop, number>>;
type _Refresh = Expect<Equal<ReturnType<typeof api.refresh>, number>>;

const props: InstanceType<typeof BackToTop>["$props"] = {
  ariaLabel: "Back to top",
  behavior: "auto",
  focusTarget: "#main",
  target: null,
  threshold: 600,
  "onScroll-top": (container: HTMLElement | Window) => container,
};

// @ts-expect-error behavior is a closed union.
const badBehavior: InstanceType<typeof BackToTop>["$props"] = { behavior: "instant" };

// @ts-expect-error threshold is numeric.
const badThreshold: InstanceType<typeof BackToTop>["$props"] = { threshold: "400" };

void badBehavior;
void badThreshold;
void props;
