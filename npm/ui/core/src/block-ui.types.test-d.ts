/** Compile-only assertions for the public BlockUI contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { BlockUI } from "./block-ui.ts";
import type {
  BlockUIAnnouncement,
  BlockUIElement,
  BlockUIExpose,
  BlockUIInteraction,
  BlockUIReason,
  BlockUISlotState,
  BlockUIState,
} from "./block-ui.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: BlockUIExpose;

type _StateIsLiteral = Expect<Equal<BlockUIState, "blocked" | "idle">>;
type _ReasonIsLiteral = Expect<
  Equal<BlockUIReason, "loading" | "saving" | "syncing" | "stale" | "offline">
>;
type _InteractionIsLiteral = Expect<Equal<BlockUIInteraction, "none" | "inert">>;
type _AnnouncementIsLiteral = Expect<Equal<BlockUIAnnouncement, "off" | "polite" | "assertive">>;
type _ElementIsRenderable = Expect<Equal<BlockUIElement, Element | ComponentPublicInstance>>;
type _ExposeBlockedIsBoolean = Expect<Equal<typeof exposed.blocked, boolean>>;
type _ExposeStateIsLiteral = Expect<Equal<typeof exposed.state, BlockUIState>>;
type _ExposeReasonIsLiteral = Expect<Equal<typeof exposed.reason, BlockUIReason>>;
type _ExposeInteractionIsLiteral = Expect<Equal<typeof exposed.interaction, BlockUIInteraction>>;
type _ExposeAnnouncementIsLiteral = Expect<Equal<typeof exposed.announcement, BlockUIAnnouncement>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    BlockUISlotState,
    {
      readonly blocked: boolean;
      readonly state: BlockUIState;
      readonly reason: BlockUIReason;
      readonly interaction: BlockUIInteraction;
      readonly announcement: BlockUIAnnouncement;
    }
  >
>;

const exposedElement: BlockUIElement | null = exposed.element;
const customHost: InstanceType<typeof BlockUI>["$props"] = {
  announce: "polite",
  as: componentTarget,
  blocked: true,
  interaction: "inert",
  label: "Saving profile",
  reason: "saving",
};
const nativeHost: InstanceType<typeof BlockUI>["$props"] = {
  announce: "assertive",
  as: "article",
  blocked: false,
  interaction: "none",
  label: "Offline",
  reason: "offline",
};
const slotState: BlockUISlotState = {
  announcement: "off",
  blocked: false,
  interaction: "none",
  reason: "loading",
  state: "idle",
};

// @ts-expect-error BlockUI state is derived, not an arbitrary token.
const invalidState: BlockUIState = "pending";

// @ts-expect-error BlockUI reasons are strict status tokens.
const invalidReason: BlockUIReason = "fetching";

// @ts-expect-error BlockUI interaction has a closed policy contract.
const invalidInteraction: BlockUIInteraction = "disabled";

// @ts-expect-error BlockUI announcements use explicit live-region policies.
const invalidAnnouncement: BlockUIAnnouncement = "loud";

// @ts-expect-error reason must use the literal BlockUIReason contract.
const badReasonProp: InstanceType<typeof BlockUI>["$props"] = { reason: "fetching" };

// @ts-expect-error interaction must use the literal BlockUIInteraction contract.
const badInteractionProp: InstanceType<typeof BlockUI>["$props"] = { interaction: "disabled" };

// @ts-expect-error announce must use the literal BlockUIAnnouncement contract.
const badAnnouncementProp: InstanceType<typeof BlockUI>["$props"] = { announce: "loud" };

void BlockUI;
void badAnnouncementProp;
void badInteractionProp;
void badReasonProp;
void customHost;
void exposedElement;
void invalidAnnouncement;
void invalidInteraction;
void invalidReason;
void invalidState;
void nativeHost;
void slotState;
