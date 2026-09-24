/** Accessible, unstyled collapsible application sidebar with rail and mobile-sheet modes. */
export { default as SidebarProvider } from "./sidebar-provider.vue";
export { default as Sidebar, default as SidebarRoot } from "./sidebar-root.vue";
export { default as SidebarContent } from "./sidebar-content.vue";
export { default as SidebarFooter } from "./sidebar-footer.vue";
export { default as SidebarGroup } from "./sidebar-group.vue";
export { default as SidebarGroupLabel } from "./sidebar-group-label.vue";
export { default as SidebarHeader } from "./sidebar-header.vue";
export { default as SidebarInset } from "./sidebar-inset.vue";
export { default as SidebarRail } from "./sidebar-rail.vue";
export { default as SidebarTrigger } from "./sidebar-trigger.vue";
export { sidebarContext } from "./sidebar-context.ts";
export type { SidebarContextValue } from "./sidebar-context.ts";
export type {
  SidebarCollapsible,
  SidebarGroupExpose,
  SidebarProviderExpose,
  SidebarRootExpose,
  SidebarSectionExpose,
  SidebarSide,
  SidebarSlotState,
  SidebarState,
  SidebarStorage,
  SidebarToggleExpose,
  SidebarVariant,
} from "./sidebar-types.ts";
