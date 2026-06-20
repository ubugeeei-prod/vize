import type { PaletteControl } from "../api";

export interface CustomProp {
  name: string;
  control: string;
  default_value: unknown;
}

export interface SavedPaletteState {
  version: 1;
  values: Record<string, unknown>;
  customProps: CustomProp[];
  deletedPaletteProps: string[];
}

export interface RestoredPaletteState {
  values: Record<string, unknown>;
  customProps: CustomProp[];
  deletedPaletteProps: Set<string>;
}

export const PALETTE_STATE_VERSION = 1;

const PALETTE_STATE_STORAGE_PREFIX = "musea:props-editor:";

export function getPaletteStateStorageKey(artPath: string): string {
  return `${PALETTE_STATE_STORAGE_PREFIX}${artPath}`;
}

export function normalizeSavedPaletteState(input: unknown): SavedPaletteState | null {
  if (!isRecord(input) || input.version !== PALETTE_STATE_VERSION) {
    return null;
  }

  if (!isRecord(input.values)) {
    return null;
  }

  const customPropsInput = Array.isArray(input.customProps) ? input.customProps : [];
  const customProps: CustomProp[] = [];
  const seenNames = new Set<string>();
  for (const item of customPropsInput) {
    const customProp = normalizeCustomProp(item);
    if (!customProp || seenNames.has(customProp.name)) {
      continue;
    }
    seenNames.add(customProp.name);
    customProps.push(customProp);
  }

  const deletedPaletteProps = Array.isArray(input.deletedPaletteProps)
    ? input.deletedPaletteProps.filter((name): name is string => typeof name === "string")
    : [];

  return {
    version: PALETTE_STATE_VERSION,
    values: { ...input.values },
    customProps,
    deletedPaletteProps,
  };
}

export function restorePaletteState(
  controls: PaletteControl[],
  saved: SavedPaletteState,
): RestoredPaletteState {
  const paletteNames = new Set(controls.map((control) => control.name));
  const customProps: CustomProp[] = [];
  const customNames = new Set<string>();

  for (const customProp of saved.customProps) {
    if (paletteNames.has(customProp.name) || customNames.has(customProp.name)) {
      continue;
    }
    customNames.add(customProp.name);
    customProps.push(customProp);
  }

  const deletedPaletteProps = new Set(
    saved.deletedPaletteProps.filter((name) => paletteNames.has(name)),
  );
  const allowedNames = new Set([...paletteNames, ...customNames]);
  const restoredValues = initialPaletteValues(controls);

  for (const customProp of customProps) {
    restoredValues[customProp.name] = customProp.default_value;
  }

  for (const [name, value] of Object.entries(saved.values)) {
    if (allowedNames.has(name)) {
      restoredValues[name] = value;
    }
  }

  return {
    values: restoredValues,
    customProps,
    deletedPaletteProps,
  };
}

export function readSavedPaletteState(artPath: string): SavedPaletteState | null {
  const storage = getPaletteStateStorage();
  if (!storage) return null;

  try {
    const raw = storage.getItem(getPaletteStateStorageKey(artPath));
    if (!raw) return null;
    return normalizeSavedPaletteState(JSON.parse(raw));
  } catch {
    return null;
  }
}

export function writeSavedPaletteState(artPath: string, state: SavedPaletteState): void {
  const storage = getPaletteStateStorage();
  if (!storage) return;

  try {
    storage.setItem(getPaletteStateStorageKey(artPath), JSON.stringify(state));
  } catch {
    // Browsers can reject localStorage writes in private or quota-limited contexts.
  }
}

export function clearSavedPaletteState(artPath: string): void {
  const storage = getPaletteStateStorage();
  if (!storage) return;

  try {
    storage.removeItem(getPaletteStateStorageKey(artPath));
  } catch {
    // Ignore storage cleanup failures for the same reason as writes.
  }
}

export function initialPaletteValues(controls: PaletteControl[]): Record<string, unknown> {
  const initial: Record<string, unknown> = {};
  for (const control of controls) {
    initial[control.name] =
      control.default_value !== undefined ? control.default_value : fallbackValue(control);
  }
  return initial;
}

function getPaletteStateStorage(): Storage | null {
  if (typeof localStorage === "undefined") {
    return null;
  }
  return localStorage;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function normalizeCustomProp(input: unknown): CustomProp | null {
  if (!isRecord(input) || typeof input.name !== "string" || typeof input.control !== "string") {
    return null;
  }

  const name = input.name.trim();
  const control = input.control.trim();
  if (!name || !control) {
    return null;
  }

  return {
    name,
    control,
    default_value: input.default_value,
  };
}

function fallbackValue(control: PaletteControl): unknown {
  if ((control.control === "select" || control.control === "radio") && control.options.length > 0) {
    return control.options[0].value;
  }
  if (control.control === "boolean") return false;
  if (control.control === "number" || control.control === "range") {
    return control.range?.min ?? 0;
  }
  if (control.control === "array") return [];
  if (control.control === "object") return {};
  return "";
}
