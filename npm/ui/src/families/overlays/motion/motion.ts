import "./motion.css";

export {
  prefersReducedMotion,
  startViewTransition,
  supportsScrollDrivenAnimations,
  supportsStartingStyle,
  supportsViewTransitions,
  useReducedMotion,
} from "./motion-adapters.ts";

export {
  motionDelays,
  motionDurations,
  motionEasings,
  motionTokenProperty,
  motionTokenVar,
  setMotionTokens,
} from "./motion-tokens.ts";

export type {
  MotionDelayToken,
  MotionDurationToken,
  MotionEasingToken,
  MotionEnterExitRecipe,
  MotionRecipe,
  MotionRecipeHook,
  MotionTokenName,
  MotionTokenOverrides,
  StartViewTransitionOptions,
  ViewTransitionHandle,
} from "./motion-types.ts";
