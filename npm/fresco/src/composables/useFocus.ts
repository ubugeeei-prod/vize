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
} from "vue";

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
  /** Descendant that receives focus when this target is focused */
  delegateFocusTo?: string | null | Ref<string | null | undefined>;
  /** Keep programmatic and sequential focus inside this target subtree */
  trapFocus?: boolean | Ref<boolean>;
}

/** Registration state retained for one focusable target. */
export interface FocusRegistrationOptions {
  /** Whether the target participates in focus traversal. @default true */
  isActive?: boolean;
  /** Focus the target when no target currently owns focus. @default false */
  autoFocus?: boolean;
  /** Registered parent used for descendant and trap boundaries. @default null */
  parentId?: string | null | undefined;
  /** Registered descendant that receives delegated focus. @default null */
  delegateFocusTo?: string | null | undefined;
}

export interface FocusManager {
  /** Currently focused element ID */
  focusedId: Ref<string | null>;
  /** Ink-compatible currently focused ID alias */
  activeId: Readonly<Ref<string | undefined>>;
  /** All active focusable element IDs */
  focusableIds: Readonly<Ref<string[]>>;
  /** Innermost active focus-trap root, when present */
  activeTrapId: Readonly<Ref<string | undefined>>;
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
  register: (id: string, options?: FocusRegistrationOptions) => void;
  /** Unregister a focusable element */
  unregister: (id: string) => void;
  /** Activate or deactivate a focusable element while keeping its order */
  setActive: (id: string, isActive: boolean) => void;
  /** Reparent a focusable element while keeping its registration order */
  setParent: (id: string, parentId: string | null | undefined) => void;
  /** Change the descendant that receives delegated focus */
  setDelegate: (id: string, delegateFocusTo: string | null | undefined) => void;
  /** Activate a nested focus boundary rooted at a registered target */
  trapFocus: (id: string) => void;
  /** Release a focus boundary and every nested boundary that depends on it */
  releaseFocusTrap: (id: string) => void;
  /** Whether a registered descendant of the target currently owns focus */
  hasFocusedDescendant: (id: string) => boolean;
}

interface Focusable {
  id: string;
  isActive: boolean;
  parentId: string | null;
  delegateFocusTo: string | null;
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
  const trapStack = ref<string[]>([]);
  const activeTrapId = computed(() => trapStack.value.at(-1));
  const isEnabled = ref(true);

  const findFocusable = (id: string) => focusables.value.find((focusable) => focusable.id === id);

  const isWithin = (id: string, rootId: string) => {
    let currentId: string | null = id;
    const visited = new Set<string>();
    while (currentId !== null) {
      if (currentId === rootId) return true;
      if (visited.has(currentId)) return false;
      visited.add(currentId);
      currentId = findFocusable(currentId)?.parentId ?? null;
    }
    return false;
  };

  const allowedByActiveTrap = (id: string) => {
    const trapId = activeTrapId.value;
    return trapId === undefined || isWithin(id, trapId);
  };

  const scopedFocusables = () =>
    activeFocusables(focusables.value).filter(({ id }) => allowedByActiveTrap(id));

  const delegatedTarget = (target: Focusable) => {
    let current = target;
    const visited = new Set<string>();
    while (current.delegateFocusTo !== null && !visited.has(current.id)) {
      visited.add(current.id);
      const delegated = findFocusable(current.delegateFocusTo);
      if (
        !delegated?.isActive ||
        !isWithin(delegated.id, current.id) ||
        !allowedByActiveTrap(delegated.id)
      ) {
        break;
      }
      current = delegated;
    }
    return current;
  };

  const focus = (id: string) => {
    const target = findFocusable(id);
    if (isEnabled.value && target?.isActive && allowedByActiveTrap(id)) {
      focusedId.value = delegatedTarget(target).id;
    }
  };

  const moveFocus = (offset: -1 | 1) => {
    const active = scopedFocusables();
    if (!isEnabled.value || active.length === 0) return;

    const currentIndex = active.findIndex((focusable) => focusable.id === focusedId.value);
    for (let distance = 1; distance <= active.length; distance += 1) {
      const index =
        currentIndex === -1
          ? offset === 1
            ? distance - 1
            : active.length - distance
          : (currentIndex + offset * distance + active.length * 2) % active.length;
      const candidate = active[index];
      if (!candidate) continue;
      const resolved = delegatedTarget(candidate);
      if (resolved.id !== focusedId.value || active.length === 1) {
        focusedId.value = resolved.id;
        return;
      }
    }
  };

  const focusNext = () => moveFocus(1);

  const focusPrevious = () => moveFocus(-1);

  const register: FocusManager["register"] = (id, options = {}) => {
    const existing = focusables.value.find((focusable) => focusable.id === id);
    if (existing) {
      existing.isActive = options.isActive ?? existing.isActive;
      if ("parentId" in options) {
        existing.parentId = options.parentId ?? null;
      }
      if ("delegateFocusTo" in options) {
        existing.delegateFocusTo = options.delegateFocusTo ?? null;
      }
    } else {
      focusables.value.push({
        id,
        isActive: options.isActive ?? true,
        parentId: options.parentId ?? null,
        delegateFocusTo: options.delegateFocusTo ?? null,
      });
    }

    if (options.autoFocus && focusedId.value === null && options.isActive !== false) {
      focus(id);
    }
  };

