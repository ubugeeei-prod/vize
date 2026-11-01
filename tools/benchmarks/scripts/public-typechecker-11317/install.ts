import assert from "node:assert/strict";
import { createRequire } from "node:module";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  readlinkSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative } from "node:path";
import { pathToFileURL } from "node:url";
import { asRecord, asString, type JsonRecord } from "../../../support/json-value.ts";
import { readTarGz } from "../../../support/archive/tar.ts";
import {
  assertCustody,
  captureDeclarationGraph,
  fileImage,
  inside,
  type FileImage,
} from "../../../support/release/computed_event_custody.ts";
import { assertNativeIdentity } from "../../../support/release/computed_event_names.ts";
import { rejectOverrides } from "../../../support/release/public_install/identity.ts";
import {
  packageMetadata,
  read,
  readBytes,
} from "../../../support/release/public_install/registry.ts";
import {
  assertPackedManifest,
  packedManifest,
} from "../../../support/release/slot_instance_public/packed-manifest.ts";
import {
  requireNativeLoadPrerequisite,
  recheckNativeLoadPrerequisite,
} from "../../../support/release/human_reports_11252_11253_public/native-loader-prerequisite.ts";
import type { PackageBase } from "../../../support/release/human_reports_11252_11253_public/model.ts";
import { profiles, sha256, sources, type Role, type SourceFile } from "./contract.ts";
import { validateNative, type authenticate } from "./authority.ts";
import { capture, healthy, type Recorder } from "./execute.ts";

export function assertLocked(lock: JsonRecord, capsule: JsonRecord, name: string, version: string) {
  const row = asRecord(asRecord(lock.packages)["node_modules/" + name]);
  const wanted = asRecord(asRecord(capsule.npmSRI)[name]);
  assert.equal(row.version, version);
  assert.equal(row.link, undefined);
  assert.equal(row.integrity, wanted.integrity);
  assert.equal(row.resolved, wanted.tarball);
  return row;
}
/** Whole public SRI precedes parsing. Every shipped regular file is checked;
 * npm's own nested .bin entries are outside this archive-membership assertion. */
export function shippedImages(root: string, payload: Buffer, integrity: unknown) {
  const archive = packedManifest(payload, integrity),
    images: FileImage[] = [];
  for (const entry of readTarGz(payload)) {
    if (entry.kind !== "file") continue;
    const target = join(root, entry.name.slice("package/".length));
    assert.ok(inside(root, target));
    assert.equal(lstatSync(target).isSymbolicLink(), false);
    const image = fileImage(target);
    assert.ok(inside(root, image.path));
    assert.equal(image.bytes, entry.data.length, entry.name);
    assert.equal(image.sha256, sha256(entry.data), entry.name);
    images.push(image);
  }
  assert.equal(images.length, archive.files.length);
  return { archive, images };
}
export function managerImages(npm: string, npx: string) {
  const root = realpathSync(dirname(dirname(npm)));
  const raw = readFileSync(join(root, "package.json"));
  const manifest = asRecord(JSON.parse(raw.toString("utf8")));
  assert.equal(manifest.name, "npm");
  assert.equal(manifest.version, "11.19.1");
  const bins = asRecord(manifest.bin);
  for (const [name, actual] of [
    ["npm", npm],
    ["npx", npx],
  ]) {
    assert.equal(bins[name!], "bin/" + name + "-cli.js");
    assert.equal(realpathSync(actual!), join(root, asString(bins[name!])));
  }
  return [
    fileImage(npm),
    fileImage(npx),
    fileImage(join(root, "package.json")),
    fileImage(process.execPath),
  ];
}
export function assertManager(images: FileImage[], npm: string, npx: string) {
  assertCustody(images);
  assert.deepEqual(managerImages(npm, npx), images);
}
export function projectBin(root: string, cli: string) {
  const link = join(root, "node_modules/.bin/vize");
  assert.equal(lstatSync(link).isSymbolicLink(), true, "npm's Linux vize link must be explicit");
  const target = readlinkSync(link),
    resolved = realpathSync(link);
  assert.equal(resolved, realpathSync(cli), "npx must select the authenticated package bin");
  const pkg = asRecord(
    JSON.parse(readFileSync(join(root, "node_modules/vize/package.json"), "utf8")),
  );
  assert.equal(asRecord(pkg.bin).vize, "bin/vize");
  return { path: link, target, resolved, image: fileImage(resolved) };
}

