/** Compile-only assertions for the public AvatarGroup contract. */

import {
  AvatarGroup,
  AvatarGroupOverflow,
  defaultAvatarGroupMessages,
  splitAvatarGroup,
  type AvatarGroupExpose,
  type AvatarGroupItemSlotState,
  type AvatarGroupMessageOverrides,
  type AvatarGroupMessages,
  type AvatarGroupOverflowExpose,
  type AvatarGroupOverflowSlotState,
  type AvatarGroupSlotState,
  type AvatarGroupState,
} from "./avatar-group.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Person {
  readonly id: string;
}

declare const group: AvatarGroupExpose<Person>;
declare const overflow: AvatarGroupOverflowExpose;
declare const itemSlot: AvatarGroupItemSlotState<Person>;
declare const overflowSlot: AvatarGroupOverflowSlotState<Person>;

type _StateIsLiteral = Expect<Equal<AvatarGroupState, "collapsed" | "empty" | "expanded">>;
type _ItemIsInferred = Expect<Equal<typeof itemSlot.item, Person>>;
type _HiddenItemsAreTyped = Expect<Equal<typeof overflowSlot.hiddenItems, readonly Person[]>>;
type _GroupHidden = Expect<Equal<typeof group.hiddenItems, readonly Person[]>>;
type _GroupElement = Expect<Equal<typeof group.element, HTMLUListElement | null>>;
type _OverflowElement = Expect<Equal<typeof overflow.element, HTMLSpanElement | null>>;
type _OverridesArePartial = Expect<
  Equal<AvatarGroupMessageOverrides, Partial<AvatarGroupMessages>>
>;
type _SlotStateHasState = Expect<Equal<AvatarGroupSlotState<Person>["state"], AvatarGroupState>>;
type _SplitInfers = Expect<
  Equal<ReturnType<typeof splitAvatarGroup<Person>>["visibleItems"], readonly Person[]>
>;
type _MessageSignature = Expect<
  Equal<typeof defaultAvatarGroupMessages.overflow, (count: number) => string>
>;

void AvatarGroup;
void AvatarGroupOverflow;

// @ts-expect-error states are a closed union.
const _unknownState: AvatarGroupState = "hidden";
// @ts-expect-error overflow messages take a count.
const _badMessage: AvatarGroupMessageOverrides = { overflow: (name: string) => name };
// @ts-expect-error exposed state is read-only.
group.overflowCount = 2;
