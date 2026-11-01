import assert from "node:assert/strict";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { homedir } from "node:os";
import { basename, dirname, isAbsolute, join, normalize } from "node:path";
import { object, sha256, wholeJson } from "./value.ts";
import type { Sink } from "./evidence.ts";
import { discard } from "./evidence.ts";

const workspaceNames = [
  "pnpm-workspace.yaml",
  "pnpm-workspaces.yaml",
  "pnpm-workspaces.yml",
  "pnpm-workspace.yml",
  ".pnpm-workspace.yaml",
  ".pnpm-workspace.yml",
  ".pnpm-workspaces.yaml",
  ".pnpm-workspaces.yml",
];
const pathKeys = ["HOME", "XDG_CONFIG_HOME", "LOCALAPPDATA", "PNPM_HOME"] as const;
interface FileInput {
  path: string;
  state: "ABSENT" | "PRESENT";
  kind: "file" | "directory" | "symlink" | "other" | null;
  bytes: number | null;
  sha256: string | null;
  rawBase64: string | null;
}
export interface BinConfiguration {
  schema: "vize.humanPnpm12161Configuration.v1";
  sourceCommit: "c7b46aedef7b81c07816cb551187449a3dd81792";
  root: string;
  platform: "darwin" | "linux";
  home: string;
  pathEnvironment: Record<string, string | null>;
  configEnvironment: Array<{
    name: string;
    present: true;
    empty: boolean;
    bytes: number;
    sha256: string;
    value: string | null;
  }>;
  explicitEnvironment: {
    npm_config_registry: string;
    npm_config_userconfig: string;
    npm_config_globalconfig: string;
  };
  argv: string[];
  configDirPreseeded: false;
  global: FileInput;
  userNpmrc: FileInput;
  globalNpmrc: FileInput;
  rootNpmrc: FileInput;
  rootPackage: FileInput;
  workspace: FileInput[];
  writerRuntimeAttribution: false;
}
/** This guard belongs at the actual ambient caller, before any sanitization.
 * Never publish an unowned config value or make it disappear before evidence. */
export function requireAmbientBinConfiguration(env: NodeJS.ProcessEnv, sink: Sink = discard): void {
  const rows = Object.keys(env)
    .filter((name) => /^(?:pnpm_config_|npm_config_)/iu.test(name))
    .sort()
    .map((name) => {
      const value = env[name];
      assert(value !== undefined);
      return {
        name,
        present: true,
        empty: value === "",
        bytes: Buffer.byteLength(value),
        sha256: sha256(value),
        value: null,
      };
    });
  sink("ambient-package-manager-configuration-before-cleanup", { rows });
  assert.equal(
    rows.length,
    0,
    "unowned ambient package-manager configuration must be witnessed and refused before cleanup",
  );
}
function absolute(value: string): string {
  assert(isAbsolute(value) && normalize(value) === value && !/[\0\r\n]/u.test(value));
  return value;
}
/** c7 defaults.rs161–174 delegates to config-dir/lib.rs. Empty/relative XDG
 * inputs are outside this two-role contract; they are retained and refused. */
export function globalBinConfigPath(
  platform: string,
  home: string,
  env: Record<string, string | null>,
): string {
  assert(platform === "darwin" || platform === "linux");
  absolute(home);
  const xdg = env.XDG_CONFIG_HOME;
  if (xdg !== null && xdg !== undefined) return join(absolute(xdg), "pnpm/config.yaml");
  return platform === "darwin"
    ? join(home, "Library/Preferences/pnpm/config.yaml")
    : join(home, ".config/pnpm/config.yaml");
}
type SafeBytes = Buffer | ((raw: Buffer) => boolean) | null;
function fileInput(path: string, expected: SafeBytes): FileInput {
  absolute(path);
  const stat = lstatSync(path, { throwIfNoEntry: false });
  if (stat === undefined)
    return { path, state: "ABSENT", kind: null, bytes: null, sha256: null, rawBase64: null };
  const kind = stat.isSymbolicLink()
    ? "symlink"
    : stat.isDirectory()
      ? "directory"
      : stat.isFile()
        ? "file"
        : "other";
  let raw: Buffer | null = null;
  if (kind === "file" && stat.size <= 64 * 1024) raw = readFileSync(path);
  // Unowned config may contain credentials. Preserve its path/type/size/hash,
  // refuse it, and never upload its body as if it were a controlled fixture.
  return {
    path,
    state: "PRESENT",
    kind,
    bytes: stat.size,
    sha256: raw === null ? null : sha256(raw),
    rawBase64:
      raw !== null &&
      expected !== null &&
      (Buffer.isBuffer(expected) ? raw.equals(expected) : expected(raw))
        ? raw.toString("base64")
        : null,
  };
}
/** Closed generated age exclusions; no arbitrary scalar, setting or secret
 * may become an approved body merely because its YAML shape looks familiar. */