  const unregister = (id: string) => {
    const index = focusables.value.findIndex((focusable) => focusable.id === id);
    if (index !== -1) {
      const trapIndex = trapStack.value.indexOf(id);
      if (trapIndex !== -1) trapStack.value.splice(trapIndex);
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
    if (!isActive) {
      const trapIndex = trapStack.value.indexOf(id);
      if (trapIndex !== -1) trapStack.value.splice(trapIndex);
    }
    if (!isActive && focusedId.value === id) {
      focusedId.value = null;
    }
  };

  const setParent: FocusManager["setParent"] = (id, parentId) => {
    const focusable = findFocusable(id);
    if (!focusable) return;

    focusable.parentId = parentId ?? null;
    if (focusedId.value !== null && !allowedByActiveTrap(focusedId.value)) {
      focusedId.value = null;
    }
  };

  const setDelegate: FocusManager["setDelegate"] = (id, delegateFocusTo) => {
    const focusable = findFocusable(id);
    if (focusable) focusable.delegateFocusTo = delegateFocusTo ?? null;
  };

  const trapFocus: FocusManager["trapFocus"] = (id) => {
    const target = findFocusable(id);
    if (!target?.isActive || !allowedByActiveTrap(id)) return;
    if (activeTrapId.value !== id) trapStack.value.push(id);
    if (focusedId.value === null || !isWithin(focusedId.value, id)) focus(id);
  };

  const releaseFocusTrap: FocusManager["releaseFocusTrap"] = (id) => {
    const index = trapStack.value.indexOf(id);
    if (index === -1) return;
    trapStack.value.splice(index);
    const remaining = activeTrapId.value;
    if (
      remaining !== undefined &&
      focusedId.value !== null &&
      !isWithin(focusedId.value, remaining)
    ) {
      focus(remaining);
    }
  };

  const hasFocusedDescendant: FocusManager["hasFocusedDescendant"] = (id) => {
    let currentId = focusedId.value;
    if (currentId === null || currentId === id) return false;

    const visited = new Set<string>();

    while (true) {
      if (visited.has(currentId)) return false;
      visited.add(currentId);
      const current = findFocusable(currentId);
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
    activeTrapId,
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
    setDelegate,
    trapFocus,
    releaseFocusTrap,
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
    delegateFocusTo: delegateFocusToOption = null,
    trapFocus: trapFocusOption = false,
  } = options;

  const manager = inject(FOCUS_KEY, null);
  const localFocused = ref(autoFocus);
  const active = isRef(isActiveOption) ? isActiveOption : ref(isActiveOption);
  const parentId = isRef(parentIdOption) ? parentIdOption : ref(parentIdOption);
  const delegateFocusTo = isRef(delegateFocusToOption)
    ? delegateFocusToOption
    : ref(delegateFocusToOption);
  const trapsFocus = isRef(trapFocusOption) ? trapFocusOption : ref(trapFocusOption);

  const isFocused = computed(() => {
    if (manager) {
      return manager.isEnabled.value && manager.focusedId.value === id;
    }
    return active.value && localFocused.value;
  });
  const hasFocusedDescendant = computed(() => (manager ? manager.hasFocusedDescendant(id) : false));
  const isFocusTrapActive = computed(() => manager?.activeTrapId.value === id);

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
    manager.register(id, {
      isActive: active.value,
      autoFocus,
      parentId: parentId.value,
      delegateFocusTo: delegateFocusTo.value,
    });
    if (trapsFocus.value) manager.trapFocus(id);

    watch(
      active,
      (enabled) => {
        manager.setActive(id, enabled);
        if (enabled && trapsFocus.value) manager.trapFocus(id);
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
    watch(
      delegateFocusTo,
      (nextDelegate) => {
        manager.setDelegate(id, nextDelegate);
      },
      { immediate: false },
    );
    watch(
      trapsFocus,
      (trapped) => {
        if (trapped) manager.trapFocus(id);
        else manager.releaseFocusTrap(id);
      },
      { immediate: false },
    );

    onUnmounted(() => {
      manager.releaseFocusTrap(id);
      manager.unregister(id);
    });
  }

  return {
    id,
    isFocused,
    hasFocusedDescendant,
    isFocusTrapActive,
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
      trapFocus: (_id: string) => {},
      releaseFocusTrap: (_id: string) => {},
      setDelegate: (_id: string, _delegateFocusTo: string | null | undefined) => {},
      activeId,
      focusableIds: empty,
      activeTrapId: activeId,
    };
  }

  return {
    enableFocus: manager.enableFocus,
    disableFocus: manager.disableFocus,
    focusNext: manager.focusNext,
    focusPrevious: manager.focusPrevious,
    focus: manager.focus,
    hasFocusedDescendant: manager.hasFocusedDescendant,
    trapFocus: manager.trapFocus,
    releaseFocusTrap: manager.releaseFocusTrap,
    setDelegate: manager.setDelegate,
    activeId: manager.activeId,
    focusableIds: manager.focusableIds,
    activeTrapId: manager.activeTrapId,
  };
}
