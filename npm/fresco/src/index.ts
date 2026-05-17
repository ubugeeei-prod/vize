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
  type RenderOptions,
  type RenderInstance,
  type Instance,
  type RenderMetrics,
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
export {
  kittyFlags,
  kittyModifiers,
  resolveKittyFlags,
  type KittyFlagName,
  type KittyKeyboardOptions,
} from "./kittyKeyboard.js";
export { measureElement, type DOMElement } from "./measureElement.js";

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
  TerminalOptionsNapi,
  LayoutResultNapi,
  ModifiersNapi,
} from "@vizejs/fresco-native";
