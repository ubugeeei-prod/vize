/** Declarative project controls shared by the gallery and preview runtime. */
export type MuseaGlobalValue = string | number | boolean;
export type MuseaGlobals = Record<string, MuseaGlobalValue>;

export interface MuseaToolbarOption {
  value: MuseaGlobalValue;
  label: string;
  /** Optional text icon, such as "☀". HTML is never interpreted. */
  icon?: string;
}

export interface MuseaToolbarControl {
  id: string;
  title: string;
  type: "select" | "toggle";
  options: Array<string | MuseaToolbarOption>;
  default: MuseaGlobalValue;
}

export interface ResolvedMuseaToolbarControl extends Omit<MuseaToolbarControl, "options"> {
  options: MuseaToolbarOption[];
}

export const MUSEA_GLOBALS_QUERY = "museaGlobals";

export function normalizeToolbar(
  toolbar: MuseaToolbarControl[] = [],
): ResolvedMuseaToolbarControl[] {
  const ids = new Set<string>();
  return toolbar.map((control) => {
    const fail = (reason: string): never => {
      throw new Error(`[musea] Invalid toolbar control ${JSON.stringify(control.id)}: ${reason}`);
    };
    if (!/^[a-zA-Z][\w-]*$/.test(control.id) || ids.has(control.id)) {
      fail("ids must be unique and start with a letter");
    }
    ids.add(control.id);
    if (!control.title?.trim()) fail("title is required");
    if (control.type !== "select" && control.type !== "toggle") fail("unknown control type");
    if (!Array.isArray(control.options) || control.options.length === 0)
      fail("options are required");
    if (control.type === "toggle" && control.options.length !== 2) fail("toggles need two options");
    const values = new Set<MuseaGlobalValue>();
    const options = control.options.map((option) => {
      const item = typeof option === "string" ? { value: option, label: option } : option;
      if (
        !item ||
        !["string", "boolean", "number"].includes(typeof item.value) ||
        (typeof item.value === "number" && !Number.isFinite(item.value)) ||
        typeof item.label !== "string" ||
        !item.label.trim() ||
        (item.icon !== undefined && typeof item.icon !== "string")
      ) {
        fail("options require a finite primitive value and a label");
      }
      if (values.has(item.value)) fail("option values must be unique");
      values.add(item.value);
      return { value: item.value, label: item.label, ...(item.icon ? { icon: item.icon } : {}) };
    });
    if (!values.has(control.default)) fail("default must match an option value");
    return {
      id: control.id,
      title: control.title,
      type: control.type,
      options,
      default: control.default,
    };
  });
}

/** Ignore unknown keys and invalid values from URLs, storage and messages. */
export function resolveToolbarGlobals(
  toolbar: ResolvedMuseaToolbarControl[],
  input: unknown,
): MuseaGlobals {
  const record = input && typeof input === "object" ? input : {};
  return Object.fromEntries(
    toolbar.map((control) => {
      const value = Object.hasOwn(record, control.id)
        ? (record as Record<string, unknown>)[control.id]
        : undefined;
      return [
        control.id,
        control.options.some((option) => option.value === value) ? value : control.default,
      ];
    }),
  );
}

export function parseToolbarGlobals(value: string | null): unknown {
  if (value === null) return undefined;
  try {
    return JSON.parse(value);
  } catch {
    return undefined;
  }
}
