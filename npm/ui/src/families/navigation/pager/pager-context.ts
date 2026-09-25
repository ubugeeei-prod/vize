import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { PagerChangeReason } from "./pager-types.ts";

/** Type-erased Pager state shared with its parts. */
export interface PagerContextValue {
  readonly baseId: ComputedRef<string>;
  readonly active: ComputedRef<string>;
  readonly pages: ComputedRef<readonly string[]>;
  readonly select: (page: string, reason: PagerChangeReason) => boolean;
  readonly registerTab: (page: string, element: HTMLElement | null) => void;
  readonly focusTab: (page: string) => void;
  readonly registerViewport: (element: HTMLElement | null) => void;
  readonly settleScroll: () => void;
}

/** Typed context shared by Pager and its parts. */
export const pagerContext = createContext<PagerContextValue>("Pager");

/** Deterministic tab and panel ids for a page. */
export function pagerIds(
  baseId: string,
  page: string,
): { readonly tab: string; readonly panel: string } {
  const segment = page.replace(/[^A-Za-z0-9_-]+/g, "-");
  return { tab: `${baseId}-tab-${segment}`, panel: `${baseId}-panel-${segment}` };
}
