import {
  getCurrentScope,
  isRef,
  onScopeDispose,
  shallowReadonly,
  shallowRef,
  toValue,
  watch,
} from "vue";

import {
  chordFromEvent,
  isModifierOnlyEvent,
  readPlatform,
  serializeShortcut,
  toShortcutSequence,
} from "./shortcut-parse.ts";
import {
  collectConflicts,
  isEditableEventTarget,
  readOptionalBoolean,
  routeChord,
  toBindingInfo,
  validateBindingStatics,
  type InternalShortcutBinding,
} from "./shortcut-registry.ts";
import type {
  ShortcutBindingOptions,
  ShortcutMatch,
  ShortcutRegistry,
  ShortcutRegistryOptions,
  ShortcutSequence,
} from "./shortcut-types.ts";

const disposedDiagnostic = "VIZE_UI_SHORTCUT_DISPOSED";
const setupDiagnostic = "VIZE_UI_SHORTCUT_SETUP";
const optionDiagnostic = "VIZE_UI_SHORTCUT_OPTION";
const inputDiagnostic = "VIZE_UI_SHORTCUT_INPUT";
const emptySequence: ShortcutSequence = Object.freeze([]);
const emptyScopes: readonly string[] = Object.freeze([]);

function defaultDocument(): Document | null {
  return typeof globalThis.document === "undefined" ? null : globalThis.document;
}

function readTimeout(source: ShortcutRegistryOptions["sequenceTimeout"]): number {
  const value = toValue(source) ?? 1000;
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    throw new TypeError(
      `${optionDiagnostic}: sequenceTimeout must resolve to a finite number >= 0`,
    );
  }
  return value;
}

function readDisabled(source: ShortcutRegistryOptions["isDisabled"]): boolean {
  const value = toValue(source);
  if (value === undefined) return false;
  if (typeof value !== "boolean") {
    throw new TypeError(`${optionDiagnostic}: isDisabled must resolve to a boolean`);
  }
  return value;
}

function toEventTarget(value: EventTarget | null | undefined): EventTarget | null {
  if (value === undefined) return defaultDocument();
  if (value === null) return null;
  if (
    typeof value.addEventListener !== "function" ||
    typeof value.removeEventListener !== "function"
  ) {
    throw new TypeError(`${optionDiagnostic}: target must resolve to an EventTarget or null`);
  }
  return value;
}

/**
 * Create an SSR-safe, scope-aware keyboard shortcut registry.
 *
 * The registry resolves `Mod` for one platform, routes chords and multi-chord
 * sequences to the winning binding, and never touches the DOM on the server:
 * with no ambient document it stays detached until a target appears or events
 * are fed through {@link ShortcutRegistry.input}. Call
 * {@link ShortcutRegistry.dispose} when using this factory outside a Vue
 * effect scope.
 */
