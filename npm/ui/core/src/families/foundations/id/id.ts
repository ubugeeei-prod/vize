export {
  createDeterministicIdScope,
  deriveDeterministicId,
  toDeterministicId,
  useDeterministicId,
} from "./deterministic-id.ts";
export type {
  DeterministicId,
  DeterministicIdChildScopeOptions,
  DeterministicIdOptions,
  DeterministicIdScope,
  DeterministicIdScopeOptions,
  DeterministicIdSeed,
} from "./deterministic-id.ts";

/** SSR-safe deterministic ID namespace for one application or component subtree. */
export { default as IdProvider } from "./deterministic-id-provider.vue";
