/** Region exposing `env(safe-area-inset-*)` as CSS variables, data attributes, and optional padding. */
export { default as SafeArea } from "./safe-area.vue";
export { SAFE_AREA_EDGES, SAFE_AREA_STYLE, useSafeAreaInsets } from "./safe-area-runtime.ts";
export type {
  SafeAreaEdge,
  SafeAreaEdgeInsets,
  SafeAreaInsetsController,
  SafeAreaInsetsOptions,
} from "./safe-area-runtime.ts";
