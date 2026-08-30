/** Compile-only assertions for the public SkipLink contract. */

import { SkipLink } from "./skip-link.ts";
import type {
  SkipLinkActivation,
  SkipLinkExpose,
  SkipLinkFocusResult,
  SkipLinkHref,
  SkipLinkProps,
  SkipLinkSlotState,
  SkipLinkState,
} from "./skip-link.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const exposed: SkipLinkExpose;
declare const target: HTMLElement;

type _HrefIsHashOnly = Expect<Equal<SkipLinkHref, `#${string}`>>;
type _StateIsClosed = Expect<Equal<SkipLinkState, "focused" | "idle" | "invalid">>;
type _PropsKeysAreClosed = Expect<Equal<keyof SkipLinkProps, "focusTarget" | "href" | "id">>;
type _FocusResultIsStrict = Expect<
  Equal<SkipLinkFocusResult, { readonly target: HTMLElement | null; readonly focused: boolean }>
>;
type _ActivationIsStrict = Expect<
  Equal<
    SkipLinkActivation,
    {
      readonly href: SkipLinkHref;
      readonly targetId: string;
      readonly target: HTMLElement | null;
      readonly focused: boolean;
    }
  >
>;
type _SlotStateIsStrict = Expect<
  Equal<
    SkipLinkSlotState,
    {
      readonly focused: boolean;
      readonly href: SkipLinkHref | undefined;
      readonly state: SkipLinkState;
      readonly targetId: string | undefined;
      readonly unavailable: boolean;
    }
  >
>;
type _ExposeElementIsAnchor = Expect<Equal<typeof exposed.element, HTMLAnchorElement | null>>;
type _ExposeHrefIsOptionalHash = Expect<Equal<typeof exposed.href, SkipLinkHref | undefined>>;
type _ExposeTargetIdIsOptional = Expect<Equal<typeof exposed.targetId, string | undefined>>;

const publicProps = {
  focusTarget: false,
  href: "#content",
  id: null,
} satisfies SkipLinkProps;
const componentProps: InstanceType<typeof SkipLink>["$props"] = publicProps;
const slotState: SkipLinkSlotState = {
  focused: false,
  href: "#content",
  state: "idle",
  targetId: "content",
  unavailable: false,
};
const activation: SkipLinkActivation = {
  focused: true,
  href: "#content",
  target,
  targetId: "content",
};

exposed.focus();
exposed.focusTarget({ preventScroll: true });
exposed.getTarget();

// @ts-expect-error SkipLink href is same-document hash navigation only.
const invalidHref: SkipLinkHref = "/content";

// @ts-expect-error state is limited to the documented availability states.
const invalidState: SkipLinkState = "active";

// @ts-expect-error focusTarget is boolean-only.
const invalidProps: SkipLinkProps = { focusTarget: "true" };

void activation;
void componentProps;
void invalidHref;
void invalidProps;
void invalidState;
void publicProps;
void slotState;
