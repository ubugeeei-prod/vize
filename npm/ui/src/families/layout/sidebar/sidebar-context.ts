import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  SidebarCollapsible,
  SidebarSide,
  SidebarState,
  SidebarVariant,
} from "./sidebar-types.ts";

/** Shared state and actions for the Sidebar compound components. */
export interface SidebarContextValue {
  readonly sidebarId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly openMobile: ComputedRef<boolean>;
  readonly isMobile: ComputedRef<boolean>;
  readonly state: ComputedRef<SidebarState>;
  readonly collapsible: ComputedRef<SidebarCollapsible>;
  readonly side: ComputedRef<SidebarSide>;
  readonly variant: ComputedRef<SidebarVariant>;
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;
  readonly setOpenMobile: (value: boolean, event?: Event | null) => boolean;
  readonly toggle: (event?: Event | null) => boolean;
}

export const sidebarContext = createContext<SidebarContextValue>("Sidebar");

/** Per-group label wiring shared by SidebarGroup and SidebarGroupLabel. */
export interface SidebarGroupContextValue {
  readonly labelId: ComputedRef<string>;
}

export const sidebarGroupContext = createContext<SidebarGroupContextValue>("SidebarGroup");
