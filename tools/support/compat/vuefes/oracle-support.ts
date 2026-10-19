import fs from "node:fs";

// Structural views of the pinned oracle packages these differentials load at
// runtime from fixture installs (not workspace dependencies). Only the members
// the differentials touch are described.

export interface SfcDescriptor {
  readonly source: string;
  readonly scriptSetup: { readonly content: string } | null;
}

export interface CompilerSfc {
  readonly version: string;
  parse(
    source: string,
    options: { filename: string },
  ): { descriptor: SfcDescriptor; errors: readonly unknown[] };
  compileScript(
    descriptor: SfcDescriptor,
    options: { id: string; genDefaultAs?: string },
  ): { content: string };
}

export interface BabelSpecifier {
  readonly type: string;
  readonly local: { readonly name: string };
  readonly imported?: { readonly name: string };
}

export interface BabelStatement {
  readonly type: string;
  readonly start: number;
  readonly end: number;
  readonly source?: { readonly value: string };
  readonly specifiers?: readonly BabelSpecifier[];
}

export type BabelParse = (
  code: string,
  options: { sourceType: "module"; plugins: string[] },
) => { program: { body: readonly BabelStatement[] } };

export interface TransformResult {
  readonly code: string;
}

/** An unplugin `raw()` Vite/Rollup plugin instance with an async transform. */
export interface RawTransformPlugin {
  transform(code: string, id: string): Promise<TransformResult | null | undefined>;
}

export interface ModuleTransformOptions {
  readonly ssr?: boolean;
}

/** The plugin hooks the fixtures register with the pinned Vite server. */
export interface FixturePlugin {
  readonly name: string;
  readonly enforce?: "pre" | "post";
  transform?(
    code: string,
    id: string,
    options?: ModuleTransformOptions,
  ): Promise<{ code: string; map: null } | undefined> | { code: string; map: null } | undefined;
}

export const errorText = (error: unknown): string =>
  error instanceof Error ? (error.stack ?? String(error)) : String(error);

export type JsonRecord = Readonly<Record<string, unknown>>;

export function record(value: unknown, label: string): JsonRecord {
  if (typeof value !== "object" || value === null || Array.isArray(value))
    throw new TypeError(`${label} must be an object`);
  return value as JsonRecord;
}

export function list(value: unknown, label: string): readonly unknown[] {
  if (!Array.isArray(value)) throw new TypeError(`${label} must be an array`);
  return value;
}

export function text(value: unknown, label: string): string {
  if (typeof value !== "string") throw new TypeError(`${label} must be a string`);
  return value;
}

export function count(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isFinite(value))
    throw new TypeError(`${label} must be a number`);
  return value;
}

/** Parse a JSON file as an object without trusting its shape. */
export function readJsonRecord(file: string): JsonRecord {
  return record(JSON.parse(fs.readFileSync(file, "utf8")), file);
}
