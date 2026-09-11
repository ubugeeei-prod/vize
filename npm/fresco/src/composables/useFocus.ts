/**
 * useFocus - Focus management composable
 */

import {
  computed,
  inject,
  isRef,
  onUnmounted,
  provide,
  ref,
  watch,
  type InjectionKey,
  type Ref,
} from "@vue/runtime-core";

export const FOCUS_KEY: InjectionKey<FocusManager> = Symbol("fresco-focus");

export interface UseFocusOptions {
  /** Enable or disable this focus target while keeping its logical identity */
  isActive?: boolean | Ref<boolean>;
  /** Auto-focus this component when nothing else is focused */
  autoFocus?: boolean;
  /** Focus ID for this element */
  id?: string;
  /** Parent focus ID used to derive descendant-focus state */
  parentId?: string | null | Ref<string | null | undefined>;
}

export interface FocusManager {
  /** Currently focused element ID */
  focusedId: Ref<string | null>;
  /** Ink-compatible currently focused ID alias */
  activeId: Readonly<Ref<string | undefined>>;
  /** All active focusable element IDs */
  focusableIds: Readonly<Ref<string[]>>;
  /** Whether focus management is enabled */
  isEnabled: Ref<boolean>;
  /** Enable focus management */
  enableFocus: () => void;
  /** Disable focus management */
  disableFocus: () => void;
  /** Focus a specific element */
  focus: (id: string) => void;
  /** Focus next element */
  focusNext: () => void;
  /** Focus previous element */
  focusPrevious: () => void;
  /** Register a focusable element */
  register: (
    id: string,
    options?: { isActive?: boolean; autoFocus?: boolean; parentId?: string | null | undefined },
  ) => void;
  /** Unregister a focusable element */
  unregister: (id: string) => void;
  /** Activate or deactivate a focusable element while keeping its order */
  setActive: (id: string, isActive: boolean) => void;
  /** Reparent a focusable element while keeping its registration order */
  setParent: (id: string, parentId: string | null | undefined) => void;
  /** Whether a registered descendant of the target currently owns focus */
  hasFocusedDescendant: (id: string) => boolean;
}

interface Focusable {
  id: string;
  isActive: boolean;
  parentId: string | null;
}

function activeFocusables(focusables: Focusable[]): Focusable[] {
  return focusables.filter((focusable) => focusable.isActive);
}

/**
 * Create a focus manager (use at app root)
 */
export function createFocusManager(): FocusManager {
  const focusedId = ref<string | null>(null);
  const activeId = computed(() => focusedId.value ?? undefined);
  const focusables = ref<Focusable[]>([]);
  const focusableIds = computed(() => activeFocusables(focusables.value).map(({ id }) => id));
  const isEnabled = ref(true);

  const focus = (id: string) => {
    const target = focusables.value.find((focusable) => focusable.id === id);
    if (isEnabled.value && target?.isActive) {
      focusedId.value = id;
    }
  };

  const focusNext = () => {
    const active = activeFocusables(focusables.value);
    if (!isEnabled.value || active.length === 0) return;

    const currentIndex =
      focusedId.value !== null
        ? focusables.value.findIndex((focusable) => focusable.id === focusedId.value)
        : -1;
    const next = focusables.value.slice(currentIndex + 1).find((focusable) => focusable.isActive);
    focusedId.value = next?.id ?? active[0]?.id ?? null;
  };

  const focusPrevious = () => {
    const active = activeFocusables(focusables.value);
    if (!isEnabled.value || active.length === 0) return;

    const currentIndex =
      focusedId.value !== null
        ? focusables.value.findIndex((focusable) => focusable.id === focusedId.value)
        : focusables.value.length;
    const previous = focusables.value
      .slice(0, currentIndex < 0 ? 0 : currentIndex)
      .findLast((focusable) => focusable.isActive);
    focusedId.value = previous?.id ?? active.at(-1)?.id ?? null;
  };

  const register: FocusManager["register"] = (id, options = {}) => {
    const existing = focusables.value.find((focusable) => focusable.id === id);
    if (existing) {
      existing.isActive = options.isActive ?? existing.isActive;
      if ("parentId" in options) {
        existing.parentId = options.parentId ?? null;
      }
    } else {
      focusables.value.push({
        id,
        isActive: options.isActive ?? true,
        parentId: options.parentId ?? null,
      });
    }

    if (options.autoFocus && focusedId.value === null && options.isActive !== false) {
      focus(id);
    }
  };

  const unregister = (id: string) => {
    const index = focusables.value.findIndex((focusable) => focusable.id === id);
    if (index !== -1) {
      focusables.value.splice(index, 1);
      if (focusedId.value === id) {
        focusedId.value = null;
      }
    }
  };

  const setActive = (id: string, isActive: boolean) => {
    const focusable = focusables.value.find((item) => item.id === id);
    if (!focusable) return;

    focusable.isActive = isActive;
    if (!isActive && focusedId.value === id) {
      focusedId.value = null;
    }
  };

  const setParent: FocusManager["setParent"] = (id, parentId) => {
    const focusable = focusables.value.find((item) => item.id === id);
    if (!focusable) return;

    focusable.parentId = parentId ?? null;
  };

  const hasFocusedDescendant: FocusManager["hasFocusedDescendant"] = (id) => {
    let currentId = focusedId.value;
    if (currentId === null || currentId === id) return false;

    const visited = new Set<string>();

    while (true) {
      if (visited.has(currentId)) return false;
      visited.add(currentId);
      const current = focusables.value.find((focusable) => focusable.id === currentId);
      const parentId = current?.parentId ?? null;
      if (parentId === null) return false;
      if (parentId === id) return true;
      currentId = parentId;
    }
  };

  const enableFocus = () => {
    isEnabled.value = true;
  };

  const disableFocus = () => {
    isEnabled.value = false;
    focusedId.value = null;
  };

  return {
    focusedId,
    activeId,
    focusableIds,
    isEnabled,
    enableFocus,
    disableFocus,
    focus,
    focusNext,
    focusPrevious,
    register,
    unregister,
    setActive,
    setParent,
    hasFocusedDescendant,
  };
}

