/** Compile-only assertions for the public StatusLight contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { StatusLight } from "./status-light.ts";
import type {
  StatusLightAriaState,
  StatusLightElement,
  StatusLightExpose,
  StatusLightProps,
  StatusLightRole,
  StatusLightSize,
  StatusLightSlotState,
  StatusLightState,
  StatusLightTone,
} from "./status-light.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: StatusLightExpose;

type _StateIsLiteral = Expect<
  Equal<StatusLightState, "away" | "busy" | "offline" | "online" | "unknown">
>;
type _ToneIsLiteral = Expect<
  Equal<StatusLightTone, "accent" | "danger" | "info" | "neutral" | "success" | "warning">
>;
type _SizeIsLiteral = Expect<Equal<StatusLightSize, "sm" | "md" | "lg">>;
type _RoleIsLiteral = Expect<Equal<StatusLightRole, "img" | "status">>;
type _AriaStateIsLiteral = Expect<Equal<StatusLightAriaState, "decorative" | StatusLightRole>>;
type _ElementIsRenderable = Expect<Equal<StatusLightElement, Element | ComponentPublicInstance>>;
type _ExposeStateIsLiteral = Expect<Equal<typeof exposed.state, StatusLightState>>;
type _ExposeToneIsLiteral = Expect<Equal<typeof exposed.tone, StatusLightTone>>;
type _ExposeSizeIsLiteral = Expect<Equal<typeof exposed.size, StatusLightSize>>;
type _ExposeAriaStateIsLiteral = Expect<Equal<typeof exposed.ariaState, StatusLightAriaState>>;
type _ExposeDecorativeIsBoolean = Expect<Equal<typeof exposed.decorative, boolean>>;
type _PropsFeedComponentProps = Expect<
  StatusLightProps extends InstanceType<typeof StatusLight>["$props"] ? true : false
>;
type _PropsKeysAreClosed = Expect<
  Equal<
    keyof StatusLightProps,
    | "ariaDescribedby"
    | "ariaHidden"
    | "ariaLabel"
    | "ariaLabelledby"
    | "as"
    | "atomic"
    | "role"
    | "size"
    | "state"
    | "tone"
  >
>;
type _SlotStateIsLiteral = Expect<
  Equal<
    StatusLightSlotState,
    {
      readonly state: StatusLightState;
      readonly tone: StatusLightTone;
      readonly size: StatusLightSize;
      readonly ariaState: StatusLightAriaState;
      readonly decorative: boolean;
    }
  >
>;

const exposedElement: StatusLightElement | null = exposed.element;
const publicProps: StatusLightProps = {
  ariaLabelledby: "service-label",
  role: "img",
  state: "away",
  tone: "info",
};
const customHost: InstanceType<typeof StatusLight>["$props"] = {
  ariaDescribedby: "service-help",
  ariaHidden: false,
  ariaLabel: "Service online",
  as: componentTarget,
  atomic: false,
  role: "status",
  size: "sm",
  state: "online",
  tone: "success",
};
const slotState: StatusLightSlotState = {
  ariaState: "img",
  decorative: false,
  size: "md",
  state: "busy",
  tone: "warning",
};

// @ts-expect-error StatusLight has a closed state token contract.
const invalidState: StatusLightState = "pending";

// @ts-expect-error exported props preserve the closed state token contract.
const invalidProps: StatusLightProps = { state: "pending" };

// @ts-expect-error StatusLight tones use the shared literal tone contract.
const invalidTone: StatusLightTone = "brand";

// @ts-expect-error StatusLight sizes are strict consumer styling tokens.
const invalidSize: StatusLightSize = "xl";

// @ts-expect-error non-decorative accessibility roles are intentionally narrow.
const invalidRole: StatusLightRole = "alert";

// @ts-expect-error component props require boolean atomic policy.
const badAtomic: InstanceType<typeof StatusLight>["$props"] = { atomic: "true" };

// @ts-expect-error slot state exposes a required decorative boolean.
const badSlotState: StatusLightSlotState = { state: "online", tone: "success", size: "sm" };

void StatusLight;
void badAtomic;
void badSlotState;
void customHost;
void exposedElement;
void invalidProps;
void invalidRole;
void invalidSize;
void invalidState;
void invalidTone;
void publicProps;
void slotState;
