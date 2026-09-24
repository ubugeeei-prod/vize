/** Compile-only assertions for the public Accordion contract. */

import type {
  AccordionContentExpose,
  AccordionContentRole,
  AccordionHeadingLevel,
  AccordionItemExpose,
  AccordionItemSlotState,
  AccordionModelValue,
  AccordionRootExpose,
  AccordionSlotState,
  AccordionTriggerExpose,
  AccordionType,
  AccordionValue,
} from "./accordion.ts";
import {
  Accordion,
  AccordionContent,
  AccordionHeader,
  AccordionItem,
  AccordionRoot,
  AccordionTrigger,
  accordionOpenValues,
  toAccordionModel,
} from "./accordion.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _ValueIsSerializable = Expect<Equal<AccordionValue, string | number>>;
type _TypeIsLiteral = Expect<Equal<AccordionType, "multiple" | "single">>;
type _RoleIsLiteral = Expect<Equal<AccordionContentRole, "group" | "region">>;
type _HeadingLevels = Expect<Equal<AccordionHeadingLevel, 1 | 2 | 3 | 4 | 5 | 6>>;
type _SingleModel = Expect<Equal<AccordionModelValue<"a" | "b", "single">, "a" | "b" | null>>;
type _MultipleModel = Expect<Equal<AccordionModelValue<number, "multiple">, readonly number[]>>;

declare const single: AccordionRootExpose<"shipping" | "returns", "single">;
declare const multiple: AccordionRootExpose<number, "multiple">;
declare const item: AccordionItemExpose;
declare const trigger: AccordionTriggerExpose;
declare const content: AccordionContentExpose;
declare const slot: AccordionSlotState<string, "multiple">;
declare const itemSlot: AccordionItemSlotState;

type _SingleValue = Expect<Equal<typeof single.value, "shipping" | "returns" | null>>;
type _SingleOpenValues = Expect<
  Equal<typeof single.openValues, readonly ("shipping" | "returns")[]>
>;
type _MultipleValue = Expect<Equal<typeof multiple.value, readonly number[]>>;
type _SlotValue = Expect<Equal<typeof slot.value, readonly string[]>>;
type _SlotType = Expect<Equal<typeof slot.type, "multiple">>;
type _ItemSlotState = Expect<Equal<typeof itemSlot.state, "closed" | "open">>;
type _TriggerElement = Expect<Equal<typeof trigger.element, HTMLButtonElement | null>>;
type _ContentElement = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _ItemValue = Expect<Equal<typeof item.value, AccordionValue>>;

single.expand("shipping");
single.setValue(null);
multiple.setValue([1, 2]);
multiple.toggle(3);
multiple.focus(1, { preventScroll: true });

type _Model = Expect<Equal<ReturnType<typeof toAccordionModel<"a", "multiple">>, readonly "a"[]>>;
type _OpenValues = Expect<Equal<ReturnType<typeof accordionOpenValues<3>>, readonly 3[]>>;

// Generic SFC inference: `type` selects the model shape and the emitted payload.
AccordionRoot({
  type: "multiple",
  modelValue: ["shipping", "returns"],
  "onUpdate:modelValue": (value) => {
    type _Emitted = Expect<Equal<typeof value, readonly ("shipping" | "returns")[]>>;
  },
});
AccordionRoot({
  type: "single",
  defaultValue: 2,
  collapsible: true,
  headingLevel: 2,
  "onUpdate:modelValue": (value) => {
    type _Emitted = Expect<Equal<typeof value, 2 | null>>;
  },
});

// @ts-expect-error single accordions do not accept array models.
AccordionRoot({ type: "single", modelValue: ["a"] });

// @ts-expect-error multiple accordions do not accept scalar models.
AccordionRoot({ type: "multiple", modelValue: "a" });

// @ts-expect-error type is required and closed.
AccordionRoot({ type: "many" });

// @ts-expect-error item values are strings or numbers.
single.expand({ id: "shipping" });

// @ts-expect-error multiple roots expose list models.
const scalar: string = multiple.value;

const itemProps: InstanceType<typeof AccordionItem>["$props"] = { value: 3, disabled: true };
const headerProps: InstanceType<typeof AccordionHeader>["$props"] = { level: 2 };
const triggerProps: InstanceType<typeof AccordionTrigger>["$props"] = { ariaLabel: "Shipping" };
const contentProps: InstanceType<typeof AccordionContent>["$props"] = {
  role: null,
  hiddenUntilFound: true,
};

// @ts-expect-error heading levels are 1 through 6.
const badHeader: InstanceType<typeof AccordionHeader>["$props"] = { level: 7 };

// @ts-expect-error content role is intentionally limited.
const badContent: InstanceType<typeof AccordionContent>["$props"] = { role: "navigation" };

// @ts-expect-error item values must be serializable keys.
const badItem: InstanceType<typeof AccordionItem>["$props"] = { value: true };

void Accordion;
void badContent;
void badHeader;
void badItem;
void contentProps;
void headerProps;
void itemProps;
void scalar;
void triggerProps;
