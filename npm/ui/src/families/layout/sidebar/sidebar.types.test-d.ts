/** Compile-only assertions for the public Sidebar contract. */

import type {
  SidebarCollapsible,
  SidebarGroupExpose,
  SidebarProviderExpose,
  SidebarSide,
  SidebarSlotState,
  SidebarState,
  SidebarStorage,
  SidebarToggleExpose,
  SidebarVariant,
} from "./sidebar.ts";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarInset,
  SidebarProvider,
  SidebarRail,
  SidebarRoot,
  SidebarTrigger,
} from "./sidebar.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const provider: SidebarProviderExpose;
declare const toggle: SidebarToggleExpose;
declare const group: SidebarGroupExpose;
declare const slot: SidebarSlotState;

type _State = Expect<Equal<SidebarState, "collapsed" | "expanded">>;
type _Collapsible = Expect<Equal<SidebarCollapsible, "icon" | "none" | "offcanvas">>;
type _Side = Expect<Equal<SidebarSide, "left" | "right">>;
type _Variant = Expect<Equal<SidebarVariant, "floating" | "inset" | "sidebar">>;
type _Storage = Expect<Equal<ReturnType<SidebarStorage["get"]>, string | null>>;
type _Mobile = Expect<Equal<typeof slot.isMobile, boolean>>;
type _ToggleElement = Expect<Equal<typeof toggle.element, HTMLButtonElement | null>>;
type _LabelId = Expect<Equal<typeof group.labelId, string>>;

provider.toggle();
provider.setOpen(false, new Event("click"));
provider.setOpenMobile(true);
toggle.focus();

const providerProps: InstanceType<typeof SidebarProvider>["$props"] = {
  collapsible: "icon",
  defaultOpen: false,
  keyboardShortcut: null,
  mobileQuery: "(max-width: 600px)",
  side: "right",
  storage: { get: () => null, set: () => undefined },
  variant: "inset",
  width: "20rem",
  "onUpdate:open": (value: boolean) => value,
};
const rootProps: InstanceType<typeof SidebarRoot>["$props"] = { ariaLabel: "Primary" };
const triggerProps: InstanceType<typeof SidebarTrigger>["$props"] = { ariaLabel: "Menu" };
const insetProps: InstanceType<typeof SidebarInset>["$props"] = { as: "div" };

// @ts-expect-error collapse mode is a closed union.
const badCollapsible: InstanceType<typeof SidebarProvider>["$props"] = { collapsible: "rail" };

// @ts-expect-error side is left or right.
const badSide: InstanceType<typeof SidebarProvider>["$props"] = { side: "top" };

// @ts-expect-error storage must provide get and set.
const badStorage: InstanceType<typeof SidebarProvider>["$props"] = { storage: { get: () => null } };

void Sidebar;
void SidebarContent;
void SidebarGroup;
void SidebarGroupLabel;
void SidebarRail;
void badCollapsible;
void badSide;
void badStorage;
void insetProps;
void providerProps;
void rootProps;
void triggerProps;
