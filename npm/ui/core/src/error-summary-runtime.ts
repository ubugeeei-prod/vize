import {
  computed,
  getCurrentScope,
  nextTick,
  onScopeDispose,
  shallowReadonly,
  shallowRef,
  toValue,
  watch,
} from "vue";

import type {
  ErrorSummaryController,
  ErrorSummaryField,
  ErrorSummaryOptions,
} from "./error-summary-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_ERROR_SUMMARY_OPTION";
const disposedDiagnostic = "VIZE_UI_ERROR_SUMMARY_DISPOSED";
const setupDiagnostic = "VIZE_UI_ERROR_SUMMARY_SETUP";

function readFields(value: ErrorSummaryOptions["fields"]): readonly ErrorSummaryField[] {
  const resolved = toValue(value);
  if (resolved === undefined) return [];
  if (!Array.isArray(resolved)) {
    throw new TypeError(`${invalidOptionDiagnostic}: fields must resolve to an array`);
  }
  const seen = new Set<string>();
  for (const field of resolved) {
    if (typeof field.id !== "string" || field.id.length === 0) {
      throw new TypeError(`${invalidOptionDiagnostic}: every field needs a non-empty id`);
    }
    if (typeof field.message !== "string") {
      throw new TypeError(`${invalidOptionDiagnostic}: every field needs a message string`);
    }
    if (seen.has(field.id)) {
      throw new TypeError(`${invalidOptionDiagnostic}: field ids must be unique, saw ${field.id}`);
    }
    seen.add(field.id);
  }
  return resolved;
}

function readBoolean(value: ErrorSummaryOptions["autoFocus"], fallback: boolean): boolean {
  const resolved = toValue(value);
  if (resolved === undefined) return fallback;
  if (typeof resolved !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: autoFocus must resolve to a boolean`);
  }
  return resolved;
}

/**
 * Create the focus contract behind an error summary.
 *
 * When invalid fields appear the previously focused element is captured and
 * focus moves to the summary; when every field is repaired that element gets
 * focus back, provided focus has not been moved elsewhere in the meantime.
 */
export function createErrorSummary(options: ErrorSummaryOptions = {}): ErrorSummaryController {
  for (const name of ["onRestore", "resolveField"] as const) {
    const callback = options[name];
    if (callback !== undefined && typeof callback !== "function") {
      throw new TypeError(`${invalidOptionDiagnostic}: ${name} must be a function`);
    }
  }
  if (typeof options.autoFocus !== "function") readBoolean(options.autoFocus, true);
  const fields = shallowRef<readonly ErrorSummaryField[]>(readFields(options.fields));
  const hasErrors = computed(() => fields.value.length > 0);
  let restoreTarget: HTMLElement | null = null;
  let disposed = false;

  const assertAlive = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };

  const readElement = (): HTMLElement | null => toValue(options.element) ?? null;

  const captureRestoreTarget = (): void => {
    if (typeof document === "undefined") return;
    const active = document.activeElement;
    restoreTarget = active instanceof HTMLElement && active !== document.body ? active : null;
  };

  const focusElement = (): boolean => {
    const element = readElement();
    if (element === null) return false;
    element.focus();
    return true;
  };

  const restoreFocus = (): boolean => {
    const target = restoreTarget;
    restoreTarget = null;
    if (target === null || !target.isConnected) return false;
    target.focus();
    return true;
  };

  const settleRepair = (): void => {
    if (typeof document === "undefined") return;
    const active = document.activeElement;
    const element = readElement();
    const focusInSummary = element !== null && active !== null && element.contains(active);
    const focusLost = active === null || active === document.body || !active.isConnected;
    if (!focusInSummary && !focusLost) {
      restoreTarget = null;
      return;
    }
    const target = restoreTarget;
    options.onRestore?.(restoreFocus() ? target : null);
  };

  const stopWatch = watch(
    () => readFields(options.fields),
    (next, previous) => {
      fields.value = next;
      if (previous.length === 0 && next.length > 0) {
        if (!readBoolean(options.autoFocus, true)) return;
        captureRestoreTarget();
        void nextTick(() => {
          if (!disposed && fields.value.length > 0) focusElement();
        });
      } else if (previous.length > 0 && next.length === 0) {
        void nextTick(() => {
          if (!disposed) settleRepair();
        });
      }
    },
    { flush: "sync" },
  );

  return Object.freeze({
    fields: shallowReadonly(fields),
    hasErrors,
    focusSummary: () => {
      assertAlive();
      captureRestoreTarget();
      return focusElement();
    },
    focusField: (id: string) => {
      assertAlive();
      const field = fields.value.find((candidate) => candidate.id === id);
      if (field === undefined) return null;
      const control =
        options.resolveField === undefined
          ? typeof document === "undefined"
            ? null
            : document.getElementById(field.id)
          : options.resolveField(field);
      if (!(control instanceof HTMLElement)) return null;
      control.focus();
      return control;
    },
    restoreFocus: () => {
      assertAlive();
      return restoreFocus();
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      restoreTarget = null;
      stopWatch();
    },
  });
}

/** Create an error summary controller disposed with the current Vue effect scope. */
export function useErrorSummary(options: ErrorSummaryOptions = {}): ErrorSummaryController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createErrorSummary(options);
  onScopeDispose(controller.dispose);
  return controller;
}
