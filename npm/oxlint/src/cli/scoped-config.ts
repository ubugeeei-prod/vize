import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { appendScriptlessWorkaround } from "../workaround.ts";
import { isStandaloneHtmlFile } from "../file-kinds.ts";
import { registerPathReplacementVariants } from "./workaround-files.ts";

type Config = Record<string, unknown>;

export interface ScopedConfig {
  file: string;
  bytes: string;
  value: Config;
}

export function readScopedConfig(cwd: string, args: readonly string[]): ScopedConfig | undefined {
  let file: string | undefined;
  for (let index = 0; index < args.length && args[index] !== "--"; index += 1) {
    if (args[index] === "-c" || args[index] === "--config") file = args[++index];
    else if (args[index].startsWith("--config=")) file = args[index].slice(9);
  }
  // Discovery/nested/dynamic configurations keep their existing route. This
  // transport owns only an explicit, complete JSON configuration.
  if (file == null || !file.endsWith(".json")) return undefined;
  file = path.resolve(cwd, file);
  const bytes = fs.readFileSync(file, "utf8");
  let value: unknown;
  try {
    value = JSON.parse(bytes);
  } catch {
    return undefined; // Let the actual engine retain its original parse error.
  }
  if (!isRecord(value) || (value.ignorePatterns == null && value.overrides == null))
    return undefined;
  if (!fs.lstatSync(file).isFile() || fs.lstatSync(file).isSymbolicLink())
    throw new Error(`Scoped Vue transport requires a regular JSON config: ${file}`);
  return { file, bytes, value };
}

export function validateScopedConfig(config: ScopedConfig, args: readonly string[]): void {
  const value = config.value;
  if (path.parse(config.file).root !== "/")
    throw new Error("Scoped Vue transport cannot preserve non-POSIX filesystem roots.");
  if (value.extends != null && (!Array.isArray(value.extends) || value.extends.length !== 0))
    throw new Error(
      "Scoped Vue transport cannot preserve inherited configs; use a complete JSON config.",
    );
  if (args.includes("--cwd") || args.some((arg) => arg.startsWith("--cwd=")))
    throw new Error("Scoped Vue transport requires the process cwd; remove --cwd.");
  if (
    args.some((arg) => ["--type-aware", "--type-check", "--type-check-only"].includes(arg)) ||
    (isRecord(value.options) && (value.options.typeAware || value.options.typeCheck))
  )
    throw new Error("Scoped Vue transport cannot preserve Oxlint's type-aware project graph.");
  if (
    args.some(
      (arg) =>
        ["--import-plugin", "--tsconfig"].includes(arg) ||
        arg.startsWith("--import-plugin=") ||
        arg.startsWith("--tsconfig="),
    ) ||
    (Array.isArray(value.plugins) && value.plugins.includes("import")) ||
    (Array.isArray(value.overrides) &&
      value.overrides.some(
        (row) => isRecord(row) && Array.isArray(row.plugins) && row.plugins.includes("import"),
      ))
  )
    throw new Error("Scoped Vue transport cannot preserve Oxlint's import/project resolution.");
}

export function validateSelectionPaths(cwd: string, candidates: Iterable<string>): void {
  // Stock --debug files emits unescaped lines and converts every backslash to
  // '/'. Refuse ambiguous names across all candidates, including non-Vue files.
  if (/[\r\n\\]/u.test(cwd) || [...candidates].some((file) => /[\r\n\\]/u.test(file)))
    throw new Error(
      "Scoped Vue transport cannot preserve filenames containing CR, LF or backslash.",
    );
  if (fs.existsSync(path.join(cwd, "oxlint-suppressions.json")))
    throw new Error("Scoped Vue transport cannot preserve path-sensitive suppression state.");
}

