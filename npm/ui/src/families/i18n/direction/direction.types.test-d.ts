/** Compile-only assertions for the public Direction contract. */

import type { ComputedRef } from "vue";

import type { Direction, DirectionProviderExpose, DirectionSlotState } from "./direction.ts";
import {
  DirectionProvider,
  directionContext,
  toDirection,
  useResolvedDirection,
} from "./direction.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const expose: DirectionProviderExpose;
declare const slot: DirectionSlotState;

type _Direction = Expect<Equal<Direction, "ltr" | "rtl">>;
type _SlotDir = Expect<Equal<typeof slot.dir, Direction>>;
type _Element = Expect<Equal<typeof expose.element, HTMLElement | null>>;
type _Resolved = Expect<Equal<ReturnType<typeof useResolvedDirection>, ComputedRef<Direction>>>;
type _Normalize = Expect<Equal<ReturnType<typeof toDirection>, Direction | undefined>>;
type _Context = Expect<Equal<ReturnType<typeof directionContext.use>, ComputedRef<Direction>>>;

const props: InstanceType<typeof DirectionProvider>["$props"] = { as: "section", dir: "rtl" };
const bare: InstanceType<typeof DirectionProvider>["$props"] = { as: null };

// @ts-expect-error direction is a closed union.
const badDir: InstanceType<typeof DirectionProvider>["$props"] = { dir: "auto" };

// @ts-expect-error `as` accepts native tag names or null.
const badTag: InstanceType<typeof DirectionProvider>["$props"] = { as: "not-a-tag" };

void badDir;
void badTag;
void bare;
void props;
