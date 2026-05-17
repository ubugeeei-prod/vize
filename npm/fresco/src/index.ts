/**
 * @vizejs/fresco - Vue TUI Framework
 *
 * Build terminal user interfaces with Vue.js
 */

// Core
export {
  createApp,
  render,
  renderToString,
  type App,
  type AppOptions,
  type RenderInstance,
  type RenderToStringOptions,
  lastKeyEvent,
  lastPasteEvent,
  lastResizeEvent,
  lastMouseEvent,
  lastFocusEvent,
  lastCompositionEvent,
  type KeyEvent,
  type PasteEvent,
  type ResizeEvent,
  type MouseEvent,
  type FocusEvent,
  type CompositionEvent,
} from "./app.js";
export { createRenderer } from "./renderer.js";

// Components
export * from "./components/index.js";

// Composables
export * from "./composables/index.js";

// Re-export native bindings types
export type {
  StyleNapi,
  FlexStyleNapi,
  RenderNodeNapi,
  InputEventNapi,
  ImeStateNapi,
  TerminalInfoNapi,
  LayoutResultNapi,
  ModifiersNapi,
} from "@vizejs/fresco-native";
