import { toValue } from "vue";

import type {
  CompositeDirection,
  CompositeFocusStrategy,
  CompositeNavigationCommand,
  CompositeNavigationOptions,
  CompositeOrientation,
} from "./composite-navigation-types.ts";
import type { CollectionKey } from "../collection/collection.ts";

const optionDiagnostic = "VIZE_UI_COMPOSITE_NAVIGATION_OPTION";

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

export function readOrientation(source: unknown): CompositeOrientation {
  const value = toValue(source) ?? "vertical";
  if (value !== "both" && value !== "horizontal" && value !== "vertical") {
    throw new TypeError(
      `${optionDiagnostic}: orientation must resolve to both, horizontal, or vertical`,
    );
  }
  return value;
}

export function readDirection(source: unknown): CompositeDirection {
  const value = toValue(source) ?? "ltr";
  if (value !== "ltr" && value !== "rtl") {
    throw new TypeError(`${optionDiagnostic}: direction must resolve to ltr or rtl`);
  }
  return value;
}

export function readPageSize(source: unknown): number {
  const value = toValue(source) ?? 10;
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 1) {
    throw new TypeError(`${optionDiagnostic}: pageSize must resolve to a positive safe integer`);
  }
  return value;
}

export function readStrategy(value: unknown): CompositeFocusStrategy {
  const strategy = value ?? "roving";
  if (strategy !== "roving" && strategy !== "active-descendant") {
    throw new TypeError(`${optionDiagnostic}: focusStrategy must be roving or active-descendant`);
  }
  return strategy;
}

export function keyIntent(
  event: KeyboardEvent,
  orientation: CompositeOrientation,
  direction: CompositeDirection,
): CompositeNavigationCommand | null {
  if (event.isComposing || event.altKey || event.ctrlKey || event.metaKey) return null;
  if (event.key === "Home") return "first";
  if (event.key === "End") return "last";
  if (event.key === "PageDown") return "page-next";
  if (event.key === "PageUp") return "page-previous";
  if ((orientation === "vertical" || orientation === "both") && event.key === "ArrowDown") {
    return "next";
  }
  if ((orientation === "vertical" || orientation === "both") && event.key === "ArrowUp") {
    return "previous";
  }
  if (orientation === "horizontal" || orientation === "both") {
    if (event.key === "ArrowRight") return direction === "rtl" ? "previous" : "next";
    if (event.key === "ArrowLeft") return direction === "rtl" ? "next" : "previous";
  }
  return null;
}

export function validateId(value: unknown): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(
      "VIZE_UI_COMPOSITE_NAVIGATION_ID: an item ID must be non-empty and contain no ASCII whitespace or controls",
    );
  }
  for (let index = 0; index < value.length; index++) {
    const code = value.charCodeAt(index);
    if (code <= 0x20 || code === 0x7f) {
      throw new TypeError(
        "VIZE_UI_COMPOSITE_NAVIGATION_ID: an item ID must be non-empty and contain no ASCII whitespace or controls",
      );
    }
  }
  return value;
}

export function validateOptions<Key extends CollectionKey, Value>(
  options: CompositeNavigationOptions<Key, Value>,
): CompositeFocusStrategy {
  if (!options || typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError(`${optionDiagnostic}: options must be an object`);
  }
  const registry = options.registry as Partial<typeof options.registry> | null;
  if (!registry || typeof registry.getNavigationKey !== "function" || !registry.activeKey) {
    throw new TypeError(`${optionDiagnostic}: registry must be a CollectionRegistry`);
  }
  const strategy = readStrategy(options.focusStrategy);
  if (strategy === "active-descendant" && typeof options.getItemId !== "function") {
    throw new TypeError(`${optionDiagnostic}: active-descendant requires a getItemId function`);
  }
  for (const name of ["getItemId", "onNavigate", "scrollIntoView"] as const) {
    const callback = options[name];
    if (callback !== undefined && typeof callback !== "function") {
      throw new TypeError(`${optionDiagnostic}: ${name} must be a function`);
    }
  }
  readOrientation(options.orientation);
  readDirection(options.direction);
  readBoolean(options.loop, "loop");
  readBoolean(options.isDisabled, "isDisabled");
  readPageSize(options.pageSize);
  if (strategy === "roving") {
    readBoolean((options as { preventScroll?: unknown }).preventScroll, "preventScroll");
  }
  if (
    options.typeahead !== undefined &&
    options.typeahead !== false &&
    (!options.typeahead ||
      typeof options.typeahead !== "object" ||
      Array.isArray(options.typeahead))
  ) {
    throw new TypeError(`${optionDiagnostic}: typeahead must be false or an options object`);
  }
  return strategy;
}

const nonTextInputType = /^(?:button|checkbox|color|file|image|radio|range|reset|submit)$/iu;

export function isEditableDescendant(event: KeyboardEvent): boolean {
  const path = typeof event.composedPath === "function" ? event.composedPath() : [event.target];
  for (const candidate of path) {
    if (candidate === event.currentTarget) break;
    const target = candidate as Partial<HTMLInputElement> | null;
    const name = target?.localName;
    if (name === "input") {
      if (!nonTextInputType.test(target?.type ?? "text")) return true;
      continue;
    }
    if (name === "select" || name === "textarea" || target?.isContentEditable === true) {
      return true;
    }
  }
  return false;
}

export function validateCommand(value: unknown): CompositeNavigationCommand {
  if (
    value !== "first" &&
    value !== "last" &&
    value !== "next" &&
    value !== "page-next" &&
    value !== "page-previous" &&
    value !== "previous"
  ) {
    throw new TypeError(`${optionDiagnostic}: navigation intent is invalid`);
  }
  return value;
}
