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
}

export interface FocusManager {
  /** Currently focused element ID */
  focusedId: Ref<string | null>;
  /** Ink-compatible currently focused ID alias */
  activeId: Readonly<Ref<string | undefined>>;
  /** All active focusable element IDs */
  focusableIds: Ref<string[]>;
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
  register: (id: string) => void;
  /** Unregister a focusable element */
  unregister: (id: string) => void;
}

function normalizeIndex(index: number, length: number): number {
  return ((index % length) + length) % length;
}

/**
 * Create a focus manager (use at app root)
 */
export function createFocusManager(): FocusManager {
  const focusedId = ref<string | null>(null);
  const activeId = computed(() => focusedId.value ?? undefined);
  const focusableIds = ref<string[]>([]);
  const isEnabled = ref(true);

  const focus = (id: string) => {
    if (isEnabled.value && focusableIds.value.includes(id)) {
      focusedId.value = id;
    }
  };

  const focusNext = () => {
    if (!isEnabled.value || focusableIds.value.length === 0) return;

    const currentIndex = focusedId.value ? focusableIds.value.indexOf(focusedId.value) : -1;
    focusedId.value =
      focusableIds.value[normalizeIndex(currentIndex + 1, focusableIds.value.length)];
  };

  const focusPrevious = () => {
    if (!isEnabled.value || focusableIds.value.length === 0) return;

    const currentIndex = focusedId.value
      ? focusableIds.value.indexOf(focusedId.value)
      : focusableIds.value.length;
    focusedId.value =
      focusableIds.value[normalizeIndex(currentIndex - 1, focusableIds.value.length)];
  };

  const register = (id: string) => {
    if (!focusableIds.value.includes(id)) {
      focusableIds.value.push(id);
    }
  };

  const unregister = (id: string) => {
    const index = focusableIds.value.indexOf(id);
    if (index !== -1) {
      focusableIds.value.splice(index, 1);
      if (focusedId.value === id) {
        focusedId.value = focusableIds.value[0] ?? null;
      }
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
  } = options;

  const manager = inject(FOCUS_KEY, null);
  const localFocused = ref(autoFocus);
  const active = isRef(isActiveOption) ? isActiveOption : ref(isActiveOption);

  const isFocused = computed(() => {
    if (manager) {
      return manager.isEnabled.value && manager.focusedId.value === id;
    }
    return active.value && localFocused.value;
  });

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
    watch(
      active,
      (enabled) => {
        if (enabled) {
          manager.register(id);
          if (autoFocus && !manager.focusedId.value) {
            manager.focus(id);
          }
        } else {
          manager.unregister(id);
        }
      },
      { immediate: true },
    );

    onUnmounted(() => {
      manager.unregister(id);
    });
  }

  return {
    id,
    isFocused,
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
    activeId: manager.activeId,
    focusableIds: manager.focusableIds,
  };
}