export function createScopedMirror(cwd: string, config: ScopedConfig, files: readonly string[]) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-vize-"));
  const configDir = path.dirname(config.file);
  const sibling = path.join(configDir, `${path.basename(root)}.json`);
  let ownsSibling = false;
  const cleanup = () => {
    try {
      if (ownsSibling) fs.unlinkSync(sibling);
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  };
  try {
    if (contains(configDir, root) || contains(cwd, root))
      throw new Error(
        "Scoped Vue transport requires a temporary directory outside the lint/config roots.",
      );
    const originalsToCopies = new Map<string, string>();
    const pathReplacements = new Map<string, string>();
    for (const file of files) {
      if (isStandaloneHtmlFile(file))
        throw new Error(
          "Scoped Vue transport requires .vue targets; HTML filtering is not equivalent.",
        );
      const relative = path.relative(configDir, file);
      const copy = contains(configDir, file)
        ? path.join(root, "inside", relative)
        : path.join(root, "outside", file.slice(path.parse(file).root.length));
      if (path.parse(file).root !== path.parse(configDir).root)
        throw new Error("Scoped Vue transport cannot preserve files on another filesystem root.");
      const source = fs.readFileSync(file, "utf8");
      fs.mkdirSync(path.dirname(copy), { recursive: true });
      fs.writeFileSync(copy, appendScriptlessWorkaround(source, file), { flag: "wx" });
      originalsToCopies.set(file, copy);
      registerPathReplacementVariants(pathReplacements, cwd, copy, file);
    }
    const copyPatterns = [...originalsToCopies.values()].map(literalGlob);
    const outside = [...originalsToCopies].filter(([file]) => !contains(configDir, file));
    const insidePrefix = toCliPath(path.join(root, "inside"));
    const outsidePrefix = toCliPath(path.join(root, "outside"));
    const mapPatterns = (value: unknown): string[] => {
      if (!Array.isArray(value) || value.some((item) => typeof item !== "string"))
        throw new Error("Scoped Vue transport requires string-array override patterns.");
      return value.flatMap((pattern: string) => {
        if (pattern.startsWith("!"))
          throw new Error("Scoped Vue transport cannot preserve negated override patterns.");
        if (path.isAbsolute(pattern)) return [`${literalGlob(outsidePrefix)}${toCliPath(pattern)}`];
        const mapped = [`${literalGlob(insidePrefix)}/${pattern}`];
        if (outside.length === 0) return mapped;
        if (pattern === "**/*.vue")
          return [...mapped, ...outside.map(([, copy]) => literalGlob(copy))];
        // A literal initial component cannot match the leading slash of the
        // original absolute outside path. Wildcard-led paths need a provider
        // capable of retaining that exact absolute comparison domain.
        const first = pattern.split("/")[0];
        if (first && !/[*?[\]{}\\]/u.test(first)) return mapped;
        throw new Error(
          `Scoped Vue transport cannot preserve outside-root override pattern: ${pattern}`,
        );
      });
    };
    const overrides = config.value.overrides;
    if (overrides != null && !Array.isArray(overrides))
      throw new Error("Scoped Vue transport requires an override array.");
    const borrowed: Config = {
      ...config.value,
      overrides: (overrides ?? []).flatMap((row: unknown) => {
        if (!isRecord(row)) throw new Error("Scoped Vue transport requires object overrides.");
        const files = mapPatterns(row.files);
        const excludes = row.excludeFiles == null ? [] : mapPatterns(row.excludeFiles);
        return [
          { ...row, excludeFiles: [...strings(row.excludeFiles), ...copyPatterns] },
          {
            ...row,
            files,
            ...(excludes.length ? { excludeFiles: excludes } : { excludeFiles: [] }),
          },
        ];
      }),
    };
    // The borrowed config is a sibling: plugin/extends resolution keeps its
    // original directory. Global ignores remain unchanged and apply to original
    // paths; copies live outside this root and were selected by the real engine.
    const descriptor = fs.openSync(sibling, "wx");
    ownsSibling = true;
    try {
      fs.writeFileSync(descriptor, `${JSON.stringify(borrowed, null, 2)}\n`);
    } finally {
      fs.closeSync(descriptor);
    }
    if (fs.readFileSync(config.file, "utf8") !== config.bytes)
      throw new Error("Oxlint config changed while preparing its scoped Vue transport.");
    return { sibling, originalsToCopies, pathReplacements, cleanup };
  } catch (error) {
    cleanup();
    throw error;
  }
}

function isRecord(value: unknown): value is Config {
  return typeof value === "object" && value != null && !Array.isArray(value);
}

function strings(value: unknown): string[] {
  if (value == null) return [];
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string"))
    throw new Error("Scoped Vue transport requires string-array override patterns.");
  return value;
}

function contains(directory: string, file: string): boolean {
  const relative = path.relative(directory, file);
  return relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
}

export function toCliPath(file: string): string {
  return file.split(path.sep).join("/");
}

export function literalGlob(file: string): string {
  return toCliPath(file).replaceAll(/[\\*?[\]{}]/gu, String.raw`\$&`);
}
