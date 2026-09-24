import { computed } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { menuContentContext, menuTreeContext } from "./menu-context.ts";
import type {
  MenuContentContextValue,
  MenuLevelContextValue,
  MenuTreeContextValue,
} from "./menu-context.ts";
import { isHoverPointer } from "./menu-dom.ts";
import type { MenuSelectEvent } from "./menu-types.ts";

/** ARIA role rendered by an item flavor. */
export type MenuItemRole = "menuitem" | "menuitemcheckbox" | "menuitemradio";

/** Static role, roving `tabindex`, and handlers bound onto an item element. */
export interface MenuItemInteractiveProps {
  readonly role: MenuItemRole;
  readonly tabindex: -1;
  readonly onClick: (event: MouseEvent) => void;
  readonly onFocus: () => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
  readonly onPointerleave: (event: PointerEvent) => void;
  readonly onPointermove: (event: PointerEvent) => void;
}

/** How an item activation was requested. */
export type MenuActivationSource = "imperative" | "keyboard" | "pointer";

/** Inputs for {@link useMenuItem}. */
export interface MenuItemOptions {
  readonly role: MenuItemRole;
  readonly element: Readonly<ShallowRef<HTMLDivElement | null>>;
  readonly hint: string;
  readonly disabled: () => boolean;
  readonly textValue: () => string | undefined;
  readonly subLevel?: MenuLevelContextValue;
  /** Activate the item; return whether an activation happened. */
  readonly activate: (event: Event | null, source: MenuActivationSource) => boolean;
  /** Item-specific keydown hook that runs before Enter/Space activation. */
  readonly keydown?: (event: KeyboardEvent) => void;
  /** Item-specific pointer-leave hook; return `true` to skip clearing the highlight. */
  readonly pointerleave?: (event: PointerEvent) => boolean;
  /** Item-specific pointer-move hook run after the highlight moved to this item. */
  readonly pointermove?: (event: PointerEvent) => void;
}

/** Registration, highlight, and handlers shared by every item flavor. */
export interface MenuItemController {
  readonly id: ComputedRef<string>;
  readonly tree: MenuTreeContextValue;
  readonly content: MenuContentContextValue;
  readonly highlighted: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly focus: () => void;
  /** Bind with `v-bind`; menu items take focus programmatically (`tabindex="-1"`). */
  readonly interactiveProps: MenuItemInteractiveProps;
}

/** Create a preventable {@link MenuSelectEvent}. */
export function createMenuSelectEvent(
  target: HTMLElement | null,
  originalEvent: Event | null,
): MenuSelectEvent {
  let prevented = false;
  return Object.freeze({
    type: "select",
    target,
    originalEvent,
    get defaultPrevented() {
      return prevented;
    },
    preventDefault: () => {
      prevented = true;
    },
  });
}

/**
 * Register one item with the nearest menu content and wire highlight,
 * pointer, and keyboard activation according to the WAI-ARIA APG menu pattern.
 */
export function useMenuItem(options: MenuItemOptions): MenuItemController {
  const tree = menuTreeContext.use();
  const content = menuContentContext.use();
  const id = useDeterministicId({ hint: options.hint });
  const key = id.value;
  const disabled = computed(() => options.disabled());
  content.registry.register({
    key,
    value: { subLevel: options.subLevel ?? null },
    element: options.element,
    textValue: options.textValue,
    disabled,
  });
  const highlighted = computed(() => content.registry.activeKey.value === key);

  function focus(): void {
    content.focusItem(key);
  }

  function onFocus(): void {
    content.registry.setActiveKey(key);
  }

  function onClick(event: MouseEvent): void {
    if (disabled.value) {
      event.preventDefault();
      return;
    }
    options.activate(event, "pointer");
  }

  function onKeydown(event: KeyboardEvent): void {
    options.keydown?.(event);
    if (event.defaultPrevented) return;
    const space = event.key === " " && content.typeaheadQuery.value.length === 0;
    if (event.key !== "Enter" && !space) return;
    event.preventDefault();
    if (!disabled.value) options.activate(event, "keyboard");
  }

  function onPointermove(event: PointerEvent): void {
    if (!isHoverPointer(event) || content.isPointerInGrace(event)) return;
    if (disabled.value) {
      if (content.registry.activeKey.value !== null) content.focusContent();
      return;
    }
    content.clearGrace();
    if (
      !highlighted.value ||
      options.element.value?.ownerDocument.activeElement !== options.element.value
    ) {
      focus();
    }
    options.pointermove?.(event);
  }

  function onPointerleave(event: PointerEvent): void {
    if (!isHoverPointer(event)) return;
    if (options.pointerleave?.(event) === true) return;
    if (content.isPointerInGrace(event)) return;
    if (highlighted.value) content.focusContent();
  }

  return {
    id,
    tree,
    content,
    highlighted,
    disabled,
    focus,
    interactiveProps: Object.freeze({
      role: options.role,
      tabindex: -1,
      onClick,
      onFocus,
      onKeydown,
      onPointerleave,
      onPointermove,
    }),
  };
}
