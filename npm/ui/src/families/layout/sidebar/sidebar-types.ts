/** Desktop open state mirrored to the Sidebar data contract. */
export type SidebarState = "collapsed" | "expanded";

/**
 * How the desktop sidebar collapses.
 *
 * - `"offcanvas"`: the collapsed sidebar leaves the layout and becomes inert.
 * - `"icon"`: the collapsed sidebar shrinks to a rail that keeps icons interactive.
 * - `"none"`: the sidebar is always expanded and toggling is a no-op.
 */
export type SidebarCollapsible = "icon" | "none" | "offcanvas";

/** Screen edge the sidebar is attached to. */
export type SidebarSide = "left" | "right";

/** Consumer styling token mirrored to `data-variant`. */
export type SidebarVariant = "floating" | "inset" | "sidebar";

/** Persistence adapter used to restore the desktop open state after mount. */
export interface SidebarStorage {
  /** Read a persisted value, or `null` when nothing was stored. */
  readonly get: (key: string) => string | null;

  /** Persist a value. */
  readonly set: (key: string, value: string) => void;
}

/** State exposed to Sidebar slots. */
export interface SidebarSlotState {
  /** Desktop open state. */
  readonly open: boolean;

  /** Desktop state token. */
  readonly state: SidebarState;

  /** Whether the mobile query currently matches. Always `false` during SSR. */
  readonly isMobile: boolean;

  /** Whether the mobile sheet is open. */
  readonly openMobile: boolean;

  /** Desktop collapse mode. */
  readonly collapsible: SidebarCollapsible;

  /** Attached screen edge. */
  readonly side: SidebarSide;

  /** Styling variant. */
  readonly variant: SidebarVariant;
}

/** Public instance exposed by SidebarProvider. */
export interface SidebarProviderExpose extends SidebarSlotState {
  /** Id of the sidebar landmark, wired to trigger `aria-controls`. */
  readonly sidebarId: string;

  /** Request a desktop open value and report whether it differs. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;

  /** Request a mobile sheet open value and report whether it differs. */
  readonly setOpenMobile: (value: boolean, event?: Event | null) => boolean;

  /** Toggle the mobile sheet on mobile, otherwise the desktop sidebar. */
  readonly toggle: (event?: Event | null) => boolean;
}

/** Public instance exposed by SidebarRoot. */
export interface SidebarRootExpose {
  /** Rendered desktop landmark, or `null` while rendered as a mobile sheet. */
  readonly element: HTMLElement | null;
}

/** Public instance exposed by SidebarTrigger and SidebarRail. */
export interface SidebarToggleExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the button. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by structural Sidebar parts. */
export interface SidebarSectionExpose {
  /** Rendered element or component instance. */
  readonly element: Element | null;
}

/** Public instance exposed by SidebarGroup. */
export interface SidebarGroupExpose extends SidebarSectionExpose {
  /** Id consumed by SidebarGroupLabel and wired to `aria-labelledby`. */
  readonly labelId: string;
}
