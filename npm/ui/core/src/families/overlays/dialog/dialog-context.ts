import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../../context.ts";
import type { DialogState } from "./dialog-types.ts";

/** Shared state and element registry for the Dialog compound components. */
export interface DialogContextValue {
  readonly id: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly titleId: ComputedRef<string>;
  readonly descriptionId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly modal: ComputedRef<boolean>;
  readonly state: ComputedRef<DialogState>;
  readonly triggerElement: ShallowRef<HTMLButtonElement | null>;
  readonly overlayElement: ShallowRef<HTMLElement | null>;
  readonly contentElement: ShallowRef<HTMLDivElement | null>;
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;
  readonly openDialog: (event?: Event | null) => boolean;
  readonly close: (event?: Event | null) => boolean;
  readonly toggle: (event?: Event | null) => boolean;
}

export const dialogContext = createContext<DialogContextValue>("Dialog");
