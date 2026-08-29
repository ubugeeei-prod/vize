/** Compile-only assertions for the public link contract. */

import type {
  LinkAriaCurrent,
  LinkDownload,
  LinkExpose,
  LinkProps,
  LinkSlotState,
} from "./link.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const link: LinkExpose;

type _CurrentStatesAreClosed = Expect<
  Equal<LinkAriaCurrent, boolean | "date" | "location" | "page" | "step" | "time">
>;
type _DownloadMatchesNativeValues = Expect<Equal<LinkDownload, boolean | string>>;
type _SlotStateIsExplicit = Expect<
  Equal<
    LinkSlotState,
    { readonly disabled: boolean; readonly inert: boolean; readonly unavailable: boolean }
  >
>;

export const validLinkProps: LinkProps = {
  id: null,
  href: "/docs",
  target: "_blank",
  rel: "noopener",
  download: true,
  disabled: false,
  inert: false,
  ariaCurrent: "page",
};
export const currentByBoolean: LinkAriaCurrent = true;
export const nonCurrentByBoolean: LinkAriaCurrent = false;
export const namedDownload: LinkDownload = "guide.pdf";

link.focus();

// @ts-expect-error current state is limited to documented native tokens.
export const invalidCurrent: LinkAriaCurrent = "route";

// @ts-expect-error download accepts only native boolean/string values.
export const invalidDownload: LinkDownload = 1;

// @ts-expect-error disabled is boolean-only.
export const invalidProps: LinkProps = { disabled: "true" };
