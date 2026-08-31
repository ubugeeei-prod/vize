import { getCurrentScope, onScopeDispose } from "vue";

import {
  eventElement,
  focusItem,
  revealItem,
  validateActiveDescendant,
} from "./composite-navigation-dom.ts";
import {
  capture,
  isEditableDescendant,
  keyIntent,
  readBoolean,
  readDirection,
  readOrientation,
  readPageSize,
  surfaceErrors,
  validateCommand,
  validateId,
  validateOptions,
} from "./composite-navigation-internal.ts";
import type {
  CompositeContainerProps,
  CompositeItemProps,
  CompositeNavigationCommand,
  CompositeNavigationChange,
  CompositeNavigationController,
  CompositeNavigationIntent,
  CompositeNavigationOptions,
} from "./composite-navigation-types.ts";
import { createTypeahead } from "../../interaction/typeahead/typeahead.ts";
import type { TypeaheadController } from "../../interaction/typeahead/typeahead.ts";
import type { CollectionItem, CollectionKey } from "../collection/collection.ts";

const disposedDiagnostic = "VIZE_UI_COMPOSITE_NAVIGATION_DISPOSED";
const itemDiagnostic = "VIZE_UI_COMPOSITE_NAVIGATION_ITEM";
const setupDiagnostic = "VIZE_UI_COMPOSITE_NAVIGATION_SETUP";

interface ItemHandlers {
  readonly onFocus: (event: FocusEvent) => void;
  readonly onPointerdown: (event: PointerEvent) => void;
}

