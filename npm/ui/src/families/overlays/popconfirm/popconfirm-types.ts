import type { Placement } from "../positioner/positioner.ts";

/** Request lifecycle mirrored to `data-state` on Popconfirm parts. */
export type PopconfirmState = "idle" | "pending";

/** Which action receives focus when the confirmation opens. */
export type PopconfirmInitialFocus = "cancel" | "confirm" | "none";

/** Why a Popconfirm closed without confirming. */
export type PopconfirmCancelReason = "cancel-button" | "dismiss" | "programmatic";

/** Placement accepted by PopconfirmContent. */
export type PopconfirmPlacement = Placement;

/**
 * Confirmation handler. Returning a promise keeps the confirmation open and
 * pending until it settles: it closes on resolve and stays open on reject.
 */
export type PopconfirmConfirmHandler = () => void | PromiseLike<void>;

/** State exposed to Popconfirm slots. */
export interface PopconfirmSlotState {
  /** Whether the confirmation is open. */
  readonly open: boolean;
  /** Request lifecycle. */
  readonly state: PopconfirmState;
  /** Whether an asynchronous confirmation is running. */
  readonly pending: boolean;
  /** Rejection reason of the latest failed confirmation, or `null`. */
  readonly error: unknown;
}

/** Public instance exposed by PopconfirmRoot. */
export interface PopconfirmRootExpose extends PopconfirmSlotState {
  /** Root-owned base id. */
  readonly id: string;
  /** Id of the rendered title. */
  readonly titleId: string;
  /** Id of the rendered description. */
  readonly descriptionId: string;
  /** Request an open value. Closing is refused while pending. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;
  /** Run the confirmation handler. Resolves `true` when it succeeded and closed. */
  readonly confirm: (event?: Event | null) => Promise<boolean>;
  /** Close without confirming. Refused while pending. */
  readonly cancel: (event?: Event | null) => boolean;
}

/** Public instance exposed by PopconfirmContent. */
export interface PopconfirmContentExpose extends PopconfirmSlotState {
  /** Rendered popover element carrying `role="alertdialog"`. */
  readonly element: HTMLDivElement | null;
}

/** Public instance exposed by PopconfirmTrigger, PopconfirmConfirm, and PopconfirmCancel. */
export interface PopconfirmButtonExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;
  /** Move focus to the button. */
  readonly focus: (options?: FocusOptions) => void;
}
