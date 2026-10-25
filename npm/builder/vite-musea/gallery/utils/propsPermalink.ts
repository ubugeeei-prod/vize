import { normalizeSavedPaletteState, type SavedPaletteState } from "../composables/paletteState";

export const MUSEA_PROPS_QUERY = "museaProps";
export const MAX_PROPS_PERMALINK_LENGTH = 24_000;

export interface PropsPermalinkState {
  palette: SavedPaletteState;
  slots: Record<string, string>;
}

type PermalinkResult = { state: PropsPermalinkState; error: null } | { state: null; error: string };

/** URL data is JSON only. No expressions or executable prop bindings are accepted. */
export function readPropsPermalink(input: unknown, artPath: string): PermalinkResult {
  try {
    if (
      typeof input !== "string" ||
      encodeURIComponent(input).length > MAX_PROPS_PERMALINK_LENGTH
    ) {
      throw new Error("The shared props state is missing or too large.");
    }
    const value: unknown = JSON.parse(input);
    canonicalJson(value);
    if (!isRecord(value) || value.version !== 1 || value.artPath !== artPath) {
      throw new Error(
        "The shared props state has an unsupported version or belongs to another component.",
      );
    }
    return { state: normalizeState(value), error: null };
  } catch (error) {
    return {
      state: null,
      error: error instanceof Error ? error.message : "Invalid shared props state.",
    };
  }
}

/** Preserve the gallery's deployment base, variant fragment and unrelated query state. */
export function createPropsPermalink(
  currentUrl: string,
  artPath: string,
  state: PropsPermalinkState,
): string {
  const normalized = normalizeState(state);
  const snapshot = canonicalJson({
    version: 1,
    artPath,
    palette: {
      ...normalized.palette,
      customProps: normalized.palette.customProps.toSorted((a, b) => compare(a.name, b.name)),
      deletedPaletteProps: [...new Set(normalized.palette.deletedPaletteProps)].toSorted(compare),
    },
    slots: normalized.slots,
  });
  if (encodeURIComponent(snapshot).length > MAX_PROPS_PERMALINK_LENGTH) {
    throw new Error(
      "The props state is too large to share in a URL. Reduce props or slot content.",
    );
  }
  const url = new URL(currentUrl);
  url.searchParams.set(MUSEA_PROPS_QUERY, snapshot);
  return url.href;
}

function normalizeState(input: unknown): PropsPermalinkState {
  if (!isRecord(input) || !isRecord(input.palette) || !isRecord(input.slots)) {
    throw new Error("Invalid shared props state.");
  }
  const palette = input.palette;
  if (
    !isRecord(palette.values) ||
    !Array.isArray(palette.customProps) ||
    !Array.isArray(palette.deletedPaletteProps) ||
    palette.customProps.some(
      (prop) =>
        !isRecord(prop) ||
        typeof prop.name !== "string" ||
        !prop.name.trim() ||
        prop.name !== prop.name.trim() ||
        typeof prop.control !== "string" ||
        ![
          "text",
          "number",
          "boolean",
          "range",
          "select",
          "radio",
          "color",
          "object",
          "array",
        ].includes(prop.control) ||
        !Object.hasOwn(prop, "default_value"),
    ) ||
    new Set(palette.customProps.map((prop) => prop.name)).size !== palette.customProps.length ||
    palette.deletedPaletteProps.some((name) => typeof name !== "string") ||
    Object.values(input.slots).some((content) => typeof content !== "string")
  ) {
    throw new Error("Invalid shared props, controls or slot content.");
  }
  // These native DOM escape hatches must never be activated by an untrusted URL.
  const names = [...Object.keys(palette.values), ...palette.customProps.map((prop) => prop.name)];
  if (names.some((name) => /^(innerHTML|outerHTML|srcdoc)$/i.test(name) || /^on/i.test(name))) {
    throw new Error("DOM HTML and event handler props cannot be shared in a URL.");
  }
  canonicalJson(input);
  const normalized = normalizeSavedPaletteState(palette);
  if (!normalized) throw new Error("The shared props palette version is unsupported.");
  return { palette: normalized, slots: { ...input.slots } as Record<string, string> };
}

function isRecord(input: unknown): input is Record<string, unknown> {
  return typeof input === "object" && input !== null && !Array.isArray(input);
}

function compare(a: string, b: string): number {
  return a < b ? -1 : a > b ? 1 : 0;
}

function canonicalJson(input: unknown): string {
  let remaining = 4_096;
  function visit(value: unknown, depth: number): unknown {
    if (--remaining < 0 || depth > 12) throw new Error("The shared props state is too complex.");
    if (value === null || typeof value === "string" || typeof value === "boolean") return value;
    if (typeof value === "number" && Number.isFinite(value)) return value;
    if (Array.isArray(value)) return value.map((item) => visit(item, depth + 1));
    if (isRecord(value) && [Object.prototype, null].includes(Object.getPrototypeOf(value))) {
      return Object.fromEntries(
        Object.keys(value)
          .toSorted(compare)
          .map((key) => [key, visit(value[key], depth + 1)]),
      );
    }
    throw new Error("Only JSON values can be shared in a props URL.");
  }
  return JSON.stringify(visit(input, 0));
}
