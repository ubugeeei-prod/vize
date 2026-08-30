/** Compile-only assertions for the public Avatar contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { Avatar } from "./avatar.ts";
import type {
  AvatarElement,
  AvatarExpose,
  AvatarFallbackElement,
  AvatarImageCrossOrigin,
  AvatarImageDecoding,
  AvatarImageElement,
  AvatarImageFetchPriority,
  AvatarImageLoading,
  AvatarImageReferrerPolicy,
  AvatarPresence,
  AvatarSlotState,
  AvatarState,
  AvatarStatus,
} from "./avatar.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: AvatarExpose;

type _StateIsLiteral = Expect<Equal<AvatarState, "fallback" | "image">>;
type _PresenceIsLiteral = Expect<Equal<AvatarPresence, "missing" | "present">>;
type _StatusIsLiteral = Expect<
  Equal<AvatarStatus, "away" | "busy" | "none" | "offline" | "online">
>;
type _LoadingIsNativeLiteral = Expect<Equal<AvatarImageLoading, "eager" | "lazy">>;
type _DecodingIsNativeLiteral = Expect<Equal<AvatarImageDecoding, "async" | "auto" | "sync">>;
type _FetchPriorityIsNativeLiteral = Expect<
  Equal<AvatarImageFetchPriority, "auto" | "high" | "low">
>;
type _CrossOriginIsNativeLiteral = Expect<
  Equal<AvatarImageCrossOrigin, "" | "anonymous" | "use-credentials">
>;
type _ReferrerPolicyIncludesStrictOrigin = Expect<
  "strict-origin" extends AvatarImageReferrerPolicy ? true : false
>;
type _ElementIsRenderable = Expect<Equal<AvatarElement, Element | ComponentPublicInstance>>;
type _ImageElementIsRenderable = Expect<Equal<AvatarImageElement, HTMLImageElement>>;
type _FallbackElementIsRenderable = Expect<Equal<AvatarFallbackElement, HTMLSpanElement>>;
type _ExposeStateIsLiteral = Expect<Equal<typeof exposed.state, AvatarState>>;
type _ExposeStatusIsLiteral = Expect<Equal<typeof exposed.status, AvatarStatus>>;
type _ExposeImageIsLiteral = Expect<Equal<typeof exposed.image, AvatarPresence>>;
type _ExposeNameStateIsLiteral = Expect<Equal<typeof exposed.nameState, AvatarPresence>>;
type _ExposeFallbackStateIsLiteral = Expect<Equal<typeof exposed.fallbackState, AvatarPresence>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    AvatarSlotState,
    {
      readonly state: AvatarState;
      readonly status: AvatarStatus;
      readonly src: string | undefined;
      readonly alt: string;
      readonly name: string | undefined;
      readonly fallback: string | undefined;
      readonly image: AvatarPresence;
      readonly nameState: AvatarPresence;
      readonly fallbackState: AvatarPresence;
    }
  >
>;

const exposedElement: AvatarElement | null = exposed.element;
const exposedImageElement: AvatarImageElement | null = exposed.imageElement;
const exposedFallbackElement: AvatarFallbackElement | null = exposed.fallbackElement;
const customHost: InstanceType<typeof Avatar>["$props"] = {
  as: componentTarget,
  crossOrigin: "anonymous",
  decoding: "async",
  fallback: "AK",
  fetchPriority: "low",
  loading: "lazy",
  name: "Aki Kimura",
  referrerPolicy: "no-referrer",
  src: "/avatars/aki.png",
  status: "online",
};
const slotState: AvatarSlotState = {
  alt: "Aki Kimura",
  fallback: "AK",
  fallbackState: "present",
  image: "present",
  name: "Aki Kimura",
  nameState: "present",
  src: "/avatars/aki.png",
  state: "image",
  status: "away",
};

// @ts-expect-error Avatar states are strict render tokens.
const invalidState: AvatarState = "loading";

// @ts-expect-error Avatar presence hooks are strict data tokens.
const invalidPresence: AvatarPresence = "visible";

// @ts-expect-error Avatar statuses are strict presence tokens.
const invalidStatus: AvatarStatus = "idle";

// @ts-expect-error loading must use native image loading tokens.
const invalidLoading: AvatarImageLoading = "auto";

// @ts-expect-error status must stay a strict token when provided as a prop.
const badStatusProp: InstanceType<typeof Avatar>["$props"] = { status: "idle" };

// @ts-expect-error decoding must stay a strict native image token.
const badDecodingProp: InstanceType<typeof Avatar>["$props"] = { decoding: "defer" };

void Avatar;
void badDecodingProp;
void badStatusProp;
void customHost;
void exposedElement;
void exposedFallbackElement;
void exposedImageElement;
void invalidLoading;
void invalidPresence;
void invalidState;
void invalidStatus;
void slotState;
