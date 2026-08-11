import { toValue } from "vue";

import type {
  FocusGuardDirection,
  FocusGuardPosition,
  FocusGuardRedirectEvent,
  FocusGuardsOptions,
} from "./focus-guards-types.ts";

const optionDiagnostic = "VIZE_UI_FOCUS_GUARDS_OPTION";

/** Resolve an event target without relying on the current realm's Element constructor. */
export function eventElement(value: EventTarget | null): Element | null {
  const candidate = value as Partial<Element> | null;
  return candidate?.nodeType === 1 && typeof candidate.getRootNode === "function"
    ? (candidate as Element)
    : null;
}

export function readBoolean(source: unknown, name: string, fallback: boolean): boolean {
  const value = toValue(source);
  if (value === undefined) return fallback;
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
    throw new TypeError(`${optionDiagnostic}: root must resolve to an Element or null`);
  }
  return value as Element;
}

export function readBranches(source: unknown, document: Document | null): readonly Element[] {
  const value = toValue(source);
  if (value === undefined || value === null) return [];
  if (!Array.isArray(value)) {
    throw new TypeError(`${optionDiagnostic}: branches must resolve to a readonly Element array`);
  }
  const branches = value as readonly Element[];
  for (const branch of branches) {
    if (branch?.nodeType !== 1 || !branch.ownerDocument) {
      throw new TypeError(`${optionDiagnostic}: every branch must be an Element`);
    }
    if (document && branch.ownerDocument !== document) {
      throw new TypeError(`${optionDiagnostic}: branches must share the root Document`);
    }
  }
  return [...new Set(branches)];
}

export function readTarget(value: unknown): HTMLElement | null {
  if (value === undefined || value === null) return null;
  const candidate = value as Partial<HTMLElement>;
  if (
    candidate.nodeType !== 1 ||
    candidate.namespaceURI !== "http://www.w3.org/1999/xhtml" ||
    typeof candidate.focus !== "function"
  ) {
    throw new TypeError(
      `${optionDiagnostic}: fallbackFocus must resolve to an HTMLElement or null`,
    );
  }
  return value as HTMLElement;
}

export function validateOptions(options: FocusGuardsOptions): void {
  if (!options || typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError(`${optionDiagnostic}: options must be an object`);
  }
  const root = readRoot(options.root);
  readBranches(options.branches, root?.ownerDocument ?? null);
  readBoolean(options.enabled, "enabled", true);
  readBoolean(options.preventScroll, "preventScroll", true);
  for (const name of ["accept", "fallbackFocus", "onRedirect"] as const) {
    if (options[name] !== undefined && typeof options[name] !== "function") {
      throw new TypeError(`${optionDiagnostic}: ${name} must be a function`);
    }
  }
}

export function createRedirectEvent(
  position: FocusGuardPosition,
  direction: FocusGuardDirection,
  reason: FocusGuardRedirectEvent["reason"],
  target: HTMLElement | null,
  relatedTarget: Element | null,
  originalEvent: globalThis.FocusEvent,
): FocusGuardRedirectEvent {
  let prevented = false;
  return Object.freeze({
    type: "focus-guard-redirect" as const,
    position,
    direction,
    reason,
    target,
    relatedTarget,
    originalEvent,
    get defaultPrevented() {
      return prevented;
    },
    preventDefault: () => {
      prevented = true;
    },
  });
}
