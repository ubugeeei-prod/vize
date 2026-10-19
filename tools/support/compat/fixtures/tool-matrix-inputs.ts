import { Buffer } from "node:buffer";
import { createHash } from "node:crypto";
import { globSync, statSync, realpathSync, readdirSync, readFileSync } from "node:fs";
import { resolve, relative, extname, isAbsolute } from "node:path";

const typecheckerAuthoredGlobs = [
  "vue",
  "ts",
  "tsx",
  "mts",
  "cts",
  "js",
  "jsx",
  "mjs",
  "cjs",
].flatMap((extension) => [`**/*.${extension}`, `**/.*/**/*.${extension}`]);

export function collectVueInputPaths(cwd: string, patterns: readonly string[]): string[] {
  return collectInputPaths(cwd, patterns);
}

export function collectInputPaths(cwd: string, patterns: readonly string[]): string[] {
  return [
    ...new Set(
      patterns.flatMap((pattern) =>
        expandHiddenRecursiveGlobs(pattern).flatMap((expanded) =>
          globSync(expanded, { cwd, exclude: [".yarn/**", "**/node_modules/**"] }),
        ),
      ),
    ),
  ]
    .filter((entry) => statSync(resolve(cwd, entry)).isFile())
    .map((entry) => entry.replaceAll("\\", "/"))
    .map((file) => ({ file, bytes: Buffer.from(file) }))
    .sort((left, right) => Buffer.compare(left.bytes, right.bytes))
    .map(({ file }) => file);
}

function expandHiddenRecursiveGlobs(pattern: string): string[] {
  const normalized = pattern.replaceAll("\\", "/");
  if (!normalized.includes("**/")) return [pattern];
  return [
    pattern,
    ...new Set(
      recursiveGlobIndexes(normalized).map((index) => {
        const prefix = normalized.slice(0, index);
        const suffix = normalized.slice(index + "**/".length);
        return `${prefix}**/.*/**/${suffix}`;
      }),
    ),
  ];
}

function recursiveGlobIndexes(pattern: string): number[] {
  const indexes: number[] = [];
  let start = 0;
  while (true) {
    const index = pattern.indexOf("**/", start);
    if (index === -1) return indexes;
    indexes.push(index);
    start = index + 3;
  }
}

export function collectTypecheckerAuthoredPaths(cwd: string): string[] {
  return collectVueInputPaths(cwd, typecheckerAuthoredGlobs);
}

export function collectOwnedScriptFiles(cwd: string): string[] {
  const root = realpathSync(cwd);
  const files: string[] = [];
  function walk(directory: string, ancestors: readonly string[]): void {
    const canonical = realpathSync(directory);
    if (relative(root, canonical).split(/[/\\]/).includes(".."))
      throw new Error(`fixture script ownership escapes root: ${directory}`);
    if (ancestors.includes(canonical))
      throw new Error(`fixture script ownership cycles: ${directory}`);
    for (const entry of readdirSync(directory)) {
      if ([".git", ".yarn", "node_modules"].includes(entry)) continue;
      const path = resolve(directory, entry);
      const canonical = realpathSync(path);
      if (relative(root, canonical).split(/[/\\]/).includes(".."))
        throw new Error(`fixture script ownership escapes root: ${path}`);
      const metadata = statSync(path);
      if (metadata.isDirectory()) walk(path, [...ancestors, realpathSync(directory)]);
      else if (
        metadata.isFile() &&
        [".js", ".jsx", ".mjs", ".cjs", ".ts", ".tsx", ".mts", ".cts"].includes(extname(path))
      ) {
        files.push(relative(root, path).replaceAll("\\", "/"));
      }
    }
  }
  walk(root, []);
  return [...new Set(files)].sort((a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b)));
}

export interface ScriptScopeFile {
  readonly file: string;
  readonly bytes: number;
  readonly sha256: string;
}

export interface ScriptSourceScope {
  readonly schema: "vize.fixtureScriptSourceScan";
  readonly version: 1;
  readonly tsconfig: string;
  readonly files: readonly ScriptScopeFile[];
  readonly roots: readonly string[];
  readonly config: ScriptScopeConfig;
  readonly configSha256: string;
}

interface ScriptScopeConfig {
  readonly compilerOptions: {
    readonly allowJs: true;
    readonly checkJs: true;
    readonly jsx: "preserve";
  };
  readonly files: readonly string[];
  readonly include: readonly never[];
}

function scriptScopeConfig(files: readonly string[]): ScriptScopeConfig {
  return { compilerOptions: { allowJs: true, checkJs: true, jsx: "preserve" }, files, include: [] };
}

export function prepareScriptSourceScope(cwd: string, tsconfig: string): ScriptSourceScope {
  const files = collectOwnedScriptFiles(cwd);
  const roots = files.filter((file) => !/\.d\.(?:ts|mts|cts)$/.test(file));
  if (roots.length === 0)
    throw new Error(
      "zero-Vue fixture has no physical script roots; no files may be reported checked",
    );
  const config = scriptScopeConfig(files);
  return {
    schema: "vize.fixtureScriptSourceScan",
    version: 1,
    tsconfig,
    files: files.map((file) => {
      const bytes = readFileSync(resolve(cwd, file));
      return {
        file,
        bytes: bytes.length,
        sha256: createHash("sha256").update(bytes).digest("hex"),
      };
    }),
    roots,
    config,
    configSha256: createHash("sha256")
      .update(JSON.stringify(config, null, 2) + "\n")
      .digest("hex"),
  };
}

type JsonRecord = Readonly<Record<string, unknown>>;

function isRecord(value: unknown): value is JsonRecord {
  return value != null && typeof value === "object" && !Array.isArray(value);
}

