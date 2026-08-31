import { getCurrentScope, isRef, onScopeDispose, shallowReadonly, shallowRef, watch } from "vue";

import {
  isCharacterKey,
  readBoolean,
  readGrapheme,
  readTimeout,
  splitGraphemes,
  validateOptions,
} from "./typeahead-internal.ts";
import type {
  TypeaheadController,
  TypeaheadMatch,
  TypeaheadOptions,
  TypeaheadProps,
} from "./typeahead-types.ts";
import type { CollectionKey } from "../../foundations/collection/collection.ts";

const disposedDiagnostic = "VIZE_UI_TYPEAHEAD_DISPOSED";
const setupDiagnostic = "VIZE_UI_TYPEAHEAD_SETUP";

/** Create an SSR-safe, locale-aware typeahead buffer for one collection. */
export function createTypeahead<Key extends CollectionKey, Value>(
  options: TypeaheadOptions<Key, Value>,
): TypeaheadController<Key> {
  validateOptions(options);
  const query = shallowRef("");
  const collator =
    options.collator ?? new Intl.Collator(undefined, { sensitivity: "base", usage: "search" });
  let timer: ReturnType<typeof setTimeout> | null = null;
  let disposed = false;

  const clearTimer = () => {
    if (timer !== null) clearTimeout(timer);
    timer = null;
  };
  const reset = (): boolean => {
    clearTimer();
    if (query.value.length === 0) return false;
    query.value = "";
    return true;
  };
  const assertActive = () => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };
  const scheduleReset = (timeout: number) => {
    clearTimer();
    timer = setTimeout(() => {
      query.value = "";
      timer = null;
    }, timeout);
  };
  const input = (value: string, originalEvent: KeyboardEvent | null = null): Key | null => {
    assertActive();
    const grapheme = readGrapheme(value);
    let timeout: number;
    try {
      if (readBoolean(options.isDisabled, "isDisabled")) {
        reset();
        return null;
      }
      timeout = readTimeout(options.timeout);
    } catch (error) {
      reset();
      throw error;
    }
    const previousGraphemes = splitGraphemes(query.value);
    const isRepeated =
      previousGraphemes.length > 0 &&
      previousGraphemes.every((part) => collator.compare(part, grapheme) === 0);
    const nextQuery = isRepeated ? grapheme : `${query.value}${grapheme}`;
    query.value = nextQuery;
    scheduleReset(timeout);

    const previousKey = options.registry.activeKey.value;
    let key: Key | null;
    try {
      key = options.registry.moveActiveByTextValue(nextQuery, { collator });
    } catch (error) {
      reset();
      throw error;
    }
    if (key !== null && key !== previousKey) {
      const match: TypeaheadMatch<Key> = Object.freeze({
        key,
        previousKey,
        query: nextQuery,
        originalEvent,
      });
      options.onMatch?.(match);
    }
    return key;
  };

  const typeaheadProps: Readonly<TypeaheadProps> = Object.freeze({
    onKeydown(event: KeyboardEvent) {
      if (!isCharacterKey(event)) return;
      if (event.key === " " && query.value.length === 0 && options.allowSpace !== true) return;
      event.preventDefault();
      input(event.key, event);
    },
  });

  const stopWatches: Array<() => void> = [];
  const disabledSource = options.isDisabled;
  if (isRef(disabledSource) || typeof disabledSource === "function") {
    stopWatches.push(
      watch(
        () => {
          try {
            return isRef(disabledSource) ? disabledSource.value : disabledSource();
          } catch (error) {
            reset();
            throw error;
          }
        },
        () => {
          try {
            if (readBoolean(options.isDisabled, "isDisabled")) reset();
          } catch (error) {
            reset();
            throw error;
          }
        },
        { flush: "sync" },
      ),
    );
  }
  const timeoutSource = options.timeout;
  if (isRef(timeoutSource) || typeof timeoutSource === "function") {
    stopWatches.push(
      watch(
        () => {
          try {
            return readTimeout(timeoutSource);
          } catch (error) {
            reset();
            throw error;
          }
        },
        (timeout) => {
          if (query.value.length > 0) scheduleReset(timeout);
        },
        { flush: "sync" },
      ),
    );
  }

  return Object.freeze({
    query: shallowReadonly(query),
    typeaheadProps,
    input,
    reset: () => {
      assertActive();
      return reset();
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      for (const stop of stopWatches) stop();
      reset();
    },
  });
}

/** Create a typeahead controller disposed with the current Vue effect scope. */
export function useTypeahead<Key extends CollectionKey, Value>(
  options: TypeaheadOptions<Key, Value>,
): TypeaheadController<Key> {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createTypeahead(options);
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  TypeaheadController,
  TypeaheadMatch,
  TypeaheadOptions,
  TypeaheadProps,
} from "./typeahead-types.ts";
