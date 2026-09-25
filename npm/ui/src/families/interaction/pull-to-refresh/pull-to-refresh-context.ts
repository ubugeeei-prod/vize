import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";

/** Refresh state shared with PullToRefreshTrigger. */
export interface PullToRefreshContextValue {
  readonly refreshing: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly rootId: ComputedRef<string>;
  readonly trigger: () => void;
}

/** Typed context shared by PullToRefresh and its keyboard trigger. */
export const pullToRefreshContext = createContext<PullToRefreshContextValue>("PullToRefresh");
