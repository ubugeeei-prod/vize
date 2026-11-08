/**
 * Fresco Composables
 */

export {
  useInput,
  useKeyPress,
  type UseInputOptions,
  type UseInputReturn,
  type InputHandler,
  type KeyHandler,
  type Key,
} from "./useInput.js";
export { usePaste, type UsePasteOptions } from "./usePaste.js";
export { useMouse, type UseMouseOptions, type UseMouseReturn } from "./useMouse.js";
export {
  provideFocusScope,
  useFocus,
  useFocusManager,
  type FocusManager,
  type UseFocusOptions,
} from "./useFocus.js";
export { useApp, type UseAppReturn } from "./useApp.js";
export { useIme, type UseImeOptions, type ImeManager } from "./useIme.js";
export {
  useStdin,
  useStdout,
  useStderr,
  type UseStdinReturn,
  type UseStdoutReturn,
  type UseStderrReturn,
} from "./useStreams.js";
export { useWindowSize, type WindowSize } from "./useWindowSize.js";
export { useCursor, type CursorPosition } from "./useCursor.js";
export { useAnimation, type AnimationResult, type UseAnimationOptions } from "./useAnimation.js";
export { useBoxMetrics, type BoxMetrics, type UseBoxMetricsResult } from "./useBoxMetrics.js";
export { useIsScreenReaderEnabled } from "./useIsScreenReaderEnabled.js";
export {
  createVirtualViewport,
  useVirtualViewport,
  type VirtualScrollAlignment,
  type VirtualViewport,
  type VirtualViewportOptions,
  type VirtualViewportRange,
} from "./useVirtualViewport.js";
export {
  createKeymapResolver,
  defineKeymap,
  useKeymap,
  type Keymap,
  type KeymapBinding,
  type KeymapMatch,
  type KeymapResolver,
  type KeymapStroke,
  type UseKeymapOptions,
  type UseKeymapReturn,
} from "./useKeymap.js";
