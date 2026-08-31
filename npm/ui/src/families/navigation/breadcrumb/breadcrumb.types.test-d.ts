/** Compile-only assertions for the public Breadcrumb composition contract. */

import type { Component, ComponentPublicInstance } from "vue";

import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbRoot,
  BreadcrumbSeparator,
} from "./breadcrumb.ts";
import type {
  BreadcrumbCurrent,
  BreadcrumbItemExpose,
  BreadcrumbLinkExpose,
  BreadcrumbLinkSlotState,
  BreadcrumbRootExpose,
  BreadcrumbSeparatorExpose,
} from "./breadcrumb.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const root: BreadcrumbRootExpose;
declare const item: BreadcrumbItemExpose;
declare const link: BreadcrumbLinkExpose;
declare const separator: BreadcrumbSeparatorExpose;

type _CurrentIsStrict = Expect<
  Equal<BreadcrumbCurrent, "date" | "location" | "page" | "step" | "time" | true>
>;
type _RootLabelIsString = Expect<Equal<typeof root.label, string>>;
type _ItemCurrentIsBoolean = Expect<Equal<typeof item.current, boolean>>;
type _LinkCurrentIsBoolean = Expect<Equal<typeof link.current, boolean>>;
type _LinkAriaCurrentIsOptional = Expect<
  Equal<typeof link.ariaCurrent, "date" | "location" | "page" | "step" | "time" | undefined>
>;
type _SeparatorDecorativeIsLiteral = Expect<Equal<typeof separator.decorative, true>>;
type _RootElementIsRenderable = Expect<
  Equal<typeof root.element, Element | ComponentPublicInstance | null>
>;

const rootProps: InstanceType<typeof Breadcrumb>["$props"] = {
  as: "nav",
  label: "Workspace path",
};
const aliasProps: InstanceType<typeof BreadcrumbRoot>["$props"] = rootProps;
const listProps: InstanceType<typeof BreadcrumbList>["$props"] = { as: "ol" };
const itemProps: InstanceType<typeof BreadcrumbItem>["$props"] = { current: true };
const linkProps: InstanceType<typeof BreadcrumbLink>["$props"] = {
  as: componentTarget,
  current: "location",
  href: "/settings",
};
const separatorProps: InstanceType<typeof BreadcrumbSeparator>["$props"] = { as: "span" };
const slotState: BreadcrumbLinkSlotState = {
  ariaCurrent: "step",
  current: true,
};

// @ts-expect-error current state must use the strict aria-current route literals.
const invalidCurrent: BreadcrumbCurrent = "route";

// @ts-expect-error item current is boolean-only.
const invalidItemProps: InstanceType<typeof BreadcrumbItem>["$props"] = { current: "page" };

// @ts-expect-error link current rejects arbitrary strings.
const invalidLinkProps: InstanceType<typeof BreadcrumbLink>["$props"] = { current: "active" };

void aliasProps;
void invalidCurrent;
void invalidItemProps;
void invalidLinkProps;
void itemProps;
void linkProps;
void listProps;
void rootProps;
void separatorProps;
void slotState;
