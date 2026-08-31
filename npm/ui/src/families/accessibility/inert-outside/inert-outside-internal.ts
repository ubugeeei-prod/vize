import { toValue } from "vue";

import type { InertOutsideMode, InertOutsideOptions } from "./inert-outside-types.ts";

const optionDiagnostic = "VIZE_UI_INERT_OUTSIDE_OPTION";
const rootDiagnostic = "VIZE_UI_INERT_OUTSIDE_ROOT";

export function readRoot(source: unknown): Element | null {
  const value = toValue(source);
  if (value === undefined || value === null) return null;
  const candidate = value as Partial<Element>;
  if (candidate.nodeType !== 1 || !candidate.ownerDocument) {
    throw new TypeError(`${rootDiagnostic}: root must resolve to an Element or null`);
  }
  return value as Element;
}

export function readBranches(source: unknown, document: Document | null): readonly Element[] {
  const value = toValue(source) ?? [];
  if (!Array.isArray(value)) {
    throw new TypeError(`${optionDiagnostic}: branches must resolve to an array of Elements`);
  }
  const unique = [...new Set(value as unknown[])];
  for (const branch of unique) {
    const candidate = branch as Partial<Element>;
    if (candidate.nodeType !== 1 || !candidate.ownerDocument) {
      throw new TypeError(`${optionDiagnostic}: branches must contain only Elements`);
    }
    if (document && candidate.ownerDocument !== document) {
      throw new TypeError(`${optionDiagnostic}: branches must share the root document`);
    }
  }
  return unique as Element[];
}

export function readEnabled(source: unknown): boolean {
  const value = toValue(source);
  if (value === undefined) return true;
  if (typeof value !== "boolean") {
    throw new TypeError(`${optionDiagnostic}: enabled must resolve to a boolean`);
  }
  return value;
}

export function readMode(source: unknown): InertOutsideMode {
  const value = toValue(source) ?? "both";
  if (value !== "aria-hidden" && value !== "both" && value !== "inert") {
    throw new TypeError(
      `${optionDiagnostic}: mode must resolve to "aria-hidden", "both", or "inert"`,
    );
  }
  return value;
}

export function validateOptions(options: InertOutsideOptions): void {
  if (!options || typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError(`${optionDiagnostic}: options must be an object`);
  }
  const root = readRoot(options.root);
  readBranches(options.branches, root?.ownerDocument ?? null);
  readEnabled(options.enabled);
  readMode(options.mode);
}
