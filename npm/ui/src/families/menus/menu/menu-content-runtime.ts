import { nextTick, onMounted, onUnmounted, shallowRef, watch } from "vue";
import type { ShallowRef } from "vue";

import { createInertOutside } from "../../accessibility/inert-outside/inert-outside.ts";
import { createFocusScope } from "../../accessibility/focus-scope/focus-scope.ts";
import { createScrollLock } from "../../accessibility/scroll-lock/scroll-lock.ts";
import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { usePointerGrace } from "../../interaction/pointer-grace/pointer-grace.ts";
import { useTypeahead } from "../../interaction/typeahead/typeahead.ts";
import { createDismissableLayer } from "../../overlays/dismissable-layer/dismissable-layer.ts";
import type { DismissableLayerController } from "../../overlays/dismissable-layer/dismissable-layer.ts";
import { menuContentContext } from "./menu-context.ts";
import type {
  MenuContentContextValue,
  MenuItemRecord,
  MenuLevelContextValue,
  MenuPoint,
  MenuTreeContextValue,
} from "./menu-context.ts";
import {
  backwardKey,
  eventTargetElement,
  focusElement,
  forwardKey,
  isInsideLevelChain,
  toHTMLElement,
} from "./menu-dom.ts";
import type {
  MenuAutoFocusEvent,
  MenuDismissEvent,
  MenuEscapeKeyDownEvent,
  MenuFocusOutsideEvent,
  MenuInteractOutsideEvent,
  MenuPointerDownOutsideEvent,
} from "./menu-types.ts";

/** Consumer callbacks forwarded from content emits. */
export interface MenuContentCallbacks {
  readonly openAutoFocus: (event: MenuAutoFocusEvent) => void;
  readonly closeAutoFocus: (event: MenuAutoFocusEvent) => void;
  readonly escapeKeyDown: (event: MenuEscapeKeyDownEvent) => void;
  readonly pointerDownOutside: (event: MenuPointerDownOutsideEvent) => void;
  readonly focusOutside: (event: MenuFocusOutsideEvent) => void;
  readonly interactOutside: (event: MenuInteractOutsideEvent) => void;
  readonly dismiss: (event: MenuDismissEvent) => void;
}

/** Inputs for {@link useMenuContent}. */
export interface MenuContentOptions {
  readonly element: Readonly<ShallowRef<HTMLDivElement | null>>;
  readonly tree: MenuTreeContextValue;
  readonly level: MenuLevelContextValue;
  readonly parentContent: MenuContentContextValue | undefined;
  readonly closeOnEscape: () => boolean;
  readonly closeOnPointerDownOutside: () => boolean;
  readonly closeOnFocusOutside: () => boolean;
  readonly callbacks: MenuContentCallbacks;
}

/** Static role, focus target, and handlers bound onto the menu element. */
export interface MenuContentInteractiveProps {
  readonly role: "menu";
  readonly tabindex: -1;
  readonly "aria-orientation": "vertical";
  readonly "data-vize-dismissable-layer": "";
  readonly onKeydown: (event: KeyboardEvent) => void;
  readonly onFocus: (event: FocusEvent) => void;
  readonly onPointerenter: () => void;
}

/** Handlers and helpers returned to MenuContent and MenuSubContent. */
export interface MenuContentController {
  readonly context: MenuContentContextValue;
  readonly dismissableLayer: DismissableLayerController;
  /** Bind with `v-bind` on the `role="menu"` element. */
  readonly contentProps: MenuContentInteractiveProps;
  readonly focusFirst: () => HTMLElement | null;
  readonly focusLast: () => HTMLElement | null;
  readonly focusContent: () => void;
}

const navigationKeys: Readonly<Record<string, "first" | "last" | "next" | "previous">> = {
  ArrowDown: "next",
  ArrowUp: "previous",
  End: "last",
  Home: "first",
  PageDown: "last",
  PageUp: "first",
};

/**
 * Shared keyboard, focus, typeahead, pointer-grace, and dismissal core of
 * every rendered menu surface (root content and submenu content).
 */
