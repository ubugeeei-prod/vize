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

export function collectVueInputPaths(cwd, patterns) {
  return collectInputPaths(cwd, patterns);
}

export function collectInputPaths(cwd, patterns) {
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

function expandHiddenRecursiveGlobs(pattern) {
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

function recursiveGlobIndexes(pattern) {
  const indexes = [];
  let start = 0;
  while (true) {
    const index = pattern.indexOf("**/", start);
    if (index === -1) return indexes;
    indexes.push(index);
    start = index + 3;
  }
}

export function collectTypecheckerAuthoredPaths(cwd) {
  return collectVueInputPaths(cwd, typecheckerAuthoredGlobs);
}

export function collectOwnedScriptFiles(cwd) {
  const root = realpathSync(cwd);
  const files = [];
  function walk(directory, ancestors) {
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

export function prepareScriptSourceScope(cwd, tsconfig) {
  const files = collectOwnedScriptFiles(cwd);
  const roots = files.filter((file) => !/\.d\.(?:ts|mts|cts)$/.test(file));
  if (roots.length === 0)
    throw new Error(
      "zero-Vue fixture has no physical script roots; no files may be reported checked",
    );
  const config = {
    compilerOptions: { allowJs: true, checkJs: true, jsx: "preserve" },
    files,
    include: [],
  };
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

export function validateScriptScope(project, output, expectedFiles, authoredFiles, scope) {
  const invalid = (message) => {
    throw new Error(`invalid typechecker JSON output: ${message}`);
  };
  const requireRecord = (value, label) => {
    if (value == null || typeof value !== "object" || Array.isArray(value))
      invalid(`${label} must be an object`);
  };
  const requireExactKeys = (value, expected, label) => {
    if (JSON.stringify(Object.keys(value).sort()) !== JSON.stringify(expected))
      invalid(`${label} keys are invalid`);
  };
  const isDependencySource = (file) =>
    typeof file === "string" && file.split("/").includes("node_modules");
  const validateManifestInput = (files, label, accepts) => {
    if (
      new Set(files).size !== files.length ||
      JSON.stringify(files) !==
        JSON.stringify([...files].sort((a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b))))
    )
      invalid(`${label} order/duplicates are invalid`);
    for (const file of files) {
      if (
        typeof file !== "string" ||
        isAbsolute(file) ||
        /^[A-Za-z]:/.test(file) ||
        file.includes("\\") ||
        file.split("/").some((part) => ["", ".", ".."].includes(part)) ||
        !accepts(file)
      )
        invalid(`${label} contains an unsupported normalized source path`);
    }
  };
  if (
    project.expectedVueFileCount !== 0 ||
    project.tsconfig != null ||
    project.typecheckPerformance?.baseline?.tsconfig != null ||
    !Array.isArray(expectedFiles) ||
    expectedFiles.length !== 0
  )
    invalid("script scope is only valid for an owned generated zero-Vue project");
  requireRecord(scope, "script scope");
  requireExactKeys(
    scope,
    ["config", "configSha256", "files", "roots", "schema", "tsconfig", "version"],
    "script scope",
  );
  if (
    scope.schema !== "vize.fixtureScriptSourceScan" ||
    scope.version !== 1 ||
    scope.tsconfig !==
      `.vize-fixture-typecheck-${String(project.id).replaceAll(/[^A-Za-z0-9_.-]/g, "_")}.tsconfig.json`
  )
    invalid("script scope identity is invalid");
  if (!Array.isArray(scope.files) || scope.files.length === 0)
    invalid("script scope has no physical files");
  const files = scope.files.map((record) => {
    requireRecord(record, "script file");
    requireExactKeys(record, ["bytes", "file", "sha256"], "script file");
    if (
      !Number.isSafeInteger(record.bytes) ||
      record.bytes < 0 ||
      !/^[a-f0-9]{64}$/.test(record.sha256)
    )
      invalid("script file bytes/hash are invalid");
    return record.file;
  });
  validateManifestInput(
    files,
    "physical script files",
    (file) =>
      /\.(?:ts|tsx|mts|cts|js|jsx|mjs|cjs)$/.test(file) &&
      !file.split("/").some((part) => [".git", ".yarn", "node_modules"].includes(part)) &&
      !file.includes("\0"),
  );
  if (files.some((file) => !authoredFiles.includes(file)))
    invalid("physical script file is not an authored fixture source");
  const roots = files.filter((file) => !/\.d\.(?:ts|mts|cts)$/.test(file));
  const config = {
    compilerOptions: { allowJs: true, checkJs: true, jsx: "preserve" },
    files,
    include: [],
  };
  if (
    roots.length === 0 ||
    JSON.stringify(scope.roots) !== JSON.stringify(roots) ||
    JSON.stringify(scope.config) !== JSON.stringify(config) ||
    scope.configSha256 !==
      createHash("sha256")
        .update(JSON.stringify(config, null, 2) + "\n")
        .digest("hex")
  )
    invalid("script scope config/files/roots are inconsistent");
  if (
    !Array.isArray(output.files) ||
    roots.some(
      (root) =>
        !authoredFiles.includes(root) ||
        !output.files.slice(0, output.fileCount).some((record) => record.file === root),
    )
  )
    invalid("physical script root was not checked");
  const programs = output.programs;
  if (
    !Array.isArray(programs) ||
    programs.length !== 1 ||
    programs[0].root !== "." ||
    programs[0].tsconfig !== scope.tsconfig ||
    JSON.stringify(programs[0].compilerOptions) !== JSON.stringify(config.compilerOptions)
  )
    invalid("script scope has a foreign or altered owning program");
  if (
    !Array.isArray(programs[0].files) ||
    roots.some((root) => !programs[0].files.includes(root)) ||
    programs[0].files.some((file) => !files.includes(file) && !isDependencySource(file))
  )
    invalid("script root is missing from or foreign to its owning program");
}
