import { computed, shallowRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import type { PositionerElement } from "../../overlays/positioner/positioner.ts";
import { menuLevelContext, menuTreeContext } from "./menu-context.ts";
import type {
  MenuEdgeDirection,
  MenuLevelContextValue,
  MenuTreeContextValue,
} from "./menu-context.ts";
import type {
  MenuDirection,
  MenuEntryFocus,
  MenuKind,
  MenuRootExpose,
  MenuSlotState,
  MenuState,
} from "./menu-types.ts";

/** Inputs for {@link createMenuLevel}. */
export interface MenuLevelOptions {
  readonly parent: MenuLevelContextValue | null;
  readonly baseId: ComputedRef<string>;
  readonly requestedOpen: ComputedRef<boolean>;
  readonly commitOpen: (value: boolean) => boolean;
  readonly disabled: () => boolean;
  readonly reference?: (() => PositionerElement | null) | undefined;
  readonly restoreTarget?: (() => HTMLElement | null) | undefined;
  readonly onOpenChange: (value: boolean, previous: boolean, event: Event | null) => void;
}

/** Create one open/closed menu level whose visibility is gated on its parent. */
export function createMenuLevel(options: MenuLevelOptions): MenuLevelContextValue {
  const { parent } = options;
  const triggerElement = shallowRef<HTMLElement | null>(null);
  const contentElement = shallowRef<HTMLDivElement | null>(null);
  const entryFocus = shallowRef<MenuEntryFocus>("content");
  const triggerItemKey = shallowRef<string | null>(null);
  const skipFocusReturn = shallowRef(false);
  const focusContentEdge = shallowRef<((edge: "first" | "last") => HTMLElement | null) | null>(
    null,
  );
  const parentOpen = () => parent?.open.value ?? true;
  const open = computed(() => options.requestedOpen.value && !options.disabled() && parentOpen());
  const state = computed<MenuState>(() => (open.value ? "open" : "closed"));
  const reference = computed<PositionerElement | null>(() =>
    options.reference ? options.reference() : triggerElement.value,
  );

  function setOpen(
    value: boolean,
    event: Event | null = null,
    entry: MenuEntryFocus = "content",
  ): boolean {
    const previous = open.value;
    const next = value && !options.disabled() && parentOpen();
    if (next) {
      entryFocus.value = entry;
      skipFocusReturn.value = false;
    }
    const changed = options.commitOpen(next);
    if (changed || previous !== next) options.onOpenChange(next, previous, event);
    return changed || previous !== next;
  }

  if (parent) {
    watch(
      () => parent.open.value,
      (next) => {
        if (!next && options.requestedOpen.value) {
          options.commitOpen(false);
          options.onOpenChange(false, true, null);
        }
      },
      { flush: "sync" },
    );
  }

  return {
    parent,
    id: options.baseId,
    triggerId: computed(() => deriveDeterministicId(options.baseId.value, "trigger")),
    contentId: computed(() => deriveDeterministicId(options.baseId.value, "content")),
    open,
    state,
    disabled: computed(() => options.disabled()),
    hasTrigger: shallowRef(false),
    triggerElement,
    reference,
    contentElement,
    entryFocus,
    triggerItemKey,
    focusContentEdge,
    skipFocusReturn,
    setOpen,
    restoreTarget: () => (options.restoreTarget ? options.restoreTarget() : triggerElement.value),
  };
}

/** Reactive inputs for {@link useMenuRoot}. */
export interface MenuRootOptions {
  readonly kind: MenuKind;
  readonly hint: string;
  readonly id: () => string | null | undefined;
  readonly open: () => boolean | undefined;
  readonly defaultOpen: () => boolean;
  readonly modal: () => boolean;
  readonly dir: () => MenuDirection;
  readonly loop: () => boolean;
  readonly disabled?: () => boolean;
  readonly reference?: () => PositionerElement | null;
  readonly restoreTarget?: () => HTMLElement | null;
  readonly edgeNavigate?: (direction: MenuEdgeDirection, event: KeyboardEvent) => boolean;
  readonly onOpenChange: (value: boolean, previous: boolean, event: Event | null) => void;
}

/** Contexts and public state created by {@link useMenuRoot}. */
export interface MenuRootController {
  readonly tree: MenuTreeContextValue;
  readonly level: MenuLevelContextValue;
  readonly slotState: ComputedRef<MenuSlotState>;
  readonly expose: MenuRootSetupExpose;
}

/** Ref-carrying expose object; Vue unwraps it to {@link MenuRootExpose}. */
export type MenuRootSetupExpose = Omit<
  MenuRootExpose,
  "contentId" | "dir" | "id" | "modal" | "open" | "state" | "triggerId"
> & {
  readonly contentId: ComputedRef<string>;
  readonly dir: ComputedRef<MenuDirection>;
  readonly id: ComputedRef<string>;
  readonly modal: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<MenuState>;
  readonly triggerId: ComputedRef<string>;
};

/**
 * Create and provide the root level and tree contexts of one menu.
 *
 * Every root-level SFC (MenuRoot, ContextMenuRoot, MenubarMenu) calls this in
 * setup so all menu surfaces share one controlled/uncontrolled contract.
 */
export function useMenuRoot(options: MenuRootOptions): MenuRootController {
  const openState = useControllableState({
    value: options.open,
    defaultValue: options.defaultOpen,
  });
  const baseId = useDeterministicId({ id: options.id, hint: options.hint });
  const disabled = () => options.disabled?.() ?? false;
  const level = createMenuLevel({
    parent: null,
    baseId,
    requestedOpen: computed(() => openState.value.value),
    commitOpen: (value) => openState.set(value),
    disabled,
    reference: options.reference,
    restoreTarget: options.restoreTarget,
    onOpenChange: options.onOpenChange,
  });
  const modal = computed(() => options.modal());
  const dir = computed(() => options.dir());
  const loop = computed(() => options.loop());

  watch(
    disabled,
    (next) => {
      if (next && openState.value.value) {
        openState.set(false);
        options.onOpenChange(false, true, null);
      }
    },
    { flush: "sync" },
  );

  const tree: MenuTreeContextValue = {
    kind: options.kind,
    root: level,
    modal,
    dir,
    loop,
    layers: shallowRef<readonly HTMLElement[]>([]),
    closeAll: (event = null) => level.setOpen(false, event),
    edgeNavigate: (direction, event) => options.edgeNavigate?.(direction, event) ?? false,
  };
  menuTreeContext.provide(tree);
  menuLevelContext.provide(level);

  const slotState = computed<MenuSlotState>(() => ({
    dir: dir.value,
    modal: modal.value,
    open: level.open.value,
    state: level.state.value,
  }));

  const expose: MenuRootSetupExpose = {
    close: (event = null) => level.setOpen(false, event),
    contentId: level.contentId,
    dir,
    id: baseId,
    modal,
    open: level.open,
    openMenu: (entry = "first", event = null) => level.setOpen(true, event, entry),
    setOpen: (value, event = null) => level.setOpen(value, event, "first"),
    state: level.state,
    toggle: (event = null) => level.setOpen(!level.open.value, event, "first"),
    triggerId: level.triggerId,
  };

  return { tree, level, slotState, expose };
}
