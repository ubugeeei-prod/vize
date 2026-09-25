/** Native landmark regions with F6 / Shift+F6 landmark cycling. */
export { default as Landmark } from "./landmark.vue";
export { default as LandmarkProvider } from "./landmark-provider.vue";
export {
  createLandmarkNavigation,
  defaultLandmarkNextKey,
  defaultLandmarkPreviousKey,
  focusLandmarkElement,
  isLandmarkAvailable,
  isNamedLandmarkRole,
  landmarkContext,
  landmarkElements,
  landmarkLabelOf,
  landmarkRoleOf,
  useLandmarkNavigation,
} from "./landmark-runtime.ts";
export type {
  LandmarkElementMap,
  LandmarkExpose,
  LandmarkInfo,
  LandmarkKeyBinding,
  LandmarkNavigationController,
  LandmarkNavigationOptions,
  LandmarkProviderExpose,
  LandmarkRegistrationInput,
  LandmarkRole,
  LandmarkSlotState,
  NamedLandmarkRole,
} from "./landmark-types.ts";
