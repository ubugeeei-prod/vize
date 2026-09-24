import { isRef, toRaw, unref } from "vue";

/**
 * Serialize a probe state into deterministic JSON.
 *
 * Test-only helper used inside probe templates. Refs are unwrapped, `Map`
 * and `Set` become arrays, `Date` becomes its ISO string (or `null` when
 * invalid), `bigint` becomes `"<n>n"`, and functions, symbols, and DOM nodes
 * are dropped so server and client output can be compared byte for byte.
 *
 * @param value State to serialize.
 * @returns JSON text.
 */
export function serializeProbeState(value: unknown): string {
  return JSON.stringify(normalize(value, 0)) ?? "null";
}

function normalize(value: unknown, depth: number): unknown {
  const plain: unknown = toRaw(isRef(value) ? unref(value) : value);
  if (depth > 6) return "[depth]";
  if (plain === undefined || plain === null) return null;
  switch (typeof plain) {
    case "bigint":
      return `${plain.toString()}n`;
    case "function":
    case "symbol":
      return undefined;
    case "number":
      return Number.isFinite(plain) ? plain : String(plain);
    case "object":
      break;
    default:
      return plain;
  }
  if (plain instanceof Date) return Number.isNaN(plain.getTime()) ? null : plain.toISOString();
  if (plain instanceof Map) return [...plain].map((entry) => normalize(entry, depth + 1));
  if (plain instanceof Set) return [...plain].map((entry) => normalize(entry, depth + 1));
  if (Array.isArray(plain)) return plain.map((entry) => normalize(entry, depth + 1));
  if (typeof Node !== "undefined" && plain instanceof Node) return "[node]";
  const prototype: unknown = Object.getPrototypeOf(plain);
  if (prototype !== Object.prototype && prototype !== null) return `[${plain.constructor.name}]`;
  const result: Record<string, unknown> = {};
  for (const [key, entry] of Object.entries(plain)) {
    const normalized = normalize(entry, depth + 1);
    if (normalized !== undefined) result[key] = normalized;
  }
  return result;
}
