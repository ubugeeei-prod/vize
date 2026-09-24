/** Compile-only assertions for the public GridList contract. */

import type {
  GridListExpose,
  GridListItemSlotProps,
  GridListLayout,
  GridListReorderEvent,
  GridListSelectionMode,
} from "./grid-list.ts";
import { GridList } from "./grid-list.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Photo {
  readonly id: number;
  readonly title: string;
}

declare const slot: GridListItemSlotProps<Photo>;
declare const reorder: GridListReorderEvent<Photo>;
declare const expose: GridListExpose;

type _Mode = Expect<Equal<GridListSelectionMode, "multiple" | "none" | "single">>;
type _Layout = Expect<Equal<GridListLayout, "grid" | "list">>;
type _SlotItem = Expect<Equal<typeof slot.item, Photo>>;
type _ReorderItems = Expect<Equal<typeof reorder.items, readonly Photo[]>>;
type _Element = Expect<Equal<typeof expose.element, HTMLDivElement | null>>;

type Props<Item> = Parameters<typeof GridList<Item>>[0];
const props: Props<Photo> = {
  items: [{ id: 1, title: "Alps" }],
  getKey: (photo) => String(photo.id),
  getTextValue: (photo) => photo.title,
  layout: "grid",
  columns: 3,
  reorderable: true,
  onAction: (photo, key) => `${photo.title}${key}`,
  onReorder: (event) => event.toIndex,
};

// @ts-expect-error getKey receives the item type.
const badKey: Props<Photo> = { items: [], getKey: (photo: string) => photo };
// @ts-expect-error layout is a closed union.
const badLayout: Props<Photo> = { items: [], layout: "masonry" };

void [badKey, badLayout, props];