export function createShortcutRegistry(options: ShortcutRegistryOptions = {}): ShortcutRegistry {
  const platform = readPlatform(options.platform);
  const capture = readOptionalBoolean(options.capture, "capture");
  readTimeout(options.sequenceTimeout);
  readDisabled(options.isDisabled);

  const pendingSequence = shallowRef<ShortcutSequence>(emptySequence);
  const activeScopes = shallowRef<readonly string[]>(emptyScopes);
  const bindings = new Map<symbol, InternalShortcutBinding>();
  const scopeStack: { readonly name: string; readonly token: symbol }[] = [];
  const detachers = new Set<() => void>();
  let timer: ReturnType<typeof setTimeout> | null = null;
  let order = 0;
  let disposed = false;

  const assertActive = () => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the registry has been disposed`);
  };
  const clearTimer = () => {
    if (timer !== null) clearTimeout(timer);
    timer = null;
  };
  const reset = (): boolean => {
    clearTimer();
    if (pendingSequence.value.length === 0) return false;
    pendingSequence.value = emptySequence;
    return true;
  };
  const scheduleReset = (timeout: number) => {
    clearTimer();
    timer = setTimeout(() => {
      pendingSequence.value = emptySequence;
      timer = null;
    }, timeout);
  };

  const route = (event: KeyboardEvent, pending: ShortcutSequence): boolean => {
    const chord = chordFromEvent(event);
    const next = Object.freeze([...pending, chord]);
    let decision;
    try {
      decision = routeChord(bindings.values(), activeScopes.value, next, {
        isEditable: isEditableEventTarget(event),
        isRepeat: event.repeat === true,
      });
    } catch (error) {
      reset();
      throw error;
    }
    if (decision.type === "match") {
      reset();
      if (decision.binding.preventDefault) event.preventDefault();
      const match: ShortcutMatch = Object.freeze({
        shortcut: decision.binding.sequence,
        scope: decision.binding.scope,
        description: decision.binding.description,
        originalEvent: event,
      });
      decision.binding.handler(match);
      return true;
    }
    if (decision.type === "pending") {
      pendingSequence.value = next;
      scheduleReset(readTimeout(options.sequenceTimeout));
      if (decision.preventDefault) event.preventDefault();
      return false;
    }
    if (pending.length > 0) {
      reset();
      return route(event, emptySequence);
    }
    return false;
  };

  const input = (event: KeyboardEvent): boolean => {
    assertActive();
    if (!event || typeof event.key !== "string") {
      throw new TypeError(`${inputDiagnostic}: input requires a keyboard event`);
    }
    let disabled: boolean;
    try {
      disabled = readDisabled(options.isDisabled);
    } catch (error) {
      reset();
      throw error;
    }
    if (disabled) {
      reset();
      return false;
    }
    if (event.isComposing === true || isModifierOnlyEvent(event)) return false;
    return route(event, pendingSequence.value);
  };

  const nativeListener = (event: Event) => {
    input(event as KeyboardEvent);
  };
  const attach = (target: EventTarget, attachOptions: { readonly capture?: boolean } = {}) => {
    assertActive();
    const resolved = toEventTarget(target ?? null);
    if (resolved === null) {
      throw new TypeError(`${optionDiagnostic}: attach requires an EventTarget`);
    }
    const useCapture = attachOptions.capture ?? capture;
    resolved.addEventListener("keydown", nativeListener, useCapture);
    let detached = false;
    const detach = () => {
      if (detached) return;
      detached = true;
      detachers.delete(detach);
      resolved.removeEventListener("keydown", nativeListener, useCapture);
    };
    detachers.add(detach);
    return detach;
  };

  let detachPrimary: (() => void) | null = null;
  const attachPrimary = () => {
    detachPrimary?.();
    detachPrimary = null;
    const target = toEventTarget(toValue(options.target));
    if (target !== null) detachPrimary = attach(target);
  };
  const stopWatches: Array<() => void> = [];
  const targetSource = options.target;
  if (isRef(targetSource) || typeof targetSource === "function") {
    stopWatches.push(watch(() => toValue(targetSource), attachPrimary, { flush: "sync" }));
  }
  attachPrimary();

  const disabledSource = options.isDisabled;
  if (isRef(disabledSource) || typeof disabledSource === "function") {
    stopWatches.push(
      watch(
        () => readDisabled(disabledSource),
        (value) => {
          if (value) reset();
        },
        { flush: "sync" },
      ),
    );
  }

  return Object.freeze({
    pendingSequence: shallowReadonly(pendingSequence),
    activeScopes: shallowReadonly(activeScopes),
    shortcutProps: Object.freeze({
      onKeydown(event: KeyboardEvent) {
        input(event);
      },
    }),
    register(binding: ShortcutBindingOptions) {
      assertActive();
      validateBindingStatics(binding);
      const key = Symbol("vize-ui-shortcut-binding");
      const sequence = toShortcutSequence(binding.shortcut, platform);
      bindings.set(key, {
        sequence,
        sequenceId: serializeShortcut(sequence),
        scope: binding.scope ?? "global",
        handler: binding.handler,
        when: binding.when,
        preventDefault: binding.preventDefault ?? true,
        allowRepeat: readOptionalBoolean(binding.allowRepeat, "allowRepeat"),
        allowInEditable: readOptionalBoolean(binding.allowInEditable, "allowInEditable"),
        description: binding.description ?? null,
        order: order++,
      });
      return () => {
        bindings.delete(key);
      };
    },
    activateScope(scope: string) {
      assertActive();
      if (typeof scope !== "string" || scope === "" || scope === "global") {
        throw new TypeError(`${optionDiagnostic}: scope must be a non-empty, non-global name`);
      }
      const token = Symbol("vize-ui-shortcut-scope");
      scopeStack.push({ name: scope, token });
      activeScopes.value = Object.freeze(scopeStack.map((entry) => entry.name));
      return () => {
        const index = scopeStack.findIndex((entry) => entry.token === token);
        if (index === -1) return;
        scopeStack.splice(index, 1);
        activeScopes.value = Object.freeze(scopeStack.map((entry) => entry.name));
      };
    },
    input,
    attach,
    reset: () => {
      assertActive();
      return reset();
    },
    getBindings: () => Object.freeze([...bindings.values()].map(toBindingInfo)),
    getConflicts: () => collectConflicts(bindings.values()),
    dispose: () => {
      if (disposed) return;
      disposed = true;
      for (const stop of stopWatches) stop();
      for (const detach of Array.from(detachers)) detach();
      detachPrimary = null;
      clearTimer();
      pendingSequence.value = emptySequence;
      bindings.clear();
    },
  });
}

/** Create a shortcut registry disposed with the current Vue effect scope. */
export function useShortcutRegistry(options: ShortcutRegistryOptions = {}): ShortcutRegistry {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const registry = createShortcutRegistry(options);
  onScopeDispose(registry.dispose);
  return registry;
}

export { detectShortcutPlatform, parseShortcut, serializeShortcut } from "./shortcut-parse.ts";
export { formatShortcut, getShortcutKeycaps } from "./shortcut-format.ts";
export type {
  ShortcutBindingInfo,
  ShortcutBindingOptions,
  ShortcutChord,
  ShortcutConflict,
  ShortcutFormatOptions,
  ShortcutMatch,
  ShortcutParseOptions,
  ShortcutPlatform,
  ShortcutProps,
  ShortcutRegistry,
  ShortcutRegistryOptions,
  ShortcutSequence,
} from "./shortcut-types.ts";
