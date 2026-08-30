import { toValue } from "vue";

import type { FocusChangeReason, FocusEvent, FocusMode, FocusOptions } from "./focus-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_FOCUS_OPTION";
const invalidTargetDiagnostic = "VIZE_UI_FOCUS_TARGET";

export function capture(errors: unknown[], callback: () => void): void {
  try {
    callback();
  } catch (error) {
    errors.push(error);
  }
}

export function surfaceErrors(errors: readonly unknown[], message: string): void {
  if (errors.length === 1) throw errors[0];
  if (errors.length < 2) return;
  const Aggregate = globalThis.AggregateError as typeof AggregateError | undefined;
  if (typeof Aggregate === "function") throw new Aggregate(errors, message);
  const fallback = Object.assign(new Error(message), { errors: [...errors] });
  fallback.name = "AggregateError";
  throw fallback;
}

export function readBoolean(value: FocusOptions["isDisabled"], name: string): boolean {
  return validateBoolean(toValue(value), name);
}

export function validateBoolean(resolved: unknown, name: string): boolean {
  if (resolved === undefined) return false;
  if (typeof resolved !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: ${name} must resolve to a boolean`);
  }
  return resolved;
}

export function readMode(value: FocusMode | undefined): FocusMode {
  const resolved = value ?? "target";
  if (resolved !== "target" && resolved !== "within") {
    throw new TypeError(`${invalidOptionDiagnostic}: mode must be target or within`);
  }
  return resolved;
}

export function validateOptions(options: FocusOptions): FocusMode {
  const mode = readMode(options.mode);
  if (options.autoFocus !== undefined && typeof options.autoFocus !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: autoFocus must be a boolean`);
  }
  for (const name of ["onBlur", "onFocus", "onFocusChange"] as const) {
    const callback = options[name];
    if (callback !== undefined && typeof callback !== "function") {
      throw new TypeError(`${invalidOptionDiagnostic}: ${name} must be a function`);
    }
  }
  if (typeof options.isDisabled !== "function") readBoolean(options.isDisabled, "isDisabled");
  return mode;
}

export function eventElement(value: EventTarget | null): Element | null {
  const candidate = value as Partial<Element> | null;
  return candidate?.nodeType === 1 && typeof candidate.getRootNode === "function"
    ? (candidate as Element)
    : null;
}

export function validateElement(value: unknown): Element {
  const candidate = value as Partial<Element> | null;
  if (
    candidate?.nodeType !== 1 ||
    typeof candidate.getRootNode !== "function" ||
    candidate.ownerDocument?.nodeType !== 9
  ) {
    throw new TypeError(`${invalidTargetDiagnostic}: refresh expects an Element`);
  }
  return candidate as Element;
}

/** Test composed ancestry without assuming that both nodes share one realm. */
export function composedContains(host: Element, candidate: Element | null): boolean {
  let current: Node | null = candidate;
  while (current) {
    if (current === host) return true;
    const element = current as Element;
    if (element.assignedSlot) {
      current = element.assignedSlot;
      continue;
    }
    if (current.parentNode) {
      current = current.parentNode;
      continue;
    }
    const root = current.getRootNode();
    current = root && "host" in root ? (root as ShadowRoot).host : null;
  }
  return false;
}

/** Resolve the deepest active element through open shadow roots. */
export function activeElementOf(host: Element): Element | null {
  const root = host.getRootNode();
  let active =
    root && "activeElement" in root
      ? (root as Document | ShadowRoot).activeElement
      : host.ownerDocument.activeElement;
  while (active?.shadowRoot?.activeElement) active = active.shadowRoot.activeElement;
  return active;
}

export function ownsFocus(mode: FocusMode, host: Element, active: Element | null): boolean {
  return mode === "target" ? active === host : composedContains(host, active);
}

export function createFocusEvent(
  type: FocusEvent["type"],
  mode: FocusMode,
  host: Element,
  focusedTarget: Element | null,
  relatedTarget: Element | null,
  originalEvent: globalThis.FocusEvent | null,
  isFocusVisible: boolean,
  reason: FocusChangeReason,
): FocusEvent {
  return Object.freeze({
    type,
    mode,
    target: host,
    focusedTarget,
    relatedTarget,
    originalEvent,
    isFocusVisible,
    reason,
  });
}
