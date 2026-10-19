// Explicit source, producer, path and initial-environment boundaries.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const overrides = [
  "NODE_OPTIONS",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
] as const;
const observerOverrides = ["VIZE_PUBLIC_NATIVE_CUSTODY", "VIZE_OXLINT_NATIVE_CUSTODY"] as const;
export const producerFiles: readonly string[] = [
  "tools/commands/release/npm/collect-public-install.rs",
  "tools/support/archive/tar.ts",
  ...[
    "collect.ts",
    "identity.ts",
    "registry.ts",
    "archive.ts",
    "probe.ts",
    "native-custody.cjs",
    "authority-schema.ts",
  ].map((name) => `tools/support/release/public_install/${name}`),
];

export interface Source {
  readonly C: string;
  readonly H: string;
  readonly tag: string;
  readonly R: string;
}
export interface FileDigest {
  readonly path: string;
  readonly sha256: string;
}
export interface ProducerAuthority {
  readonly schema: "vize-public-install-collector-v1";
  readonly sha256: string;
  readonly files: readonly FileDigest[];
  readonly scope: string;
}

/** A refusal reported as the collector's one-line failure message. */
export class CustodyError extends Error {}

/** Read a nested JSON value; anything that is not an object yields undefined. */
export function lookup(value: unknown, ...keys: readonly string[]): unknown {
  let current = value;
  for (const key of keys) {
    if (typeof current !== "object" || current === null || Array.isArray(current)) return undefined;
    current = (current as Readonly<Record<string, unknown>>)[key];
  }
  return current;
}

export function rejectOverrides(environment: NodeJS.ProcessEnv): void {
  const nonempty = [...overrides, ...observerOverrides].filter((key) => environment[key]);
  if (nonempty.length > 0)
    throw new CustodyError(
      `initial source/runtime overrides must be empty: ${nonempty.join(", ")}`,
    );
}

export function digest(data: string | Uint8Array): string {
  return createHash("sha256").update(data).digest("hex");
}

/** Python pathlib spelling: repeated slashes and `.` collapse; `..` is kept. */
export function purePath(value: string): string {
  const parts = value.split("/").filter((part) => part !== "" && part !== ".");
  return `${value.startsWith("/") ? "/" : ""}${parts.join("/")}` || ".";
}

/** An absolute canonical path below `owner`, of the expected kind. */
export function exactPath(value: string, owner?: string, directory = false): string {
  const canonical = purePath(value);
  if (!path.isAbsolute(canonical) || fs.realpathSync(canonical) !== canonical)
    throw new CustodyError(`absolute canonical paths without symlinks required: ${canonical}`);
  if (owner !== undefined && !canonical.startsWith(`${owner}${path.sep}`))
    throw new CustodyError(`path escaped its owner: ${canonical}`);
  const stat = fs.statSync(canonical);
  if (directory ? !stat.isDirectory() : !stat.isFile())
    throw new CustodyError(`unexpected path type: ${canonical}`);
  return canonical;
}

export function freshOutputs(value: string): readonly [string, string, string] {
  const output = purePath(value);
  const parent = path.dirname(output);
  if (!path.isAbsolute(output) || fs.realpathSync(parent) !== parent)
    throw new CustodyError("absolute canonical output parent required");
  const paths = [
    output,
    `${output}.public-payload-files.json`,
    `${output}.native-journal.jsonl`,
  ] as const;
  if (paths.some((candidate) => fs.lstatSync(candidate, { throwIfNoEntry: false }) !== undefined))
    throw new CustodyError("receipt, payload manifest and native journal must all be new");
  return paths;
}

function git(root: string, ...args: readonly string[]): Buffer {
  return execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    maxBuffer: 256 * 1024 * 1024,
    stdio: ["ignore", "pipe", "pipe"],
  });
}

export function sourceIdentity(
  root: string,
  cut: string,
  head: string,
  tag: string,
  run: string,
): Source {
  if (
    !/^[0-9a-f]{40}$/u.test(cut) ||
    !/^[0-9a-f]{40}$/u.test(head) ||
    !/^v0\.[0-9]+\.0$/u.test(tag) ||
    !/^[1-9][0-9]*$/u.test(run)
  )
    throw new CustodyError("explicit full C/H/stable minor tag/Release run required");
  for (const commit of [cut, head])
    if (git(root, "cat-file", "-t", commit).toString().trim() !== "commit")
      throw new CustodyError("immutable source object must be a commit");
  const raw = git(root, "cat-file", "commit", head).toString("utf8");
  // rev-list can still obey legacy grafts even with replacements disabled.
  const headers = raw.split("\n\n", 1)[0] ?? "";
  const parents = headers
    .split("\n")
    .filter((line) => line.startsWith("parent "))
    .map((line) => line.slice(7));
  if (parents.length !== 1 || parents[0] !== cut)
    throw new CustodyError("H must be the exact single-parent immutable C cut");
  return { C: cut, H: head, tag, R: run };
}

