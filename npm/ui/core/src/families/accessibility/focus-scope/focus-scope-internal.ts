import { toValue } from "vue";

import type { FocusScopeMoveOptions, FocusScopeOptions } from "./focus-scope-types.ts";

const optionDiagnostic = "VIZE_UI_FOCUS_SCOPE_OPTION";
const rootDiagnostic = "VIZE_UI_FOCUS_SCOPE_ROOT";

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

export function readBoolean(source: unknown, name: string): boolean {
  const value = toValue(source);
  if (value === undefined) return false;
  if (typeof value !== "boolean") {
    throw new TypeError(`${optionDiagnostic}: ${name} must resolve to a boolean`);
  }
  return value;
}

export function readRoot(source: unknown): Element | null {
  const value = toValue(source);
  if (value === undefined || value === null) return null;
  const candidate = value as Partial<Element>;
  if (candidate.nodeType !== 1 || !candidate.ownerDocument) {
    throw new TypeError(`${rootDiagnostic}: root must resolve to an Element or null`);
  }
  return value as Element;
}

export function readTarget(value: unknown, name: string): HTMLElement | null {
  if (value === undefined || value === null) return null;
  const candidate = value as Partial<HTMLElement>;
  if (
    candidate.nodeType !== 1 ||
    candidate.namespaceURI !== "http://www.w3.org/1999/xhtml" ||
    typeof candidate.focus !== "function"
  ) {
    throw new TypeError(`${optionDiagnostic}: ${name} must resolve to an HTMLElement or null`);
  }
  return value as HTMLElement;
}

export function validateOptions(options: FocusScopeOptions): void {
  if (!options || typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError(`${optionDiagnostic}: options must be an object`);
  }
  readRoot(options.root);
  readBoolean(options.contain, "contain");
  readBoolean(options.autoFocus, "autoFocus");
  readBoolean(options.restoreFocus, "restoreFocus");
  for (const name of [
    "initialFocus",
    "restoreTarget",
    "fallbackFocus",
    "accept",
    "onMountAutoFocus",
    "onUnmountAutoFocus",
  ] as const) {
    if (options[name] !== undefined && typeof options[name] !== "function") {
      throw new TypeError(`${optionDiagnostic}: ${name} must be a function`);
    }
  }
}

export type ResolvedMoveOptions = Required<
  Pick<FocusScopeMoveOptions, "includeProgrammatic" | "preventScroll" | "wrap">
> &
  FocusScopeMoveOptions;

export function resolveMoveOptions(value: FocusScopeMoveOptions | undefined): ResolvedMoveOptions {
  const options = value ?? {};
  if (!options || typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError(`${optionDiagnostic}: focus movement options must be an object`);
  }
  for (const name of ["includeProgrammatic", "preventScroll", "wrap"] as const) {
    const current = options[name];
    if (current !== undefined && typeof current !== "boolean") {
      throw new TypeError(`${optionDiagnostic}: ${name} must be a boolean`);
    }
  }
  if (options.accept !== undefined && typeof options.accept !== "function") {
    throw new TypeError(`${optionDiagnostic}: accept must be a function`);
  }
  return {
    ...options,
    includeProgrammatic: options.includeProgrammatic ?? false,
    preventScroll: options.preventScroll ?? true,
    wrap: options.wrap ?? false,
  };
}

/** Order positive tabindex values first and ascending, leaving ties to the caller. */
export function comparePositiveTabindex(left: HTMLElement, right: HTMLElement): number {
  const leftPositive = left.tabIndex > 0;
  const rightPositive = right.tabIndex > 0;
  if (leftPositive && rightPositive) return left.tabIndex - right.tabIndex;
  if (leftPositive !== rightPositive) return leftPositive ? -1 : 1;
  return 0;
}

export function documentOrder(elements: readonly HTMLElement[]): HTMLElement[] {
  const unique = [...new Set(elements)];
  const order = new Map(unique.map((element, index) => [element, index]));
  return unique.sort((left, right) => {
    const precedence = comparePositiveTabindex(left, right);
    if (precedence !== 0) return precedence;
    const position = left.compareDocumentPosition(right);
    if (position & 1) return (order.get(left) ?? 0) - (order.get(right) ?? 0);
    return position & 4 ? -1 : 1;
  });
}
