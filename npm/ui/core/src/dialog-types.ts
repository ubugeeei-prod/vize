import type { FocusScopeAutoFocusEvent } from "./focus-scope.ts";

/** Dialog role announced by assistive technology. */
export type DialogRole = "alertdialog" | "dialog";

/** Open state mirrored to the Dialog data contract. */
export type DialogState = "closed" | "open";

/** State exposed to compound Dialog slots. */
export interface DialogSlotState {
  /** Whether the dialog content is currently visible and interactive. */
  readonly open: boolean;

  /** Whether outside content is inert, focus-contained, and scroll-locked. */
  readonly modal: boolean;

  /** Stable state token for styling and tests. */
  readonly state: DialogState;
}

/** Public instance exposed by DialogRoot. */
export interface DialogRootExpose extends DialogSlotState {
  /** Root-owned base id for the compound dialog family. */
  readonly id: string;

  /** Id wired from DialogTrigger to DialogContent. */
  readonly contentId: string;

  /** Default id consumed by DialogTitle. */
  readonly titleId: string;

  /** Default id consumed by DialogDescription. */
  readonly descriptionId: string;

  /** Request a specific open value and report whether it differs. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;

  /** Request the open state. */
  readonly openDialog: (event?: Event | null) => boolean;

  /** Request the closed state. */
  readonly close: (event?: Event | null) => boolean;

  /** Request the opposite open state. */
  readonly toggle: (event?: Event | null) => boolean;
}

/** Public instance exposed by DialogTrigger. */
export interface DialogTriggerExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by DialogPortal. */
export interface DialogPortalExpose extends DialogSlotState {
  /** Whether portal contents are rendered. */
  readonly present: boolean;
}

/** Public instance exposed by DialogOverlay. */
export interface DialogOverlayExpose extends DialogSlotState {
  /** Rendered overlay element. */
  readonly element: HTMLDivElement | null;
}

/** Public instance exposed by DialogContent. */
export interface DialogContentExpose extends DialogSlotState {
  /** Rendered dialog content element. */
  readonly element: HTMLDivElement | null;

  /** Focus the first eligible descendant in the dialog content. */
  readonly focusFirst: () => HTMLElement | null;

  /** Move focus to the content fallback target. */
  readonly focusContent: (options?: FocusOptions) => void;
}

/** Public instance exposed by DialogTitle. */
export interface DialogTitleExpose {
  /** Rendered title element or component instance. */
  readonly element: Element | null;
}

/** Public instance exposed by DialogDescription. */
export interface DialogDescriptionExpose {
  /** Rendered description element or component instance. */
  readonly element: Element | null;
}

/** Public instance exposed by DialogClose. */
export interface DialogCloseExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the close button. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Preventable Dialog auto-focus lifecycle event. */
export type DialogAutoFocusEvent = FocusScopeAutoFocusEvent;