/** Create an SSR-safe roving-tabindex or active-descendant collection adapter. */
export function createCompositeNavigation<Key extends CollectionKey, Value>(
  options: CompositeNavigationOptions<Key, Value>,
): CompositeNavigationController<Key> {
  const strategy = validateOptions(options);
  const registry = options.registry;
  const itemHandlers = new Map<Key, ItemHandlers>();
  let container: Element | null = null;
  let disposed = false;

  const assertActive = () => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };
  const getItem = (key: Key): CollectionItem<Key, Value> => {
    const item = registry.getItem(key);
    if (!item) throw new Error(`${itemDiagnostic}: no item is registered for ${String(key)}`);
    return item;
  };
  const effectiveKey = (): Key | null =>
    registry.activeKey.value ?? registry.navigableItems.value[0]?.key ?? null;
  const itemId = (item: CollectionItem<Key, Value>): string | undefined => {
    const getId = options.getItemId;
    const value = getId?.(item) ?? (item.element?.id || undefined);
    if (value === undefined) {
      if (strategy === "active-descendant") {
        throw new Error(
          "VIZE_UI_COMPOSITE_NAVIGATION_ID: active-descendant items require a stable ID",
        );
      }
      return undefined;
    }
    return validateId(value);
  };

  const synchronizeDom = (
    key: Key,
    originalEvent: Event | null,
    moveDomFocus: boolean,
    errors: unknown[],
  ): void => {
    const item = getItem(key);
    if (strategy === "active-descendant") {
      validateActiveDescendant(container, item, itemId(item)!, errors);
    }
    const preventScroll =
      strategy === "roving" &&
      readBoolean((options as { preventScroll?: unknown }).preventScroll, "preventScroll");
    if (strategy === "roving" && moveDomFocus) {
      focusItem(item.element, preventScroll, errors);
    }
    if (
      options.scrollIntoView ||
      strategy === "active-descendant" ||
      (moveDomFocus && preventScroll)
    ) {
      revealItem(item, options.scrollIntoView, originalEvent, errors);
    }
  };
  const notify = (
    key: Key,
    previousKey: Key | null,
    intent: CompositeNavigationIntent,
    originalEvent: Event | null,
    errors: unknown[],
  ): void => {
    const change: CompositeNavigationChange<Key> = Object.freeze({
      key,
      previousKey,
      intent,
      originalEvent,
      focusStrategy: strategy,
    });
    capture(errors, () => options.onNavigate?.(change));
  };
  const commit = (
    mutate: () => void,
    intent: CompositeNavigationIntent,
    originalEvent: Event | null,
    moveDomFocus: boolean,
  ): Key | null => {
    const previousKey = registry.activeKey.value;
    const errors: unknown[] = [];
    capture(errors, mutate);
    const key = registry.activeKey.value;
    if (key !== null && key !== previousKey) {
      capture(errors, () => synchronizeDom(key, originalEvent, moveDomFocus, errors));
      notify(key, previousKey, intent, originalEvent, errors);
    }
    surfaceErrors(errors, "Composite navigation transition failed");
    return key;
  };

  const pageTarget = (intent: "page-next" | "page-previous"): Key | null => {
    const direction = intent === "page-next" ? "next" : "previous";
    let key = effectiveKey();
    for (let index = 0; index < readPageSize(options.pageSize); index++) {
      const next = registry.getNavigationKey(direction, { fromKey: key, loop: false });
      if (next === null || next === key) break;
      key = next;
    }
    return key;
  };
  const navigate = (
    intent: CompositeNavigationCommand,
    originalEvent: Event | null = null,
  ): Key | null => {
    assertActive();
    intent = validateCommand(intent);
    if (readBoolean(options.isDisabled, "isDisabled")) return null;
    if (intent === "page-next" || intent === "page-previous") {
      const target = pageTarget(intent);
      if (target === null) return null;
      return commit(
        () => {
          registry.setActiveKey(target);
        },
        intent,
        originalEvent,
        strategy === "roving",
      );
    }
    return commit(
      () => {
        registry.moveActive(intent, { loop: readBoolean(options.loop, "loop") });
      },
      intent,
      originalEvent,
      strategy === "roving",
    );
  };

  const configuredTypeahead = options.typeahead;
  const typeahead: TypeaheadController<Key> | null =
    configuredTypeahead && typeof configuredTypeahead === "object"
      ? createTypeahead({
          ...configuredTypeahead,
          registry,
          isDisabled: () =>
            readBoolean(options.isDisabled, "isDisabled") ||
            readBoolean(configuredTypeahead.isDisabled, "typeahead.isDisabled"),
          onMatch(match) {
            const errors: unknown[] = [];
            if (match.key !== match.previousKey) {
              capture(errors, () =>
                synchronizeDom(match.key, match.originalEvent, strategy === "roving", errors),
              );
              notify(match.key, match.previousKey, "typeahead", match.originalEvent, errors);
            }
            capture(errors, () => configuredTypeahead.onMatch?.(match));
            surfaceErrors(errors, "Composite typeahead transition failed");
          },
        })
      : null;

  const activateItem = (key: Key, event: FocusEvent | PointerEvent): void => {
    if (disposed || readBoolean(options.isDisabled, "isDisabled")) return;
    if (!registry.navigableItems.value.some((item) => item.key === key)) return;
    commit(
      () => {
        registry.setActiveKey(key);
      },
      event.type === "pointerdown" ? "pointer" : "focus",
      event,
      false,
    );
  };
  const handlersFor = (key: Key): ItemHandlers => {
    const cached = itemHandlers.get(key);
    if (cached) return cached;
    for (const cachedKey of itemHandlers.keys()) {
      if (!registry.getItem(cachedKey)) itemHandlers.delete(cachedKey);
    }
    const handlers = Object.freeze({
      onFocus: (event: FocusEvent) => activateItem(key, event),
      onPointerdown: (event: PointerEvent) => activateItem(key, event),
    });
    itemHandlers.set(key, handlers);
    return handlers;
  };

  const onFocus = (event: FocusEvent): void => {
    if (disposed) return;
    container = eventElement(event.currentTarget);
    if (
      strategy !== "active-descendant" ||
      event.target !== event.currentTarget ||
      readBoolean(options.isDisabled, "isDisabled")
    ) {
      return;
    }
    const key = effectiveKey();
    if (key !== null && registry.activeKey.value === null) {
      commit(
        () => {
          registry.setActiveKey(key);
        },
        "focus",
        event,
        false,
      );
    }
  };
  const onKeydown = (event: KeyboardEvent): void => {
    if (disposed) return;
    container = eventElement(event.currentTarget);
    if (
      event.defaultPrevented ||
      isEditableDescendant(event) ||
      readBoolean(options.isDisabled, "isDisabled")
    ) {
      return;
    }
    const intent = keyIntent(
      event,
      readOrientation(options.orientation),
      readDirection(options.direction),
    );
    if (intent && registry.navigableItems.value.length > 0) {
      event.preventDefault();
      navigate(intent, event);
      return;
    }
    typeahead?.typeaheadProps.onKeydown(event);
  };

  return Object.freeze({
    activeKey: registry.activeKey,
    typeahead,
    getContainerProps: () => {
      assertActive();
      const key = effectiveKey();
      const activeDescendant =
        strategy === "active-descendant" && key !== null ? itemId(getItem(key)) : undefined;
      const props: CompositeContainerProps = {
        onFocus,
        onKeydown,
        ...(strategy === "active-descendant" ? { tabindex: 0 as const } : {}),
        ...(activeDescendant !== undefined ? { "aria-activedescendant": activeDescendant } : {}),
      };
      return Object.freeze(props);
    },
    getItemProps: (key: Key) => {
      assertActive();
      const item = getItem(key);
      const handlers = handlersFor(key);
      const id = itemId(item);
      const props: CompositeItemProps = {
        ...handlers,
        ...(id ? { id } : {}),
        ...(strategy === "roving" ? { tabindex: effectiveKey() === key ? 0 : -1 } : {}),
      };
      return Object.freeze(props);
    },
    navigate,
    dispose: () => {
      if (disposed) return;
      disposed = true;
      typeahead?.dispose();
      itemHandlers.clear();
      container = null;
    },
  });
}

/** Create a composite adapter disposed with the current Vue effect scope. */
export function useCompositeNavigation<Key extends CollectionKey, Value>(
  options: CompositeNavigationOptions<Key, Value>,
): CompositeNavigationController<Key> {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createCompositeNavigation(options);
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  ActiveDescendantNavigationOptions,
  CompositeContainerProps,
  CompositeDirection,
  CompositeFocusStrategy,
  CompositeItemProps,
  CompositeNavigationBaseOptions,
  CompositeNavigationChange,
  CompositeNavigationCommand,
  CompositeNavigationController,
  CompositeNavigationIntent,
  CompositeNavigationOptions,
  CompositeOrientation,
  RovingNavigationOptions,
} from "./composite-navigation-types.ts";
