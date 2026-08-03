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
} from "./app.js";
export { createRenderer } from "./renderer.js";
export type {
  CompositionEvent,
  FocusEvent,
  FrescoAlignContent,
  FrescoAlignItems,
  FrescoAlignSelf,
  FrescoAppearance,
  FrescoBorderStyle,
  FrescoBoxRenderNode,
  FrescoCanonicalStyle,
  FrescoDimension,
  FrescoDisplay,
  FrescoFlexDirection,
  FrescoFlexWrap,
  FrescoInputEvent,
  FrescoInputRenderNode,
  FrescoJustifyContent,
  FrescoModifiers,
  FrescoOverflow,
  FrescoPosition,
  FrescoRenderNode,
  FrescoRenderNodeKind,
  FrescoRenderStyle,
  FrescoRootRenderNode,
  FrescoSnakeStyleAliases,
  FrescoStyle,
  FrescoTextRenderNode,
  FrescoTextWrapMode,
  InputEvent,
  KeyEvent,
  KeyEventType,
  MouseEvent,
  PasteEvent,
  ResizeEvent,
} from "./protocol.js";
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
