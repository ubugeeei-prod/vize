/** Compile-only assertions for the public Callout contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { Callout } from "./callout.ts";
import type {
  CalloutAriaState,
  CalloutDensity,
  CalloutElement,
  CalloutExpose,
  CalloutLive,
  CalloutProps,
  CalloutRole,
  CalloutSlotState,
  CalloutSlots,
  CalloutState,
  CalloutTone,
} from "./callout.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: CalloutExpose;
declare const slots: CalloutSlots;

type _RoleIsLiteral = Expect<Equal<CalloutRole, "alert" | "note" | "status">>;
type _AriaStateIsLiteral = Expect<Equal<CalloutAriaState, "decorative" | CalloutRole>>;
type _ToneIsLiteral = Expect<
  Equal<CalloutTone, "accent" | "danger" | "info" | "neutral" | "success" | "warning">
>;
type _DensityIsLiteral = Expect<Equal<CalloutDensity, "compact" | "comfortable">>;
type _StateIsLiteral = Expect<Equal<CalloutState, "closed" | "open">>;
type _LiveIsLiteral = Expect<Equal<CalloutLive, "assertive" | "polite">>;
type _ElementIsRenderable = Expect<Equal<CalloutElement, Element | ComponentPublicInstance>>;
type _PropsFeedComponentProps = Expect<
  CalloutProps extends InstanceType<typeof Callout>["$props"] ? true : false
>;
type _PropsKeysAreClosed = Expect<
  Equal<
    keyof CalloutProps,
    | "ariaDescribedby"
    | "ariaHidden"
    | "ariaLabel"
    | "ariaLabelledby"
    | "as"
    | "atomic"
    | "density"
    | "descriptionId"
    | "iconAriaHidden"
    | "id"
    | "open"
    | "role"
    | "titleId"
    | "tone"
  >
>;
type _ExposeStateIsLiteral = Expect<Equal<typeof exposed.state, CalloutState>>;
type _ExposeRoleIsLiteral = Expect<Equal<typeof exposed.role, CalloutRole>>;
type _ExposeAriaStateIsLiteral = Expect<Equal<typeof exposed.ariaState, CalloutAriaState>>;
type _ExposeLiveIsOptional = Expect<Equal<typeof exposed.live, CalloutLive | undefined>>;
type _ExposeToneIsLiteral = Expect<Equal<typeof exposed.tone, CalloutTone>>;
type _ExposeDensityIsLiteral = Expect<Equal<typeof exposed.density, CalloutDensity>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    CalloutSlotState,
    {
      readonly open: boolean;
      readonly state: CalloutState;
      readonly role: CalloutRole;
      readonly ariaState: CalloutAriaState;
      readonly live: CalloutLive | undefined;
      readonly atomic: boolean;
      readonly tone: CalloutTone;
      readonly density: CalloutDensity;
      readonly titleId: string | undefined;
      readonly descriptionId: string | undefined;
      readonly ariaLabelledby: string | undefined;
      readonly ariaDescribedby: string | undefined;
      readonly hasIcon: boolean;
      readonly hasTitle: boolean;
      readonly hasDescription: boolean;
      readonly hasActions: boolean;
    }
  >
>;

const exposedElement: CalloutElement | null = exposed.element;
const publicProps: CalloutProps = {
  ariaDescribedby: "deploy-help",
  ariaLabelledby: "deploy-title",
  density: "compact",
  role: "alert",
  tone: "danger",
};
const customHost: InstanceType<typeof Callout>["$props"] = {
  ariaHidden: false,
  ariaLabel: "Upload queued",
  as: componentTarget,
  atomic: false,
  descriptionId: "upload-description",
  iconAriaHidden: false,
  id: "upload-callout",
  open: true,
  role: "status",
  titleId: "upload-title",
  tone: "info",
};
const slotState: CalloutSlotState = {
  ariaDescribedby: "upload-description",
  ariaLabelledby: "upload-title",
  ariaState: "status",
  atomic: true,
  density: "comfortable",
  descriptionId: "upload-description",
  hasActions: true,
  hasDescription: true,
  hasIcon: true,
  hasTitle: true,
  live: "polite",
  open: true,
  role: "status",
  state: "open",
  titleId: "upload-title",
  tone: "success",
};

slots.default(slotState);
slots.icon(slotState);
slots.title(slotState);
slots.description(slotState);
slots.actions(slotState);

// @ts-expect-error Callout roles are intentionally narrow.
const invalidRole: CalloutRole = "region";

// @ts-expect-error Callout tones use a closed token contract.
const invalidTone: CalloutTone = "brand";

// @ts-expect-error Callout densities are strict consumer styling tokens.
const invalidDensity: CalloutDensity = "dense";

// @ts-expect-error component props require boolean open state.
const badOpen: InstanceType<typeof Callout>["$props"] = { open: "true" };

// @ts-expect-error component props require a supported live-region role.
const badRoleProp: InstanceType<typeof Callout>["$props"] = { role: "dialog" };

// @ts-expect-error slot state must include the resolved accessibility hooks.
const badSlotState: CalloutSlotState = { state: "open", role: "note" };

void Callout;
void badOpen;
void badRoleProp;
void badSlotState;
void customHost;
void exposedElement;
void invalidDensity;
void invalidRole;
void invalidTone;
void publicProps;
void slotState;
