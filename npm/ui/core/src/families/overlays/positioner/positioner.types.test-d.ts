/** Compile-only assertions for the public positioner contract. */

import { ref } from "vue";
import type { ShallowRef } from "vue";

import { createPositioner, type Placement, type PositionerStyle } from "./positioner.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const placements: readonly Placement[] = ["bottom", "bottom-start", "left", "top-end"];
// @ts-expect-error diagonal is not a placement.
export const invalidPlacement: Placement = "diagonal";

const offset = ref(0);
export const controller = createPositioner({
  offset,
  placement: "bottom",
  strategy: "fixed",
});

type _XIsReadonly = Expect<Equal<typeof controller.x, Readonly<ShallowRef<number>>>>;
type _StyleIsReadonly = Expect<
  Equal<typeof controller.style, Readonly<ShallowRef<PositionerStyle>>>
>;
type _AvailableWidthIsReadonly = Expect<
  Equal<typeof controller.availableWidth, Readonly<ShallowRef<number | null>>>
>;

// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.x.value = 10;
// @ts-expect-error placement must resolve to the closed union.
createPositioner({ placement: "diagonal" });
// @ts-expect-error the size strategy is a boolean toggle.
createPositioner({ size: "always" });
// @ts-expect-error safe-area awareness is a boolean toggle.
createPositioner({ safeArea: "notch" });
