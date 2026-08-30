/** Accessible, unstyled breadcrumb navigation primitive for route hierarchies. */
export { default as Breadcrumb, default as BreadcrumbRoot } from "./breadcrumb.vue";
export { default as BreadcrumbItem } from "./breadcrumb-item.vue";
export { default as BreadcrumbLink } from "./breadcrumb-link.vue";
export { default as BreadcrumbList } from "./breadcrumb-list.vue";
export { default as BreadcrumbSeparator } from "./breadcrumb-separator.vue";
export type {
  BreadcrumbCurrent,
  BreadcrumbItemExpose,
  BreadcrumbItemSlotState,
  BreadcrumbLinkExpose,
  BreadcrumbLinkSlotState,
  BreadcrumbListExpose,
  BreadcrumbRootExpose,
  BreadcrumbRootSlotState,
  BreadcrumbSeparatorExpose,
  BreadcrumbSeparatorSlotState,
} from "./breadcrumb-types.ts";
