import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { PopconfirmCancelReason, PopconfirmState } from "./popconfirm-types.ts";

/** Shared state and actions for the Popconfirm compound components. */
export interface PopconfirmContextValue {
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<PopconfirmState>;
  readonly pending: ComputedRef<boolean>;
  readonly error: Readonly<ShallowRef<unknown>>;
  readonly titleId: ComputedRef<string>;
  readonly descriptionId: ComputedRef<string>;
  readonly confirmId: ComputedRef<string>;
  readonly cancelId: ComputedRef<string>;
  readonly confirmElement: ShallowRef<HTMLButtonElement | null>;
  readonly cancelElement: ShallowRef<HTMLButtonElement | null>;
  readonly confirm: (event?: Event | null) => Promise<boolean>;
  readonly cancel: (event: Event | null, reason: PopconfirmCancelReason) => boolean;
}

export const popconfirmContext = createContext<PopconfirmContextValue>("Popconfirm");