export function safeAgeWorkspace(raw: Buffer, manifest: Buffer): boolean {
  try {
    const version = object(object(wholeJson(manifest)).devDependencies).vize;
    if (typeof version !== "string" || !/^\d+\.\d+\.\d+$/u.test(version)) return false;
    const allowed = new Set(
      [
        "vize",
        "@vizejs/native",
        ...[
          "darwin-arm64",
          "darwin-x64",
          "linux-arm64-gnu",
          "linux-arm64-musl",
          "linux-x64-gnu",
          "linux-x64-musl",
          "win32-arm64-msvc",
          "win32-x64-msvc",
        ].map((target) => "@vizejs/native-" + target),
      ].map((name) => name + "@" + version),
    );
    const text = new TextDecoder("utf8", { fatal: true }).decode(raw);
    const prefix = "minimumReleaseAgeExclude:\n";
    if (!text.startsWith(prefix) || !text.endsWith("\n")) return false;
    const lines = text.slice(prefix.length, -1).split("\n"),
      seen = new Set<string>();
    if (lines.length === 0 || lines.length > allowed.size) return false;
    for (const line of lines) {
      if (!line.startsWith("  - ")) return false;
      const token = line.slice(4),
        quoted = token.startsWith("'") && token.endsWith("'");
      const value = quoted ? token.slice(1, -1) : token;
      if (!allowed.has(value) || seen.has(value) || (value.startsWith("@") && !quoted))
        return false;
      seen.add(value);
    }
    return true;
  } catch {
    return false;
  }
}
export function workspaceInputPaths(root: string): string[] {
  absolute(root);
  const result: string[] = [];
  for (let directory = root; ; directory = dirname(directory)) {
    for (const name of workspaceNames) result.push(join(directory, name));
    if (dirname(directory) === directory) return result;
  }
}
function environmentInputs(
  env: NodeJS.ProcessEnv,
  expected: BinConfiguration["explicitEnvironment"],
) {
  return Object.keys(env)
    .filter((name) => /^(?:pnpm_config_|npm_config_)/iu.test(name))
    .sort()
    .map((name) => {
      const raw = env[name];
      assert(raw !== undefined);
      return {
        name,
        present: true as const,
        empty: raw === "",
        bytes: Buffer.byteLength(raw),
        sha256: sha256(raw),
        value: name in expected && raw === expected[name as keyof typeof expected] ? raw : null,
      };
    });
}
function explicitEnvironment(root: string): BinConfiguration["explicitEnvironment"] {
  const kind = basename(root).replace(/^consumer-/u, "");
  assert(kind === "original" || kind === "tsrs-supplement");
  return {
    npm_config_registry: "https://registry.npmjs.org",
    npm_config_userconfig: join(dirname(root), `${kind}-user.npmrc`),
    npm_config_globalconfig: join(dirname(root), `${kind}-global.npmrc`),
  };
}
function absent(row: FileInput): void {
  assert.deepEqual(
    row,
    { path: row.path, state: "ABSENT", kind: null, bytes: null, sha256: null, rawBase64: null },
    `unowned config input must be absent: ${row.path}`,
  );
}
function empty(row: FileInput, path: string): void {
  assert.equal(row.path, path);
  assert.deepEqual(
    { ...row, path: "" },
    { path: "", state: "PRESENT", kind: "file", bytes: 0, sha256: sha256(""), rawBase64: "" },
  );
}
export function validateBinConfiguration(input: BinConfiguration, expectedManifest: Buffer): void {
  assert.equal(input.schema, "vize.humanPnpm12161Configuration.v1");
  assert.equal(input.sourceCommit, "c7b46aedef7b81c07816cb551187449a3dd81792");
  assert.equal(input.writerRuntimeAttribution, false);
  absolute(input.root);
  absolute(input.home);
  assert.equal(input.pathEnvironment.HOME, input.home);
  assert.deepEqual(Object.keys(input.pathEnvironment).sort(), [...pathKeys].sort());
  assert.equal(
    input.global.path,
    globalBinConfigPath(input.platform, input.home, input.pathEnvironment),
  );
  assert.deepEqual(input.argv, ["install"]);
  assert.equal(input.configDirPreseeded, false);
  assert.deepEqual(input.explicitEnvironment, explicitEnvironment(input.root));
  const entries = Object.entries(input.explicitEnvironment).sort(([a], [b]) => a.localeCompare(b));
  assert.deepEqual(
    input.configEnvironment.map((row) => row.name),
    entries.map(([name]) => name),
    "unowned PNPM_CONFIG or npm_config namespace entry (including empty/masked values)",
  );
  for (const row of input.configEnvironment) {
    const value =
      input.explicitEnvironment[row.name as keyof BinConfiguration["explicitEnvironment"]];
    assert.equal(row.value, value);
    assert.equal(row.bytes, Buffer.byteLength(value));
    assert.equal(row.present, true);
    assert.equal(row.empty, value === "");
    assert.equal(row.sha256, sha256(value));
  }
  absent(input.global);
  absent(input.rootNpmrc);
  assert.equal(input.rootNpmrc.path, join(input.root, ".npmrc"));
  empty(input.userNpmrc, input.explicitEnvironment.npm_config_userconfig);
  empty(input.globalNpmrc, input.explicitEnvironment.npm_config_globalconfig);
  assert.equal(input.rootPackage.path, join(input.root, "package.json"));
  assert.equal(input.rootPackage.state, "PRESENT");
  assert.equal(input.rootPackage.kind, "file");
  assert.equal(input.rootPackage.rawBase64, expectedManifest.toString("base64"));
  assert.equal(input.rootPackage.sha256, sha256(expectedManifest));
  assert.equal(input.rootPackage.bytes, expectedManifest.length);
  assert.equal(object(wholeJson(expectedManifest)).pnpm, undefined);
  // The original fresh fixture has no workspace file. Do not create one to
  // short-circuit pnpm's source-defined upward walk or change the recipe.
  assert.deepEqual(
    input.workspace.map((row) => row.path),
    workspaceInputPaths(input.root),
  );
  for (const row of input.workspace) absent(row);
}
export function collectBinConfiguration(
  root: string,
  env: NodeJS.ProcessEnv,
  expectedManifest: Buffer,
  sink: Sink,
): BinConfiguration {
  assert.equal(realpathSync(root), root);
  assert(process.platform === "darwin" || process.platform === "linux");
  const paths = Object.fromEntries(pathKeys.map((key) => [key, env[key] ?? null]));
  const expected = explicitEnvironment(root);
  const home = homedir();
  sink("bin-configuration-environment-before-assertion", {
    root,
    platform: process.platform,
    home,
    pathEnvironment: Object.entries(paths).map(([name, value]) => ({
      name,
      present: value !== null,
      empty: value === "",
      bytes: value === null ? null : Buffer.byteLength(value),
      sha256: value === null ? null : sha256(value),
    })),
    configEnvironment: environmentInputs(env, expected),
    argv: ["install"],
  });
  for (const value of Object.values(paths)) if (value !== null) absolute(value);
  assert.equal(paths.HOME, home);
  const globalPath = globalBinConfigPath(process.platform, home, paths);
  const observe = (path: string, expected: SafeBytes): FileInput => {
    try {
      const input = fileInput(path, expected);
      sink("bin-configuration-file-input", input);
      return input;
    } catch (error) {
      sink("bin-configuration-file-refusal", { path, error: String(error) });
      throw error;
    }
  };
  const input: BinConfiguration = {
    schema: "vize.humanPnpm12161Configuration.v1",
    sourceCommit: "c7b46aedef7b81c07816cb551187449a3dd81792",
    root,
    platform: process.platform,
    home,
    pathEnvironment: paths,
    configEnvironment: environmentInputs(env, expected),
    explicitEnvironment: expected,
    argv: ["install"],
    configDirPreseeded: false,
    global: observe(globalPath, null),
    userNpmrc: observe(absolute(expected.npm_config_userconfig), Buffer.alloc(0)),
    globalNpmrc: observe(absolute(expected.npm_config_globalconfig), Buffer.alloc(0)),
    rootNpmrc: observe(join(root, ".npmrc"), null),
    rootPackage: observe(join(root, "package.json"), expectedManifest),
    workspace: workspaceInputPaths(root).map((path) => observe(path, null)),
    writerRuntimeAttribution: false,
  };
  sink("bin-configuration-before-install", input);
  validateBinConfiguration(input, expectedManifest);
  return input;
}
export function recheckBinConfiguration(
  input: BinConfiguration,
  expectedManifest: Buffer = Buffer.from(input.rootPackage.rawBase64 ?? "", "base64"),
  sink: Sink = discard,
): void {
  const observations = [
    input.global,
    input.userNpmrc,
    input.globalNpmrc,
    input.rootNpmrc,
    ...input.workspace.filter((row) => row.path !== join(input.root, "pnpm-workspace.yaml")),
  ].map((row) => ({
    expected: row,
    actual: fileInput(
      row.path,
      row === input.userNpmrc || row === input.globalNpmrc ? Buffer.alloc(0) : null,
    ),
  }));
  const manifest = fileInput(join(input.root, "package.json"), expectedManifest);
  const workspace = fileInput(join(input.root, "pnpm-workspace.yaml"), (raw) =>
    safeAgeWorkspace(raw, expectedManifest),
  );
  sink("bin-configuration-after-install-or-command", { observations, manifest, workspace });
  for (const row of observations) assert.deepEqual(row.actual, row.expected);
  assert.equal(manifest.rawBase64, expectedManifest.toString("base64"));
  assert.equal(manifest.sha256, sha256(expectedManifest));
  assert.equal(object(wholeJson(expectedManifest)).pnpm, undefined);
  if (workspace.state === "PRESENT") {
    assert.equal(workspace.kind, "file");
    assert(workspace.rawBase64 !== null);
    assert(safeAgeWorkspace(Buffer.from(workspace.rawBase64, "base64"), expectedManifest));
  }
}
