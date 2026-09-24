/** Compile-only assertions for the public TOC contract. */

import {
  Toc,
  TocItem,
  TocLink,
  TocList,
  TocRoot,
  collectTocEntries,
  type TocEntry,
  type TocLinkSlotState,
  type TocRootExpose,
  type TocScrollBehavior,
  type TocSlotState,
  type TocState,
} from "./toc.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _BehaviorIsLiteral = Expect<Equal<TocScrollBehavior, "auto" | "instant" | "smooth">>;
type _StateIsLiteral = Expect<Equal<TocState, "active" | "idle">>;
type _SlotActiveIsNullable = Expect<Equal<TocSlotState["activeId"], string | null>>;
type _LinkSlotActive = Expect<Equal<TocLinkSlotState["active"], boolean>>;
type _CollectReturns = Expect<Equal<ReturnType<typeof collectTocEntries>, readonly TocEntry[]>>;
type _ExposeScroll = Expect<Equal<TocRootExpose["scrollTo"], (id: string) => boolean>>;

const rootProps: InstanceType<typeof TocRoot>["$props"] = {
  activeId: null,
  ariaLabel: "On this page",
  defaultActiveId: "intro",
  offset: 64,
  root: null,
  scrollBehavior: "smooth",
  track: true,
  updateHash: false,
  "onUpdate:activeId": (id: string | null) => id,
  onNavigate: (id: string, event: MouseEvent) => [id, event],
};
const linkProps: InstanceType<typeof TocLink>["$props"] = { targetId: "intro" };
const listProps: InstanceType<typeof TocList>["$props"] = { level: 2 };
const itemProps: InstanceType<typeof TocItem>["$props"] = { targetId: "intro" };

const badRoot: InstanceType<typeof TocRoot>["$props"] = {
  // @ts-expect-error scroll behavior is a closed union.
  scrollBehavior: "slow",
};

// @ts-expect-error links require a target id.
const badLink: InstanceType<typeof TocLink>["$props"] = {};

void Toc;
void badLink;
void badRoot;
void itemProps;
void linkProps;
void listProps;
void rootProps;
