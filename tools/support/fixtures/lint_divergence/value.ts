import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";

export class JsonFloat {
  readonly value: number;
  constructor(value: number) {
    this.value = value;
  }
  toJSON(): number {
    return this.value;
  }
}
export type Integer = number | bigint;
export type Json =
  | null
  | boolean
  | number
  | bigint
  | JsonFloat
  | string
  | Json[]
  | { [key: string]: Json };
export type JsonObject = { [key: string]: Json };
export function object(value: Json | undefined): JsonObject | undefined {
  return value !== null &&
    typeof value === "object" &&
    !Array.isArray(value) &&
    !(value instanceof JsonFloat)
    ? value
    : undefined;
}
export function get(value: Json | undefined, ...keys: string[]): Json {
  let current = value;
  for (const key of keys) current = object(current)?.[key];
  return current ?? null;
}
export function list(value: Json | undefined): Json[] {
  return Array.isArray(value) ? value : [];
}
export function text(value: Json | undefined, fallback = ""): string {
  return typeof value === "string" ? value : fallback;
}
export function unsigned(value: Json | undefined): Integer | undefined {
  if (typeof value === "bigint")
    return value >= 0n && value <= 18446744073709551615n ? value : undefined;
  return typeof value === "number" &&
    Number.isInteger(value) &&
    value >= 0 &&
    value < 18446744073709551616
    ? value
    : undefined;
}
export function number(value: Json | undefined): number {
  return Number(unsigned(value) ?? 0);
}
export function byteOrder(a: string, b: string): number {
  return Buffer.compare(Buffer.from(a), Buffer.from(b));
}
// serde_json's default object map is a BTreeMap. Preserve that ordering in
// packets and hashes, including the explicit f64 ratios produced by Rust.
export function canonicalJson(value: Json, pretty = false): string {
  const float = (value: number): string => {
    if (Object.is(value, -0)) return "-0.0";
    const magnitude = Math.abs(value);
    if (magnitude !== 0 && (magnitude < 0.00001 || magnitude >= 1e16)) return value.toExponential();
    return Number.isInteger(value) ? `${value}.0` : String(value);
  };
  function encode(item: Json, depth: number): string {
    if (item instanceof JsonFloat) return float(item.value);
    if (typeof item === "bigint") return item.toString();
    if (item === null || typeof item !== "object") {
      return JSON.stringify(item);
    }
    const indent = (level: number) => (pretty ? "  ".repeat(level) : "");
    const gap = pretty ? "\n" : "";
    const values = Array.isArray(item)
      ? item.map((entry) => encode(entry, depth + 1))
      : Object.keys(item)
          .sort(byteOrder)
          .map(
            (name) =>
              `${JSON.stringify(name)}:${pretty ? " " : ""}${encode(item[name], depth + 1)}`,
          );
    const [open, close] = Array.isArray(item) ? ["[", "]"] : ["{", "}"];
    return values.length
      ? `${open}${gap}${values.map((entry) => indent(depth + 1) + entry).join(`,${gap}`)}${gap}${indent(depth)}${close}`
      : open + close;
  }
  return encode(value, 0);
}
export function sortValues(values: Json[]): Json[] {
  return values.sort((a, b) => byteOrder(canonicalJson(a), canonicalJson(b)));
}
export function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}
export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
export function readText(filename: string): string {
  try {
    return new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(
      fs.readFileSync(filename),
    );
  } catch (error) {
    throw new Error(`cannot read ${filename}: ${errorMessage(error)}`);
  }
}
export function readJson(filename: string): Json {
  const source = readText(filename);
  try {
    return parseJsonSource(source);
  } catch (error) {
    throw new Error(`cannot parse JSON ${filename}: ${errorMessage(error)}`);
  }
}
/** Preserve serde_json scalar domains, including overwritten invalid tokens. */
export function parseJsonSource(source: string): Json {
  const parsed = JSON.parse(
    source,
    (_key: string, value: unknown, context?: { source?: string }): unknown => {
      if (typeof value !== "number") return value;
      const token = context?.source;
      if (token !== undefined && /^-?\d+$/.test(token) && token !== "-0") {
        const integer = BigInt(token);
        if (integer >= -9223372036854775808n && integer <= 18446744073709551615n)
          return Number.isSafeInteger(value) ? value : integer;
      }
      return new JsonFloat(value);
    },
  ) as Json;
  const loneSurrogate = /[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/;
  for (const [token] of source.matchAll(
    /"(?:\\[\s\S]|[^"\\])*"|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?/g,
  )) {
    if (token.startsWith('"')) {
      if (loneSurrogate.test(JSON.parse(token) as string))
        throw new SyntaxError("JSON strings must contain valid Unicode scalar values");
    } else if (!Number.isFinite(Number(token)))
      throw new SyntaxError("JSON number is out of range");
  }
  return parsed;
}
export function writeText(filename: string, source: string): void {
  try {
    fs.mkdirSync(path.dirname(filename), { recursive: true });
  } catch (error) {
    throw new Error(`cannot create ${path.dirname(filename)}: ${errorMessage(error)}`);
  }
  try {
    fs.writeFileSync(filename, source);
  } catch (error) {
    throw new Error(`cannot write ${filename}: ${errorMessage(error)}`);
  }
}
export function writeJson(filename: string, value: Json): void {
  writeText(filename, canonicalJson(value, true) + "\n");
}
export function isFile(filename: string): boolean {
  try {
    return fs.statSync(filename).isFile();
  } catch {
    return false;
  }
}
export function relativePath(root: string, filename: string): string {
  const relative = path.relative(root, filename);
  return (
    relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative)
      ? relative
      : filename
  ).replaceAll("\\", "/");
}
export function repoRoot(): string {
  const valid = (directory: string) =>
    isFile(path.join(directory, "Cargo.toml")) &&
    isFile(path.join(directory, "pnpm-workspace.yaml"));
  const configured = process.env.VIZE_REPO_ROOT;
  if (configured !== undefined) {
    if (!valid(configured))
      throw new Error(`VIZE_REPO_ROOT=${configured} is not a Vize repository root`);
    return fs.realpathSync.native(configured);
  }
  for (const candidate of [
    process.env.RUST_SCRIPT_PATH,
    process.env.RUST_SCRIPT_BASE_PATH,
    fileURLToPath(import.meta.url),
    process.cwd(),
  ]) {
    if (!candidate) continue;
    let directory = isFile(candidate) ? path.dirname(candidate) : path.resolve(candidate);
    for (;;) {
      if (valid(directory)) return fs.realpathSync.native(directory);
      const parent = path.dirname(directory);
      if (parent === directory) break;
      directory = parent;
    }
  }
  throw new Error(
    `cannot find Vize repository root from ${process.cwd()}; run inside the repository or set VIZE_REPO_ROOT`,
  );
}
export function requiredString(value: Json, key: string, label: string): string {
  const field = get(value, key);
  if (typeof field !== "string" || !field) throw new Error(`${label}.${key} must be non-empty`);
  return field;
}
export function projectString(project: Json, field: string): string {
  const value = get(project, field);
  if (typeof value !== "string") throw new Error(`project is missing ${field}`);
  return value;
}
export function projectStrings(project: Json, field: string): string[] {
  const values = get(project, field);
  if (!Array.isArray(values)) throw new Error(`${text(get(project, "id"))} has no ${field}`);
  return values.map((value) => {
    if (typeof value !== "string") throw new Error(`${field} entries must be strings`);
    return value;
  });
}
