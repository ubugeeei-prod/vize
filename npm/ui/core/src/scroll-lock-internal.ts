import { toValue } from "vue";

import type { ScrollLockOptions, ScrollLockStrategy } from "./scroll-lock-types.ts";

const optionDiagnostic = "VIZE_UI_SCROLL_LOCK_OPTION";

export function readDocument(source: unknown): Document | null {
  const value = toValue(source);
  if (value === undefined || value === null) return null;
  const candidate = value as Partial<Document>;
  if (candidate.nodeType !== 9 || candidate.documentElement?.nodeType !== 1) {
    throw new TypeError(`${optionDiagnostic}: document must resolve to a Document or null`);
  }
  return value as Document;
}

export function readBoolean(source: unknown, name: string, fallback: boolean): boolean {
  const value = toValue(source);
  if (value === undefined) return fallback;
  if (typeof value !== "boolean") {
    throw new TypeError(`${optionDiagnostic}: ${name} must resolve to a boolean`);
  }
  return value;
}

export function readStrategy(source: unknown): ScrollLockStrategy {
  const value = toValue(source) ?? "auto";
  if (value !== "auto" && value !== "fixed" && value !== "overflow") {
    throw new TypeError(
      `${optionDiagnostic}: strategy must resolve to "auto", "fixed", or "overflow"`,
    );
  }
  return value;
}

export function validateScrollLockOptions(options: ScrollLockOptions): void {
  if (!options || typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError(`${optionDiagnostic}: options must be an object`);
  }
  readDocument(options.document);
  readBoolean(options.enabled, "enabled", true);
  readBoolean(options.preserveScrollbarGap, "preserveScrollbarGap", true);
  readBoolean(options.restoreScroll, "restoreScroll", true);
  readStrategy(options.strategy);
}