export function sourceFile(root: string, head: string, file: string): Buffer {
  return git(root, "cat-file", "blob", `${head}:${file}`);
}

const escapeRegExp = (text: string): string => text.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");

function catalogVersion(workspace: Buffer, catalog: string, packageName: string): string {
  const catalogs = [
    ...workspace
      .toString("utf8")
      .matchAll(new RegExp(`^  ${escapeRegExp(catalog)}:\\n((?:    [^\\n]*\\n|\\n)*)`, "gmu")),
  ].map((match) => match[1] ?? "");
  const entries =
    catalogs.length === 1
      ? [
          ...(catalogs[0] ?? "").matchAll(
            new RegExp(`^    "${escapeRegExp(packageName)}": "([^"]+)"$`, "gmu"),
          ),
        ]
      : [];
  const [entry] = entries;
  if (entries.length !== 1 || entry?.[1] === undefined)
    throw new CustodyError("H exact named catalog declaration missing or duplicated");
  return entry[1];
}

export const corsaVersionPattern = /^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$/u;
export const majorAtLeast7 = (version: string): boolean => Number(version.split(".")[0]) >= 7;

export function sourceManifests(
  root: string,
  source: Pick<Source, "H" | "tag">,
): { files: FileDigest[]; corsaVersion: string } {
  const version = source.tag.slice(1);
  const manifests = [
    ["npm/cli/package.json", "vize"],
    ["npm/oxlint/package.json", "oxlint-plugin-vize"],
    ["npm/native/package.json", "@vizejs/native"],
  ] as const;
  const workspace = sourceFile(root, source.H, "pnpm-workspace.yaml");
  if (catalogVersion(workspace, "native-binaries", "@vizejs/native-darwin-arm64") !== version)
    throw new CustodyError("H platform package catalog version differs from requested version");
  const corsa = catalogVersion(workspace, "corsa-runtime", "@typescript/typescript-darwin-arm64");
  if (!corsaVersionPattern.test(corsa) || !majorAtLeast7(corsa))
    throw new CustodyError("H exact public Corsa catalog version required");
  const files: FileDigest[] = [{ path: "pnpm-workspace.yaml", sha256: digest(workspace) }];
  for (const [file, expectedName] of manifests) {
    const data = sourceFile(root, source.H, file);
    const manifest: unknown = JSON.parse(data.toString("utf8"));
    if (
      lookup(manifest, "name") !== expectedName ||
      lookup(manifest, "version") !== version ||
      lookup(manifest, "private") === true
    )
      throw new CustodyError(`H source manifest differs from requested public version: ${file}`);
    if (expectedName === "vize") {
      const native = lookup(manifest, "dependencies", "@vizejs/native");
      if (native !== "workspace:*" && native !== version)
        throw new CustodyError("H CLI native workspace edge differs from requested native version");
      const declared = lookup(
        manifest,
        "optionalDependencies",
        "@typescript/typescript-darwin-arm64",
      );
      if (declared !== "catalog:corsa-runtime" && declared !== corsa)
        throw new CustodyError("H CLI bundled Corsa declaration differs from its catalog");
    } else {
      const declaration = lookup(manifest, "optionalDependencies", "@vizejs/native-darwin-arm64");
      if (declaration !== version && declaration !== "catalog:native-binaries")
        throw new CustodyError("H native platform declaration differs from requested version");
    }
    const targets = lookup(manifest, "napi", "targets");
    if (
      expectedName === "@vizejs/native" &&
      !(Array.isArray(targets) && targets.includes("aarch64-apple-darwin"))
    )
      throw new CustodyError("H does not declare the Darwin ARM64 native target");
    files.push({ path: file, sha256: digest(data) });
  }
  return { files, corsaVersion: corsa };
}

export function producerAuthority(root: string, expected?: string): ProducerAuthority {
  const files = [...producerFiles].sort().map((name) => ({
    path: name,
    sha256: digest(fs.readFileSync(exactPath(path.join(root, name)))),
  }));
  const snapshot = digest(files.map((item) => `${item.path}\0${item.sha256}\n`).join(""));
  if (expected !== undefined && (!/^[0-9a-f]{64}$/u.test(expected) || snapshot !== expected))
    throw new CustodyError("reviewed collector snapshot digest differs");
  return {
    schema: "vize-public-install-collector-v1",
    sha256: snapshot,
    files,
    scope: "reviewed collector tool snapshot; not a claim these tools are in H",
  };
}
