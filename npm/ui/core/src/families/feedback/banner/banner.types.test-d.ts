/** Compile-only assertions for the public Banner contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { Banner } from "./banner.ts";
import type {
  BannerAriaInput,
  BannerAriaState,
  BannerElement,
  BannerEmits,
  BannerExpose,
  BannerLive,
  BannerProps,
  BannerRole,
  BannerSlots,
  BannerSlotState,
  BannerState,
  BannerTone,
  NormalizedBannerAria,
} from "./banner.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: BannerExpose;
declare const slotState: BannerSlotState;

type _RoleIsLiteral = Expect<Equal<BannerRole, "alert" | "region" | "status">>;
type _ToneIsLiteral = Expect<
  Equal<BannerTone, "accent" | "danger" | "info" | "neutral" | "success" | "warning">
>;
type _StateIsLiteral = Expect<Equal<BannerState, "closed" | "open">>;
type _LiveIsLiteral = Expect<Equal<BannerLive, "assertive" | "off" | "polite">>;
type _AriaStateIsLiteral = Expect<Equal<BannerAriaState, "live" | "named" | "unnamed">>;
type _ElementIsRenderable = Expect<Equal<BannerElement, Element | ComponentPublicInstance>>;
type _UpdatePayloadIsBoolean = Expect<Equal<BannerEmits["update:open"], [open: boolean]>>;
type _DismissPayloadIsNativeMouseEvent = Expect<
  Equal<BannerEmits["dismiss"], [nativeEvent: MouseEvent]>
>;
type _ExposeStateIsLiteral = Expect<Equal<typeof exposed.state, BannerState>>;
type _ExposeRoleIsLiteral = Expect<Equal<typeof exposed.role, BannerRole>>;
type _ExposeToneIsLiteral = Expect<Equal<typeof exposed.tone, BannerTone>>;
type _ExposeLiveIsLiteral = Expect<Equal<typeof exposed.live, BannerLive>>;
type _ExposeNamedIsBoolean = Expect<Equal<typeof exposed.named, boolean>>;
type _ExposeAriaStateIsLiteral = Expect<Equal<typeof exposed.ariaState, BannerAriaState>>;
type _ExposeTitleIdIsString = Expect<Equal<typeof exposed.titleId, string>>;
type _ExposeDescriptionIdIsString = Expect<Equal<typeof exposed.descriptionId, string>>;
type _ExposeDismissibleIsBoolean = Expect<Equal<typeof exposed.dismissible, boolean>>;
type _DefaultSlotReceivesState = Expect<
  Equal<Parameters<NonNullable<BannerSlots["default"]>>[0], BannerSlotState>
>;
type _TitleSlotReceivesState = Expect<
  Equal<Parameters<NonNullable<BannerSlots["title"]>>[0], BannerSlotState>
>;
type _PropsFeedComponentProps = Expect<
  BannerProps extends InstanceType<typeof Banner>["$props"] ? true : false
>;
type _PropsKeysAreClosed = Expect<
  Equal<
    keyof BannerProps,
    | "ariaDescribedby"
    | "ariaLabel"
    | "ariaLabelledby"
    | "as"
    | "atomic"
    | "description"
    | "dismissLabel"
    | "dismissible"
    | "id"
    | "open"
    | "role"
    | "title"
    | "tone"
  >
>;
type _SlotStateIsLiteral = Expect<
  Equal<
    BannerSlotState,
    {
      readonly state: BannerState;
      readonly role: BannerRole;
      readonly tone: BannerTone;
      readonly live: BannerLive;
      readonly named: boolean;
      readonly ariaState: BannerAriaState;
      readonly titleId: string;
      readonly descriptionId: string;
      readonly ariaLabelledby: string | undefined;
      readonly ariaDescribedby: string | undefined;
      readonly dismissible: boolean;
    }
  >
>;
type _NormalizedAriaIsClosed = Expect<
  Equal<
    NormalizedBannerAria,
    {
      readonly role: BannerRole | undefined;
      readonly ariaState: BannerAriaState;
      readonly live: BannerLive;
      readonly named: boolean;
      readonly ariaLabel: string | undefined;
      readonly ariaLabelledby: string | undefined;
      readonly ariaDescribedby: string | undefined;
      readonly ariaLive: "assertive" | "polite" | undefined;
      readonly ariaAtomic: "false" | "true" | undefined;
    }
  >
>;

const exposedElement: BannerElement | null = exposed.element;
const titleProps: BannerProps = {
  description: "Scheduled outage",
  dismissible: true,
  role: "region",
  title: "Maintenance",
  tone: "warning",
};
const labelProps: BannerProps = {
  ariaLabel: "Payment failed",
  as: componentTarget,
  role: "alert",
  tone: "danger",
};
const labelledbyProps: BannerProps = {
  ariaLabelledby: "external-label",
  atomic: false,
  open: false,
  role: "status",
};
const unnamedStatusProps: BannerProps = {
  role: "status",
  tone: "info",
};
const unnamedAlertProps: BannerProps = {
  dismissible: true,
  role: "alert",
  tone: "danger",
};
const ariaInput: BannerAriaInput = {
  atomic: true,
  descriptionId: "banner-description",
  hasDescription: true,
  hasTitle: true,
  role: "region",
  titleId: "banner-title",
};

exposed.focus();
exposed.dismiss();
exposed.dismiss(new MouseEvent("click"));

// @ts-expect-error default region banners need a title, aria-label, or aria-labelledby name.
const unnamedDefaultProps: BannerProps = { tone: "info" };

// @ts-expect-error region banners need a title, aria-label, or aria-labelledby name.
const unnamedProps: BannerProps = { role: "region" };

// @ts-expect-error exported props preserve the closed role token contract.
const invalidRole: BannerRole = "dialog";

// @ts-expect-error Banner tones use the shared literal tone contract.
const invalidTone: BannerTone = "brand";

// @ts-expect-error Banner state is derived from controlled open visibility.
const invalidState: BannerState = "hidden";

// @ts-expect-error dismiss emits a native mouse event payload.
const invalidDismissPayload: BannerEmits["dismiss"] = [];

// @ts-expect-error slot state exposes required deterministic ids.
const badSlotState: BannerSlotState = { state: "open", role: "region", tone: "neutral" };

// @ts-expect-error component props require boolean controlled visibility.
const badOpen: InstanceType<typeof Banner>["$props"] = { open: "false", title: "Bad" };

void Banner;
void ariaInput;
void badOpen;
void badSlotState;
void exposedElement;
void invalidDismissPayload;
void invalidRole;
void invalidState;
void invalidTone;
void labelledbyProps;
void labelProps;
void slotState;
void titleProps;
void unnamedAlertProps;
void unnamedDefaultProps;
void unnamedProps;
void unnamedStatusProps;
