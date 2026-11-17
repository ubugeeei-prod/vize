// Canonical plugin-plan data. Native verification re-serializes the same bytes.
import { createHash } from "node:crypto";

const encoder = new TextEncoder();

export class VizePluginPlanError extends Error {
  constructor(code, subject, message) {
    super(`[plugin/${code}] ${subject}: ${message}`);
    this.name = "VizePluginPlanError";
    this.code = code;
    this.subject = subject;
  }
}

/**
 * Normalize authored data to canonical JSON values: plain objects, arrays,
 * strings, booleans, null and safe integers. Object members whose value is
 * `undefined` are omitted, as in JSON; everything else fails closed.
 */
export function canonicalData(value, subject, path, code = "options-invalid") {
  const parents = new Set();
  const visit = (current, at) => {
    if (current === null || typeof current === "boolean") return current;
    if (typeof current === "string") {
      if (!current.isWellFormed())
        throw new VizePluginPlanError(code, subject, `${at} contains a lone surrogate`);
      return current;
    }
    if (typeof current === "number") {
      if (!Number.isSafeInteger(current))
        throw new VizePluginPlanError(
          code,
          subject,
          `${at} must be a safe integer; pass other numbers as strings`,
        );
      return current === 0 ? 0 : current;
    }
    if (typeof current !== "object")
      throw new VizePluginPlanError(code, subject, `${at} is not canonical JSON data`);
    if (parents.has(current))
      throw new VizePluginPlanError(code, subject, `${at} contains a cycle`);
    parents.add(current);
    try {
      if (Array.isArray(current)) {
        return Array.from({ length: current.length }, (_, index) => {
          if (!Object.hasOwn(current, index) || current[index] === undefined)
            throw new VizePluginPlanError(code, subject, `${at}[${index}] is undefined`);
          return visit(current[index], `${at}[${index}]`);
        });
      }
      const prototype = Object.getPrototypeOf(current);
      if (prototype !== Object.prototype && prototype !== null)
        throw new VizePluginPlanError(code, subject, `${at} must be a plain object`);
      if (Object.getOwnPropertySymbols(current).length !== 0)
        throw new VizePluginPlanError(code, subject, `${at} has symbol keys`);
      const result = {};
      for (const key of sortedKeys(current)) {
        if (!key.isWellFormed())
          throw new VizePluginPlanError(code, subject, `${at} has a key with a lone surrogate`);
        const descriptor = Object.getOwnPropertyDescriptor(current, key);
        if (descriptor.get || descriptor.set)
          throw new VizePluginPlanError(code, subject, `${at}.${key} is an accessor`);
        if (descriptor.value === undefined) continue;
        result[key] = visit(descriptor.value, `${at}.${key}`);
      }
      return result;
    } finally {
      parents.delete(current);
    }
  };
  return visit(value, path);
}

/** Object keys in Unicode code point order (UTF-8 byte order), like the native verifier. */
export function sortedKeys(object) {
  return Object.keys(object).sort(compareCodePoints);
}

export function compareCodePoints(left, right) {
  const a = encoder.encode(left);
  const b = encoder.encode(right);
  const length = Math.min(a.length, b.length);
  for (let index = 0; index < length; index++) {
    if (a[index] !== b[index]) return a[index] - b[index];
  }
  return a.length - b.length;
}

/** Serialize canonical data without whitespace, with sorted keys. */
export function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${sortedKeys(value)
      .map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

export function digestOf(json) {
  return createHash("sha256").update(json).digest("hex");
}

export function deepFreeze(value) {
  if (value !== null && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value)) deepFreeze(child);
    Object.freeze(value);
  }
  return value;
}
