/** Compile-only assertions for the masonry layout. */

import { computeMasonryLayout, visibleMasonryItems } from "./masonry.ts";
import type { MasonryItemSlotState, MasonryLayout, MasonryPlacement } from "./masonry.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const layout = computeMasonryLayout([1, 2, 3], 2, 8);
type _LayoutIsTyped = Expect<Equal<typeof layout, MasonryLayout>>;
type _PlacementsAreReadonly = Expect<Equal<typeof layout.placements, readonly MasonryPlacement[]>>;
visibleMasonryItems(layout, 0, 100, 50) satisfies number[];
// @ts-expect-error heights are numbers.
computeMasonryLayout(["1"], 2);
declare const placement: MasonryPlacement;
// @ts-expect-error placements are readonly.
placement.top = 1;

interface Photo {
  readonly src: string;
}
type PhotoSlot = MasonryItemSlotState<Photo>;
type _SlotCarriesTheItemType = Expect<Equal<PhotoSlot["item"], Photo>>;