export function useMenuContent(options: MenuContentOptions): MenuContentController {
  const { element, tree, level, parentContent, callbacks } = options;
  const isRoot = level.parent === null;
  const registry = createCollectionRegistry<string, MenuItemRecord>({
    disabledBehavior: "focusable",
  });
  const ownerDocument = shallowRef<Document | null>(null);
  let graceArmed = false;
  let graceTimer: ReturnType<typeof setTimeout> | undefined;
  const graceDelay = 300;

  function focusItem(key: string): void {
    registry.setActiveKey(key);
    focusElement(registry.getItem(key)?.element);
  }

  function focusContent(): void {
    registry.setActiveKey(null);
    focusElement(element.value);
  }

  function focusEdge(edge: "first" | "last"): HTMLElement | null {
    registry.refresh();
    const key = registry.getNavigationKey(edge);
    if (key === null) return null;
    focusItem(key);
    return toHTMLElement(registry.getItem(key)?.element);
  }

  const typeahead = useTypeahead({
    registry,
    onMatch: (match) => focusItem(match.key),
  });

  const grace = usePointerGrace({ delay: graceDelay, onGraceEnd: () => clearGrace() });

  function clearGrace(): void {
    if (graceTimer !== undefined) clearTimeout(graceTimer);
    graceTimer = undefined;
    graceArmed = false;
    grace.setOrigin(null);
    grace.setTarget(null);
  }

  function startGrace(origin: MenuPoint, target: HTMLElement): void {
    const rect = target.getBoundingClientRect();
    grace.setTarget({ x: rect.x, y: rect.y, width: rect.width, height: rect.height });
    grace.setOrigin(origin);
    graceArmed = true;
    grace.handleMove(origin);
    // The safe triangle is a transit corridor, not a resting place: it expires
    // after the grace delay even while the pointer stays inside it.
    if (graceTimer !== undefined) clearTimeout(graceTimer);
    graceTimer = setTimeout(clearGrace, graceDelay);
  }

  function isPointerInGrace(event: PointerEvent): boolean {
    if (!graceArmed) return false;
    const point = { x: event.clientX, y: event.clientY };
    grace.handleMove(point);
    return grace.contains(point);
  }

  const context: MenuContentContextValue = {
    level,
    registry,
    typeaheadQuery: typeahead.query,
    focusItem,
    focusContent,
    isPointerInGrace,
    startGrace,
    clearGrace,
  };
  menuContentContext.provide(context);

  // Moving the highlight away from a submenu trigger closes that submenu.
  watch(
    registry.activeKey,
    (key) => {
      if (key === null) return;
      for (const item of registry.items.value) {
        const sub = item.value.subLevel;
        if (sub && item.key !== key && sub.open.value) sub.setOpen(false);
      }
    },
    { flush: "sync" },
  );

  function entryTarget(): HTMLElement | null {
    const entry = level.entryFocus.value;
    if (entry === "first" || entry === "last") {
      registry.refresh();
      const key = registry.getNavigationKey(entry);
      if (key !== null) {
        registry.setActiveKey(key);
        const target = toHTMLElement(registry.getItem(key)?.element);
        if (target) return target;
      }
    }
    return element.value;
  }

  function onDismiss(event: MenuDismissEvent): void {
    callbacks.dismiss(event);
    if (isRoot) {
      if (event.reason !== "escape-key") level.skipFocusReturn.value = true;
      tree.closeAll(event.originalEvent);
      return;
    }
    if (event.reason === "escape-key") {
      level.setOpen(false, event.originalEvent);
      focusElement(level.triggerElement.value);
      return;
    }
    if (event.target && isInsideLevelChain(level.parent, event.target)) {
      level.setOpen(false, event.originalEvent);
      return;
    }
    tree.root.skipFocusReturn.value = true;
    tree.closeAll(event.originalEvent);
  }

  const dismissableLayer = createDismissableLayer({
    root: element,
    branches: () => {
      const trigger = level.triggerElement.value;
      return trigger ? [trigger] : [];
    },
    enabled: () => level.open.value,
    escapeKey: options.closeOnEscape,
    outsideFocus: options.closeOnFocusOutside,
    outsidePointerDown: options.closeOnPointerDownOutside,
    onEscapeKeyDown: callbacks.escapeKeyDown,
    onFocusOutside: callbacks.focusOutside,
    onInteractOutside: callbacks.interactOutside,
    onPointerDownOutside: callbacks.pointerDownOutside,
    onDismiss,
  });
  const focusScope = createFocusScope({
    root: element,
    autoFocus: () => level.entryFocus.value !== "none",
    restoreFocus: () => isRoot,
    initialFocus: entryTarget,
    restoreTarget: () => level.restoreTarget(),
    fallbackFocus: () => element.value,
    onMountAutoFocus: callbacks.openAutoFocus,
    onUnmountAutoFocus: (event) => {
      if (level.skipFocusReturn.value) event.preventDefault();
      callbacks.closeAutoFocus(event);
    },
  });
  const modalActive = () => isRoot && level.open.value && tree.modal.value;
  const inertOutside = createInertOutside({
    root: element,
    branches: () => {
      const trigger = level.triggerElement.value;
      return trigger ? [trigger, ...tree.layers.value] : tree.layers.value;
    },
    enabled: modalActive,
  });
  const scrollLock = createScrollLock({ document: ownerDocument, enabled: modalActive });

  let mounted = false;
  const shouldActivate = () => mounted && level.open.value && element.value !== null;
  function activate(): void {
    // The portal moves freshly mounted content into its target on the next
    // render; activating afterwards keeps entry focus from being dropped by
    // that DOM move and lets the inert/dismissal roots see the final tree.
    if (!shouldActivate()) return;
    ownerDocument.value = element.value?.ownerDocument ?? null;
    registerLayer(element.value);
    scrollLock.activate();
    inertOutside.activate();
    dismissableLayer.activate();
    focusScope.activate();
  }
  function registerLayer(layer: HTMLElement | null): void {
    if (isRoot || !layer || tree.layers.value.includes(layer)) return;
    tree.layers.value = [...tree.layers.value, layer];
  }
  function releaseLayer(): void {
    const layer = element.value ?? level.contentElement.value;
    if (isRoot) return;
    tree.layers.value = tree.layers.value.filter(
      (candidate) => candidate !== layer && candidate.isConnected,
    );
  }
  function sync(): void {
    if (!shouldActivate()) {
      releaseLayer();
      dismissableLayer.deactivate();
      inertOutside.deactivate();
      scrollLock.deactivate();
      focusScope.deactivate();
      return;
    }
    void nextTick(activate);
  }

  watch(
    element,
    (next, previous) => {
      if (previous && level.contentElement.value === previous) level.contentElement.value = null;
      if (next) level.contentElement.value = next;
      sync();
    },
    { flush: "post" },
  );
  watch(() => level.open.value, sync, { flush: "post" });
  onMounted(() => {
    mounted = true;
    level.focusContentEdge.value = focusEdge;
    sync();
  });
  onUnmounted(() => {
    mounted = false;
    sync();
    dismissableLayer.dispose();
    focusScope.dispose();
    inertOutside.dispose();
    scrollLock.dispose();
    if (graceTimer !== undefined) clearTimeout(graceTimer);
    graceArmed = false;
    if (level.focusContentEdge.value === focusEdge) level.focusContentEdge.value = null;
    if (level.contentElement.value === element.value) level.contentElement.value = null;
  });

  function ownsEvent(event: Event): boolean {
    const root = element.value;
    if (!root) return false;
    const target = eventTargetElement(event.target, root);
    return target?.closest('[role="menu"]') === root;
  }

  function onKeydown(event: KeyboardEvent): void {
    if (event.defaultPrevented || event.isComposing || !ownsEvent(event)) return;
    const dir = tree.dir.value;
    if (event.key === "Tab") {
      event.preventDefault();
      tree.closeAll(event);
      return;
    }
    const direction =
      event.altKey || event.ctrlKey || event.metaKey ? undefined : navigationKeys[event.key];
    if (direction) {
      event.preventDefault();
      const key = registry.moveActive(direction, { loop: tree.loop.value });
      if (key !== null) focusItem(key);
      return;
    }
    if (event.key === backwardKey(dir) && !isRoot) {
      event.preventDefault();
      level.setOpen(false, event);
      focusElement(level.triggerElement.value);
      return;
    }
    if (event.key === backwardKey(dir) || event.key === forwardKey(dir)) {
      const edge = event.key === forwardKey(dir) ? "next" : "previous";
      if (tree.edgeNavigate(edge, event)) event.preventDefault();
      return;
    }
    typeahead.typeaheadProps.onKeydown(event);
  }

  function onFocus(event: FocusEvent): void {
    if (event.target === element.value) registry.setActiveKey(null);
  }

  function onPointerenter(): void {
    parentContent?.clearGrace();
  }

  return {
    context,
    dismissableLayer,
    contentProps: Object.freeze({
      role: "menu",
      tabindex: -1,
      "aria-orientation": "vertical",
      ...dismissableLayer.layerProps,
      onKeydown,
      onFocus,
      onPointerenter,
    }),
    focusFirst: () => focusEdge("first"),
    focusLast: () => focusEdge("last"),
    focusContent,
  };
}
