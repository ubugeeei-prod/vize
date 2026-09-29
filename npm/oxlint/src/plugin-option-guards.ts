export function optionField(value: unknown): boolean {
  return value === undefined || value === "always" || value === "never" || value === "any";
}

export function optionalBooleanField(value: unknown): boolean {
  return value === undefined || typeof value === "boolean";
}

export function isString(value: unknown): value is string {
  return typeof value === "string";
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function hasOnlyKeys(value: Record<string, unknown>, keys: readonly string[]): boolean {
  return Object.keys(value).every((key) => keys.includes(key));
}