export interface ScriptScopeProject {
  readonly id: unknown;
  readonly expectedVueFileCount: number;
  readonly tsconfig?: unknown;
  readonly typecheckPerformance?: { readonly baseline?: { readonly tsconfig?: unknown } };
}

const byteOrder = (left: string, right: string): number =>
  Buffer.compare(Buffer.from(left), Buffer.from(right));

export function validateScriptScope(
  project: ScriptScopeProject,
  output: JsonRecord,
  expectedFiles: unknown,
  authoredFiles: readonly string[],
  scope: unknown,
): void {
  const invalid = (message: string): never => {
    throw new Error(`invalid typechecker JSON output: ${message}`);
  };
  const requireRecord = (value: unknown, label: string): JsonRecord =>
    isRecord(value) ? value : invalid(`${label} must be an object`);
  const requireExactKeys = (value: JsonRecord, expected: readonly string[], label: string) => {
    if (JSON.stringify(Object.keys(value).sort()) !== JSON.stringify(expected))
      invalid(`${label} keys are invalid`);
  };
  const isDependencySource = (file: unknown) =>
    typeof file === "string" && file.split("/").includes("node_modules");
  const validateManifestInput = (
    files: readonly unknown[],
    label: string,
    accepts: (file: string) => boolean,
  ): string[] => {
    const strings = files.map((file) =>
      typeof file === "string"
        ? file
        : invalid(`${label} contains an unsupported normalized source path`),
    );
    if (
      new Set(strings).size !== strings.length ||
      JSON.stringify(strings) !== JSON.stringify([...strings].sort(byteOrder))
    )
      invalid(`${label} order/duplicates are invalid`);
    for (const file of strings) {
      if (
        isAbsolute(file) ||
        /^[A-Za-z]:/.test(file) ||
        file.includes("\\") ||
        file.split("/").some((part) => ["", ".", ".."].includes(part)) ||
        !accepts(file)
      )
        invalid(`${label} contains an unsupported normalized source path`);
    }
    return strings;
  };
  if (
    project.expectedVueFileCount !== 0 ||
    project.tsconfig != null ||
    project.typecheckPerformance?.baseline?.tsconfig != null ||
    !Array.isArray(expectedFiles) ||
    expectedFiles.length !== 0
  )
    invalid("script scope is only valid for an owned generated zero-Vue project");
  const record = requireRecord(scope, "script scope");
  requireExactKeys(
    record,
    ["config", "configSha256", "files", "roots", "schema", "tsconfig", "version"],
    "script scope",
  );
  if (
    record.schema !== "vize.fixtureScriptSourceScan" ||
    record.version !== 1 ||
    record.tsconfig !==
      `.vize-fixture-typecheck-${String(project.id).replaceAll(/[^A-Za-z0-9_.-]/g, "_")}.tsconfig.json`
  )
    invalid("script scope identity is invalid");
  if (!Array.isArray(record.files) || record.files.length === 0)
    invalid("script scope has no physical files");
  const fileRecords: readonly unknown[] = Array.isArray(record.files) ? record.files : [];
  const listed = fileRecords.map((entry) => {
    const file = requireRecord(entry, "script file");
    requireExactKeys(file, ["bytes", "file", "sha256"], "script file");
    if (
      typeof file.bytes !== "number" ||
      !Number.isSafeInteger(file.bytes) ||
      file.bytes < 0 ||
      typeof file.sha256 !== "string" ||
      !/^[a-f0-9]{64}$/.test(file.sha256)
    )
      invalid("script file bytes/hash are invalid");
    return file.file;
  });
  const files = validateManifestInput(
    listed,
    "physical script files",
    (file) =>
      /\.(?:ts|tsx|mts|cts|js|jsx|mjs|cjs)$/.test(file) &&
      !file.split("/").some((part) => [".git", ".yarn", "node_modules"].includes(part)) &&
      !file.includes("\0"),
  );
  if (files.some((file) => !authoredFiles.includes(file)))
    invalid("physical script file is not an authored fixture source");
  const roots = files.filter((file) => !/\.d\.(?:ts|mts|cts)$/.test(file));
  const config = scriptScopeConfig(files);
  if (
    roots.length === 0 ||
    JSON.stringify(record.roots) !== JSON.stringify(roots) ||
    JSON.stringify(record.config) !== JSON.stringify(config) ||
    record.configSha256 !==
      createHash("sha256")
        .update(JSON.stringify(config, null, 2) + "\n")
        .digest("hex")
  )
    invalid("script scope config/files/roots are inconsistent");
  const checked = output.files;
  const fileCount = typeof output.fileCount === "number" ? output.fileCount : undefined;
  if (
    !Array.isArray(checked) ||
    roots.some(
      (root) =>
        !authoredFiles.includes(root) ||
        !checked
          .slice(0, fileCount)
          .some((entry: unknown) => isRecord(entry) && entry.file === root),
    )
  )
    invalid("physical script root was not checked");
  const programs = output.programs;
  const program: unknown = Array.isArray(programs) ? programs[0] : undefined;
  if (
    !Array.isArray(programs) ||
    programs.length !== 1 ||
    !isRecord(program) ||
    program.root !== "." ||
    program.tsconfig !== record.tsconfig ||
    JSON.stringify(program.compilerOptions) !== JSON.stringify(config.compilerOptions)
  )
    return invalid("script scope has a foreign or altered owning program");
  const programFiles = program.files;
  if (
    !Array.isArray(programFiles) ||
    roots.some((root) => !programFiles.includes(root)) ||
    programFiles.some(
      (file: unknown) =>
        !(typeof file === "string" && files.includes(file)) && !isDependencySource(file),
    )
  )
    invalid("script root is missing from or foreign to its owning program");
}