/**
 * Provide focus manager to descendants
 */
export function provideFocusManager(manager: FocusManager) {
  provide(FOCUS_KEY, manager);
}

/**
 * Use focus management
 */
export function useFocus(options: UseFocusOptions = {}) {
  const {
    autoFocus = false,
    id = `focus-${Math.random().toString(36).slice(2)}`,
    isActive: isActiveOption = true,
    parentId: parentIdOption = null,
  } = options;

  const manager = inject(FOCUS_KEY, null);
  const localFocused = ref(autoFocus);
  const active = isRef(isActiveOption) ? isActiveOption : ref(isActiveOption);
  const parentId = isRef(parentIdOption) ? parentIdOption : ref(parentIdOption);

  const isFocused = computed(() => {
    if (manager) {
      return manager.isEnabled.value && manager.focusedId.value === id;
    }
    return active.value && localFocused.value;
  });
  const hasFocusedDescendant = computed(() => (manager ? manager.hasFocusedDescendant(id) : false));

  const focus = (targetId = id) => {
    if (manager) {
      manager.focus(targetId);
    } else if (targetId === id) {
      localFocused.value = true;
    }
  };

  const blur = () => {
    if (manager) {
      if (manager.focusedId.value === id) {
        manager.focusedId.value = null;
      }
    } else {
      localFocused.value = false;
    }
  };

  if (manager) {
    manager.register(id, { isActive: active.value, autoFocus, parentId: parentId.value });

    watch(
      active,
      (enabled) => {
        manager.setActive(id, enabled);
        if (enabled && autoFocus && !manager.focusedId.value) manager.focus(id);
      },
      { immediate: false },
    );
    watch(
      parentId,
      (nextParentId) => {
        manager.setParent(id, nextParentId);
      },
      { immediate: false },
    );

    onUnmounted(() => {
      manager.unregister(id);
    });
  }

  return {
    id,
    isFocused,
    hasFocusedDescendant,
    focus,
    blur,
  };
}

/**
 * Use global focus manager controls.
 */
export function useFocusManager() {
  const manager = inject(FOCUS_KEY, null);

  if (!manager) {
    const empty = ref<string[]>([]);
    const activeId = ref<string | undefined>(undefined);
    return {
      enableFocus: () => {},
      disableFocus: () => {},
      focusNext: () => {},
      focusPrevious: () => {},
      focus: (_id: string) => {},
      hasFocusedDescendant: (_id: string) => false,
      activeId,
      focusableIds: empty,
    };
  }

  return {
    enableFocus: manager.enableFocus,
    disableFocus: manager.disableFocus,
    focusNext: manager.focusNext,
    focusPrevious: manager.focusPrevious,
    focus: manager.focus,
    hasFocusedDescendant: manager.hasFocusedDescendant,
    activeId: manager.activeId,
    focusableIds: manager.focusableIds,
  };
}
