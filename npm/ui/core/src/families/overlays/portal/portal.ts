export {
  portalDepthKey,
  registerPortalLayer,
  topPortalLayer,
  usePortalStack,
} from "./portal-stack.ts";
export type { PortalStackEntry } from "./portal-stack.ts";

/** Accessible, unstyled portal that moves content to a document target. */
export { default as Portal } from "./portal.vue";
