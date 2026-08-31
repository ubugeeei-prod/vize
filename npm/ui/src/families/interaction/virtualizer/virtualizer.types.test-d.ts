/** Compile-only assertions for the public virtualizer contract. */

import { ref } from "vue";
import type { ShallowRef } from "vue";

import {
  createGridVirtualizer,
  createInfiniteLoader,
  createVirtualizer,
  type VirtualItem,
  type VirtualizerAlignment,
  type VirtualizerOrientation,
  type VirtualRange,
} from "./virtualizer.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const orientations: readonly VirtualizerOrientation[] = ["horizontal", "vertical"];
// @ts-expect-error diagonal is not a scroll axis.
export const invalidOrientation: VirtualizerOrientation = "diagonal";

export const alignments: readonly VirtualizerAlignment[] = ["auto", "center", "end", "start"];
// @ts-expect-error nearest is not a supported alignment.
export const invalidAlignment: VirtualizerAlignment = "nearest";

const count = ref(100);
export const controller = createVirtualizer({
  count,
  estimateItemSize: (index) => 20 + (index % 5),
  lanes: () => 2,
  stickyIndexes: () => [0, 10],
  getItemKey: (index) => `item-${index}`,
});

type _ItemsAreReadonly = Expect<
  Equal<typeof controller.virtualItems, Readonly<ShallowRef<readonly VirtualItem[]>>>
>;
type _RangeIsNullable = Expect<
  Equal<typeof controller.range, Readonly<ShallowRef<VirtualRange | null>>>
>;
type _StickyIsNullable = Expect<
  Equal<typeof controller.activeStickyIndex, Readonly<ShallowRef<number | null>>>
>;
type _KeysAreUnions = Expect<Equal<VirtualItem["key"], string | number>>;

// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.totalSize.value = 0;
// @ts-expect-error virtual items are immutable snapshots.
controller.virtualItems.value[0].start = 10;
// @ts-expect-error count is required.
createVirtualizer({ itemSize: 20 });
// @ts-expect-error item sizes are pixels or per-index resolvers.
createVirtualizer({ count: 10, itemSize: "20px" });

export const grid = createGridVirtualizer({
  rowCount: 10,
  columnCount: 10,
  estimateRowSize: 20,
  columnSize: (index) => 40 + index,
});
// @ts-expect-error cells expose readonly geometry.
grid.virtualCells.value[0].top = 1;

export const loader = createInfiniteLoader({
  range: controller.range,
  count,
  canLoadForward: true,
  onLoadForward({ signal }) {
    void signal.aborted;
  },
});
// @ts-expect-error sideways is not a loading direction.
loader.cancel("sideways");
// @ts-expect-error consumers cannot mutate loading state.
loader.forwardStatus.value = "loading";