export async function install(
  root: string,
  role: Role,
  npm: string,
  original: SourceFile[],
  authority: Awaited<ReturnType<typeof authenticate>>,
  record: Recorder,
  verifyManager: () => void,
) {
  verifyManager();
  rejectOverrides(process.env);
  assert.equal(process.version, "v24.22.1");
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  mkdirSync(root);
  for (const file of sources(original, role, "Original")) {
    const path = join(root, file.path);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, file.source, { flag: "wx" });
  }
  const manager = capture(record, "npm-version", [process.execPath, npm, "--version"], root);
  healthy(manager, 0);
  assert.equal(manager.stdout, "11.19.1\n");
  assert.equal(manager.stderr, "");
  record.json("install-input.json", {
    node: { version: process.version, image: fileImage(process.execPath) },
    npm: fileImage(npm),
    root,
    manifest: readFileSync(join(root, "package.json"), "utf8"),
    lifecycle: "original default npm install; no ignore-scripts or dependency/provider override",
  });
  verifyManager();
  const installed = capture(record, "npm-install", [process.execPath, npm, "install"], root);
  for (const file of ["package.json", "package-lock.json"])
    if (existsSync(join(root, file)))
      writeFileSync(join(record.root, "installed-" + file), readFileSync(join(root, file)));
  healthy(installed, 0);
  const lock = asRecord(JSON.parse(readFileSync(join(root, "package-lock.json"), "utf8")));
  assert.equal(lock.lockfileVersion, 3);
  assert.deepEqual(
    asRecord(asRecord(lock.packages)[""]).devDependencies,
    asRecord(JSON.parse(sources(original, role, "Original")[0]!.source)).devDependencies,
  );
  const modules = realpathSync(join(root, "node_modules")),
    version = profiles[role].version;
  const packageFacts: PackageBase[] = [];
  const images: FileImage[] = [],
    packages: unknown[] = [];
  const readRegistry = async (url: string) => {
    const value = await read(url);
    packages.push({ url, value });
    record.json("registry-observations.json", packages);
    return value;
  };
  const names = new Set([
    "vize",
    "@vizejs/native",
    "@vizejs/native-linux-x64-gnu",
    "vue",
    "typescript-native",
    "@typescript/typescript-linux-x64",
  ]);
  for (const name of Object.keys(asRecord(lock.packages)))
    if (/^node_modules\/@vue\/[^/]+$/u.test(name)) names.add(name.slice("node_modules/".length));
  for (const name of names) {
    record.json("phase.json", { stage: "authenticate-installed-package", name });
    const packageRoot = realpathSync(join(modules, name));
    assert.ok(inside(modules, packageRoot));
    const raw = readFileSync(join(packageRoot, "package.json"));
    // Full manifests and identity remain readable even if a later assertion fails.
    const label = name.replaceAll("/", "_").replaceAll("@", "");
    writeFileSync(join(record.root, label + ".installed-manifest.raw.json"), raw);
    const manifest = asRecord(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(raw)));
    const vize = name === "vize" || name.startsWith("@vizejs/");
    const wanted = vize
      ? version
      : name === "typescript-native" || name.startsWith("@typescript/")
        ? "7.0.2"
        : "3.5.44";
    assert.equal(manifest.version, wanted, name);
    assert.equal(manifest.name, name === "typescript-native" ? "typescript" : name);
    const entry = asRecord(asRecord(lock.packages)["node_modules/" + name]);
    assert.equal(entry.version, wanted);
    assert.equal(entry.link, undefined);
    if (vize) assertLocked(lock, authority.capsule, name, version);
    const source = vize
      ? { H: profiles[role].candidate, tag: "v" + version, R: profiles[role].run }
      : undefined;
    const provenance = await packageMetadata(
      asString(manifest.name),
      wanted,
      entry,
      source,
      readRegistry,
    );
    const payload = await readBytes(asString(entry.resolved));
    record.json(label + ".archive-download.json", {
      url: entry.resolved,
      integrity: entry.integrity,
      bytes: payload.length,
      sha256: sha256(payload),
      verified: false,
    });
    const checked = shippedImages(packageRoot, payload, entry.integrity);
    assert.deepEqual(raw, checked.archive.manifest);
    if (name === "vize")
      assertPackedManifest(
        raw,
        checked.archive.manifest,
        asRecord(
          JSON.parse(Buffer.from(authority.sourceFiles[0]!.rawBase64, "base64").toString("utf8")),
        ),
      );
    images.push(...checked.images);
    packageFacts.push({
      name,
      version: wanted,
      directory: packageRoot,
      resolved: asString(entry.resolved),
      integrity: asString(entry.integrity),
      archiveSha512: checked.archive.tarballSHA512,
      files: checked.images,
    });
    record.json(label + ".archive-authority.json", {
      name,
      version: wanted,
      provenance,
      files: checked.archive.files,
      tarballSHA512: checked.archive.tarballSHA512,
      allShippedBytesEqualInstalled: true,
      completeNodeModulesFileSetAsserted: false,
    });
  }
  const cli = join(modules, "vize/bin/vize"),
    cliRoot = join(modules, "vize");
  const graph = captureDeclarationGraph(cliRoot);
  record.json("installed-sdk-graph.json", graph);
  assert.deepEqual(
    graph.files.map((file) => relative(cliRoot, file.path)).sort(),
    authority.sourceFiles
      .slice(1)
      .map((file) => file.file)
      .sort(),
  );
  for (const file of graph.files) {
    const source = authority.sourceFiles.find(
      (source) => source.file === relative(cliRoot, file.path),
    );
    assert.ok(source);
    assert.equal(file.bytes, source.bytes);
    assert.equal(file.sha256, source.sha256);
  }
  verifyManager();
  const nativeLoad = requireNativeLoadPrerequisite(
    root,
    packageFacts,
    graph.files,
    {
      version,
      candidate: profiles[role].candidate,
      source: profiles[role].source,
      workflowHead: authority.ownership.workflowHead,
      run: profiles[role].run,
      capsule: authority.capsule,
      sdk: graph.files,
      observations: [],
      produced: authority.ownership.produced,
    },
    process.env,
    (kind, value) => record.json("native-prerequisite-" + kind + ".json", value),
  );
  record.json("native-prerequisite.json", nativeLoad);
  const bin = projectBin(root, cli);
  record.json("project-bin-before.json", bin);
  const require = createRequire(cli),
    nativeEntry = realpathSync(require.resolve("@vizejs/native"));
  const native = asRecord(require("@vizejs/native"));
  const loaded = Object.keys(require.cache).filter(
    (path) => path.endsWith(".node") && inside(modules, path),
  );
  assert.equal(loaded.length, 1);
  const binary = fileImage(loaded[0]!);
  assert.ok(inside(realpathSync(join(modules, "@vizejs/native-linux-x64-gnu")), binary.path));
  const exports = asRecord(require.cache[binary.path]?.exports);
  const helper = asRecord(require(join(modules, "@vizejs/native/native-binding-identity.js")));
  assert.equal(typeof native.getNativeBuildToken, "function");
  assert.equal(typeof exports.getNativeBuildToken, "function");
  assert.equal(typeof helper.getBindingIdentity, "function");
  const token: unknown = Reflect.apply(native.getNativeBuildToken as () => unknown, native, []);
  const imageToken: unknown = Reflect.apply(
    exports.getNativeBuildToken as () => unknown,
    exports,
    [],
  );
  const binding: unknown = Reflect.apply(
    helper.getBindingIdentity as (value: unknown) => unknown,
    helper,
    [exports],
  );
  record.json("installed-native-companion.json", {
    role,
    nativeEntry,
    binary,
    token,
    imageToken,
    binding,
    companionPid: process.pid,
    selectedOriginalCliPid: null,
    selectedOriginalBackendPid: null,
    scope: "this separate identity companion only; no original-command PID transfer",
  });
  assertNativeIdentity(token, imageToken, binding);
  validateNative(role, authority.capsule, binary, asString(token));
  verifyManager();
  const cliVersion = capture(
    record,
    "installed-cli-version",
    [process.execPath, cli, "--version"],
    root,
  );
  healthy(cliVersion, 0);
  assert.equal(cliVersion.stdout, "vize " + version + "\n");
  assert.equal(cliVersion.stderr, "");
  const resolver = join(modules, "typescript-native/lib/getExePath.js");
  verifyManager();
  const checkerModule = asRecord(await import(pathToFileURL(resolver).href));
  assert.equal(typeof checkerModule.default, "function");
  const checker = realpathSync(
    asString(Reflect.apply(checkerModule.default as () => unknown, undefined, [])),
  );
  assert.ok(inside(realpathSync(join(modules, "@typescript/typescript-linux-x64")), checker));
  verifyManager();
  const checkerVersion = capture(record, "installed-checker-version", [checker, "--version"], root);
  healthy(checkerVersion, 0);
  assert.equal(checkerVersion.stdout, "Version 7.0.2\n");
  assert.equal(checkerVersion.stderr, "");
  const unique = new Map<string, FileImage>();
  for (const image of [
    ...images,
    ...graph.files,
    binary,
    fileImage(nativeEntry),
    fileImage(checker),
    fileImage(resolver),
    fileImage(npm),
    fileImage(process.execPath),
    fileImage(join(root, "package-lock.json")),
  ]) {
    const old = unique.get(image.path);
    if (old) assert.deepEqual(image, old);
    else unique.set(image.path, image);
  }
  const custody = [...unique.values()];
  assertCustody(custody);
  record.json("installed-identity.json", {
    role,
    profiles: profiles[role],
    root,
    modules,
    cli,
    binary,
    checker: fileImage(checker),
    graph,
    images: custody,
    originalSelectedBackend: "UNKNOWN",
    originalSelectedCliPid: null,
    completeNodeModulesFileSetAsserted: false,
  });
  const verifyRoute = () => {
    verifyManager();
    assert.deepEqual(projectBin(root, cli), bin);
    recheckNativeLoadPrerequisite(
      { root, packages: packageFacts, sdk: graph.files, nativeLoad },
      process.env,
    );
  };
  return { images: custody, cli, checker, verifyRoute };
}
