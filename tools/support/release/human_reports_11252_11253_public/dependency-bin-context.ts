import assert from "node:assert/strict";
import { lstatSync, readFileSync, readdirSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { fileImage, inside } from "../computed_event_custody.ts";
import type { Image, Observation } from "./model.ts";
import type { BinPolicy } from "./dependency-bin-policy.ts";
import type { FileFacts, HumanBinFacts, PublicPackageFacts } from "./dependency-bin-types.ts";
import { bytes, object, sha256, text, wholeJson } from "./value.ts";
import type { BinConfiguration } from "./bin-configuration.ts";

export interface BinEdge {
  parentDirectory: string;
  dependencyName: string;
  version: string;
  directory: string;
}
interface PublicDeclaration {
  name: string;
  version: string;
  directory: string;
  resolved: string;
  integrity: string;
  manifestRawBase64: string;
}
type VersionCapture = Pick<
  Observation,
  "pid" | "status" | "signal" | "error" | "stdoutBase64" | "stderrBase64"
>;
export interface BinContext {
  schema: "vize.humanPnpm12161Context.v1";
  role: HumanBinFacts["role"];
  root: string;
  configuration: BinConfiguration;
  host: HumanBinFacts["host"];
  policy: BinPolicy;
  parent: PublicDeclaration;
  dependencyOwners: PublicDeclaration[];
  graph: BinEdge[];
  lockRawBase64: string;
  manager: {
    originalImage: Image;
    currentImage: Image;
    rawBase64: string;
    reportedVersion: string;
    version: VersionCapture;
    originalArgv: string[];
  };
  settings: {
    userBase64: string;
    globalBase64: string;
    rootNpmrcBase64: string | null;
    rootWorkspaceBase64: string;
    rootPackageBase64: string;
  };
  targets: Array<Image & { rawBase64: string }>;
  writerImageOwner: "UNKNOWN";
  writerRuntimeAttribution: false;
}
export interface BinManagerContext {
  node: Image;
  pnpm: Image;
  version: VersionCapture;
  reportedVersion: string;
  contextPath: string;
  configuration: BinConfiguration;
}
function smallText(path: string, maximum = 4200): Buffer {
  const stat = lstatSync(path);
  assert(stat.isFile() && !stat.isSymbolicLink());
  assert(stat.size <= maximum, "bounded text witness; no native payload body upload");
  const raw = readFileSync(path);
  assert.equal(raw.length, stat.size);
  assert(!new TextDecoder("utf8", { fatal: true }).decode(raw).includes("\0"));
  return raw;
}
function declaration(pkg: PublicPackageFacts): PublicDeclaration {
  return {
    name: pkg.name,
    version: pkg.version,
    directory: pkg.directory,
    resolved: pkg.resolved,
    integrity: pkg.integrity,
    manifestRawBase64: pkg.manifest.toString("base64"),
  };
}
function physicalOwner(root: string, parent: string, name: string): string {
  const search = createRequire(join(parent, "package.json")).resolve.paths(name);
  assert(search);
  for (const base of search) {
    const candidate = join(base, name, "package.json");
    if (!inside(root, candidate)) break;
    if (lstatSync(candidate, { throwIfNoEntry: false }) === undefined) continue;
    const actual = realpathSync(candidate);
    assert(inside(root, actual));
    return actual.slice(0, -"/package.json".length);
  }
  assert.fail(`declared dependency has no physical owner: ${name}`);
}
export function collectBinContext(
  root: string,
  role: HumanBinFacts["role"],
  parent: PublicPackageFacts,
  catalog: ReadonlyMap<string, PublicPackageFacts>,
  allEdges: readonly BinEdge[],
  policy: BinPolicy,
  manager: BinManagerContext,
): BinContext {
  assert.equal(parent.name, "vue");
  assert.equal(parent.version, "3.5.44");
  assert(inside(root, parent.directory));
  const manifest = object(wholeJson(parent.manifest));
  const names = new Set<string>();
  for (const kind of ["dependencies", "optionalDependencies", "peerDependencies"])
    if (manifest[kind] !== undefined)
      for (const name of Object.keys(object(manifest[kind]))) names.add(name);
  const graph: BinEdge[] = [],
    dependencyOwners: PublicDeclaration[] = [];
  for (const name of [...names].sort()) {
    const candidates = allEdges.filter(
      (edge) => edge.parentDirectory === parent.directory && edge.dependencyName === name,
    );
    assert(candidates.length > 0, "complete source-declared dependency edges required");
    for (const edge of candidates) assert.deepEqual(edge, candidates[0]);
    const edge = candidates[0]!;
    assert.equal(physicalOwner(root, parent.directory, name), edge.directory);
    const owner = catalog.get(edge.directory);
    assert(owner && owner.name === name && owner.version === edge.version);
    graph.push(edge);
    dependencyOwners.push(declaration(owner));
  }
  const owner = dependencyOwners.filter((row) => row.name === "typescript");
  assert.equal(owner.length, 1);
  const targetOwner = owner[0]!,
    targetManifest = object(wholeJson(bytes(targetOwner.manifestRawBase64)));
  const targets = Object.entries(object(targetManifest.bin)).map(([name, declared]) => {
    assert(["tsc", "tsserver"].includes(name));
    const path = join(targetOwner.directory, text(declared));
    assert(inside(targetOwner.directory, path));
    assert.equal(realpathSync(path), path);
    const raw = smallText(path);
    return { ...fileImage(path), rawBase64: raw.toString("base64") };
  });
  const initialRaw = readFileSync(manager.contextPath),
    initial = object(wholeJson(initialRaw));
  assert.equal(initial.root, root);
  assert.deepEqual(initial.node, manager.node);
  assert.deepEqual(initial.pnpm, manager.pnpm);
  assert.deepEqual(initial.originalArgv, ["install"]);
  const env = object(initial.registryEnvironment);
  const user = readFileSync(text(env.npm_config_userconfig));
  const global = readFileSync(text(env.npm_config_globalconfig));
  const optionalText = (name: string) =>
    lstatSync(join(root, name), { throwIfNoEntry: false }) === undefined
      ? null
      : readFileSync(join(root, name)).toString("base64");
  const currentRaw = smallText(manager.pnpm.path),
    currentImage = fileImage(manager.pnpm.path);
  assert.deepEqual(currentImage, manager.pnpm);
  return {
    schema: "vize.humanPnpm12161Context.v1",
    role,
    root,
    configuration: manager.configuration,
    host:
      role === "runtime-prop-null-11252"
        ? { platform: "darwin", arch: "arm64" }
        : { platform: "linux", arch: "x64" },
    policy,
    parent: declaration(parent),
    dependencyOwners,
    graph,
    lockRawBase64: readFileSync(join(root, "pnpm-lock.yaml")).toString("base64"),
    manager: {
      originalImage: manager.pnpm,
      currentImage,
      rawBase64: currentRaw.toString("base64"),
      reportedVersion: manager.reportedVersion,
      version: manager.version,
      originalArgv: ["install"],
    },
    settings: {
      userBase64: user.toString("base64"),
      globalBase64: global.toString("base64"),
      rootNpmrcBase64: optionalText(".npmrc"),
      rootWorkspaceBase64: optionalText("pnpm-workspace.yaml") ?? "",
      rootPackageBase64: readFileSync(join(root, "package.json")).toString("base64"),
    },
    targets,
    writerImageOwner: "UNKNOWN",
    writerRuntimeAttribution: false,
  };
}
export function binInventory(directory: string): FileFacts[] {
  const rows: FileFacts[] = [];
  const walk = (directory: string): void => {
    for (const name of readdirSync(directory).sort()) {
      const path = join(directory, name),
        stat = lstatSync(path);
      const kind = stat.isSymbolicLink()
        ? "symlink"
        : stat.isDirectory()
          ? "directory"
          : stat.isFile()
            ? "file"
            : "other";
      rows.push({
        path,
        realPath: realpathSync(path),
        kind,
        body: kind === "file" ? readFileSync(path) : null,
      });
      if (kind === "directory") walk(path);
    }
  };
  walk(directory);
  return rows;
}
export function contextFacts(
  context: BinContext,
  archives: ReadonlyMap<string, Buffer> | null,
  inventory: FileFacts[],
): HumanBinFacts {
  assert.equal(context.schema, "vize.humanPnpm12161Context.v1");
  assert.equal(context.writerImageOwner, "UNKNOWN");
  assert.equal(context.writerRuntimeAttribution, false);
  const pkg = (row: PublicDeclaration): PublicPackageFacts => {
    const archive = archives === null ? null : archives.get(row.name + "@" + row.version);
    assert(archive !== undefined);
    return { ...row, manifest: bytes(row.manifestRawBase64), archive };
  };
  const parent = pkg(context.parent),
    dependencyOwners = context.dependencyOwners.map(pkg);
  const owners = dependencyOwners.filter((row) => row.name === "typescript");
  assert.equal(owners.length, 1);
  const manager = context.manager,
    version = manager.version;
  assert.equal(sha256(bytes(manager.rawBase64)), manager.currentImage.sha256);
  assert.equal(bytes(manager.rawBase64).length, manager.currentImage.bytes);
  return {
    root: context.root,
    configuration: context.configuration,
    role: context.role,
    host: context.host,
    parent,
    owner: owners[0]!,
    dependencyOwners,
    graph: context.graph,
    lock: bytes(context.lockRawBase64),
    inventory,
    expectedInputPolicyGitBlob: context.policy.gitBlob,
    inputPolicyRaw: bytes(context.policy.rawBase64),
    manager: {
      reportedVersion: manager.reportedVersion,
      versionStdout: bytes(version.stdoutBase64),
      versionStatus: version.status,
      versionSignal: version.signal,
      versionError: version.error,
      versionStderr: bytes(version.stderrBase64),
      originalArgv: manager.originalArgv,
      originalImage: manager.originalImage,
      currentImage: {
        ...manager.currentImage,
        realPath: manager.currentImage.path,
        kind: "file",
        body: bytes(manager.rawBase64),
      },
    },
    targets: context.targets.map((row) => {
      const body = bytes(row.rawBase64);
      assert.equal(body.length, row.bytes);
      assert.equal(sha256(body), row.sha256);
      return { ...row, realPath: row.path, kind: "file", body };
    }),
    settings: {
      userConfig: bytes(context.settings.userBase64),
      globalConfig: bytes(context.settings.globalBase64),
      rootNpmrc:
        context.settings.rootNpmrcBase64 === null ? null : bytes(context.settings.rootNpmrcBase64),
      rootWorkspace: bytes(context.settings.rootWorkspaceBase64),
      rootPackage: bytes(context.settings.rootPackageBase64),
    },
  };
}
