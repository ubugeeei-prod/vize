/** Compile-only assertions for the public NavigationMenu contract. */

import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuIndicator,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuRoot,
  NavigationMenuTrigger,
  NavigationMenuViewport,
  type NavigationMenuChangeReason,
  type NavigationMenuItemSlotState,
  type NavigationMenuMotion,
  type NavigationMenuOpenState,
  type NavigationMenuOrientation,
  type NavigationMenuRootExpose,
  type NavigationMenuValue,
} from "./navigation-menu.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _ValueIsNullable = Expect<Equal<NavigationMenuValue, string | null>>;
type _OrientationIsLiteral = Expect<Equal<NavigationMenuOrientation, "horizontal" | "vertical">>;
type _OpenStateIsLiteral = Expect<Equal<NavigationMenuOpenState, "closed" | "open">>;
type _MotionIsLiteral = Expect<
  Equal<NavigationMenuMotion, "from-end" | "from-start" | "to-end" | "to-start">
>;
type _ReasonIsLiteral = Expect<
  Equal<
    NavigationMenuChangeReason,
    "dismiss" | "keyboard" | "link" | "pointer" | "programmatic" | "toggle"
  >
>;
type _ItemSlotValue = Expect<Equal<NavigationMenuItemSlotState["value"], string>>;
type _ExposeOpen = Expect<Equal<NavigationMenuRootExpose["open"], (value: string) => boolean>>;

const rootProps: InstanceType<typeof NavigationMenuRoot>["$props"] = {
  ariaLabel: "Main",
  closeDelay: 100,
  defaultValue: null,
  delayDuration: 150,
  dir: "rtl",
  modelValue: "docs",
  orientation: "vertical",
  skipDelayDuration: 250,
  onChange: (
    value: NavigationMenuValue,
    previous: NavigationMenuValue,
    reason: NavigationMenuChangeReason,
  ) => [value, previous, reason],
  "onUpdate:modelValue": (value: NavigationMenuValue) => value,
};
const itemProps: InstanceType<typeof NavigationMenuItem>["$props"] = { value: "docs" };
const triggerProps: InstanceType<typeof NavigationMenuTrigger>["$props"] = { disabled: true };
const contentProps: InstanceType<typeof NavigationMenuContent>["$props"] = { forceMount: true };
const linkProps: InstanceType<typeof NavigationMenuLink>["$props"] = {
  active: true,
  href: "/docs",
  onSelect: (event: MouseEvent) => event,
};

// @ts-expect-error items require a string value.
const badItem: InstanceType<typeof NavigationMenuItem>["$props"] = {};

// @ts-expect-error hrefs are strings.
const badLink: InstanceType<typeof NavigationMenuLink>["$props"] = { href: 1 };

void NavigationMenu;
void NavigationMenuIndicator;
void NavigationMenuList;
void NavigationMenuViewport;
void badItem;
void badLink;
void contentProps;
void itemProps;
void linkProps;
void rootProps;
void triggerProps;
