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
import { asRecord, asString } from "../../../support/json-value.ts";
import {
  fileImage,
  assertCustody,
  inside,
  captureDeclarationGraph,
  type FileImage,
} from "../../../support/release/computed_event_custody.ts";
import { assertNativeIdentity } from "../../../support/release/computed_event_names.ts";
import { rejectOverrides } from "../../../support/release/public_install/identity.ts";
import {
  packageMetadata,
  read,
  readBytes,
} from "../../../support/release/public_install/registry.ts";
import { assertPackedManifest } from "../../../support/release/slot_instance_public/packed-manifest.ts";
import {
  managerImages,
  assertManager,
  shippedImages,
  projectBin,
} from "../public-typechecker-11317/install.ts";
import {
  requireNativeLoadPrerequisite,
  recheckNativeLoadPrerequisite,
} from "../../../support/release/human_reports_11252_11253_public/native-loader-prerequisite.ts";
import type { PackageBase } from "../../../support/release/human_reports_11252_11253_public/model.ts";
import { authenticateProvenance, validateNative, type authenticate } from "./authority.ts";
import { profileFor, sources, limitations, type Role, type SourceFile } from "./contract.ts";
import { capture, healthy, inputImages, type Recorder } from "./execute.ts";

export function sourceBin(root: string, name: "vize" | "vue-tsc", files: FileImage[]) {
  const directory = realpathSync(join(root, "node_modules", name));
  const manifest = asRecord(JSON.parse(readFileSync(join(directory, "package.json"), "utf8")));
  const declared = typeof manifest.bin === "string" ? manifest.bin : asRecord(manifest.bin)[name];
  const target = realpathSync(join(directory, asString(declared)));
  assert(inside(directory, target));
  const owned = files.filter((file) => file.path === target);
  assert.equal(owned.length, 1);
  assert.deepEqual(fileImage(target), owned[0]);
  const path = join(root, "node_modules/.bin", name);
  assert.equal(lstatSync(path).isSymbolicLink(), true);
  assert.equal(realpathSync(path), target);
  return { path, target, link: readlinkSync(path), image: owned[0]! };
}
export async function install(
  root: string,
  role: Role,
  original: SourceFile[],
  npm: string,
  npx: string,
  authority: Awaited<ReturnType<typeof authenticate>>,
  record: Recorder,
  cache: Map<string, Buffer>,
) {
  rejectOverrides(process.env);
  assert.equal(process.version, "v24.22.1");
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  const manager = managerImages(npm, npx),
    checkManager = () => assertManager(manager, npm, npx);
  mkdirSync(root);
  const initial = sources(original, role, "Original");
  for (const file of initial) {
    const path = join(root, file.path);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, file.source, { flag: "wx" });
  }
  checkManager();
  const version = capture(record, "npm-version", [process.execPath, npm, "--version"], root);
  healthy(version, 0);
  assert.equal(version.stdout, "11.19.1\n");
  assert.equal(version.stderr, "");
  record.json("install-input.json", { manager, sourceFiles: initial, limitations });
  checkManager();
  const attempt = capture(record, "npm-install", [process.execPath, npm, "install"], root);
  healthy(attempt, 0);
  inputImages(root, initial);
  const lockPath = join(root, "package-lock.json"),
    lockStat = lstatSync(lockPath);
  assert(lockStat.isFile() && !lockStat.isSymbolicLink(), "lock must be a regular physical file");
  assert(realpathSync(lockPath) === lockPath, "lock path must remain in the canonical project");
  for (const name of ["package.json", "package-lock.json"])
    if (existsSync(join(root, name)))
      writeFileSync(join(record.root, "installed-" + name), readFileSync(join(root, name)));
  const lock = asRecord(JSON.parse(readFileSync(join(root, "package-lock.json"), "utf8")));
  assert.equal(lock.lockfileVersion, 3);
  const rows = asRecord(lock.packages),
    requested = asRecord(JSON.parse(initial[0]!.source));
  assert.deepEqual(asRecord(rows[""]).devDependencies, requested.devDependencies);
  const modules = realpathSync(join(root, "node_modules")),
    profile = profileFor(role);
  const registry: unknown[] = [],
    packages: PackageBase[] = [],
    images: FileImage[] = [];
  const reader = async (url: string) => {
    const value = await read(url);
    registry.push({ url, value });
    record.json("registry-observations.json", registry);
    return value;
  };
  const absent: unknown[] = [];
  for (const [path, value] of Object.entries(rows)) {
    if (path === "") continue;
    assert(
      path.startsWith("node_modules/") && !path.includes("\\") && !path.split("/").includes(".."),
    );
    const row = asRecord(value);
    assert.equal(row.link, undefined);
    if (!existsSync(join(root, path))) {
      assert.equal(row.optional, true, "missing required installed package");
      absent.push({ path, row });
      continue;
    }
    const directory = realpathSync(join(root, path));
    assert(inside(modules, directory));
    const raw = readFileSync(join(directory, "package.json")),
      manifest = asRecord(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(raw)));
    const name = asString(manifest.name),
      packageVersion = asString(manifest.version);
    assert.equal(packageVersion, row.version);
    const vize = name === "vize" || name.startsWith("@vizejs/");
    if (vize) {
      assert.equal(packageVersion, profile.version);
      const sri = asRecord(asRecord(authority.capsule.npmSRI)[name]);
      assert.equal(row.integrity, sri.integrity);
      assert.equal(row.resolved, sri.tarball);
    }
    if (path === "node_modules/vue")
      assert.equal(packageVersion, asRecord(requested.devDependencies).vue);
    if (path === "node_modules/typescript") assert.equal(packageVersion, "6.0.3");
    if (path === "node_modules/vue-tsc") assert.equal(packageVersion, "3.0.8");
    let metadata: unknown;
    await packageMetadata(name, packageVersion, row, undefined, async (url) => {
      metadata = await reader(url);
      return metadata;
    });
    const provenance = vize
      ? await authenticateProvenance(role, name, asRecord(asRecord(metadata).dist), reader)
      : { vizeSourceClaimed: false };
    const integrity = asString(row.integrity);
    let payload = cache.get(integrity);
    if (!payload) {
      payload = await readBytes(asString(row.resolved));
      cache.set(integrity, payload);
    }
    const checked = shippedImages(directory, payload, integrity);
    assert.deepEqual(checked.archive.manifest, raw);
    if (name === "vize")
      assertPackedManifest(
        raw,
        checked.archive.manifest,
        asRecord(
          JSON.parse(Buffer.from(authority.sourceFiles[0]!.rawBase64, "base64").toString("utf8")),
        ),
      );
    packages.push({
      name,
      version: packageVersion,
      directory,
      integrity,
      resolved: asString(row.resolved),
      archiveSha512: checked.archive.tarballSHA512,
      files: checked.images,
    });
    images.push(...checked.images);
    record.json("package-authority-" + packages.length + ".json", {
      path,
      name,
      version: packageVersion,
      provenance,
      manifestBase64: raw.toString("base64"),
      files: checked.archive.files,
      tarballSHA512: checked.archive.tarballSHA512,
      completeNodeModulesFileSetAsserted: false,
    });
  }
  record.json("absent-optional-packages.json", absent);
  for (const name of [
    "vize",
    "@vizejs/native",
    "@vizejs/native-linux-x64-gnu",
    "vue",
    "typescript",
    "vue-tsc",
  ])
    assert(packages.some((pkg) => pkg.name === name));
  const cliRoot = realpathSync(join(modules, "vize")),
    cli = join(cliRoot, "bin/vize");
  const graph = captureDeclarationGraph(cliRoot);
  assert.deepEqual(
    graph.files.map((file) => relative(cliRoot, file.path)).sort(),
    authority.sourceFiles
      .slice(1)
      .map((file) => file.file)
      .sort(),
  );
  for (const file of graph.files) {
    const expected = authority.sourceFiles.find(
      (source) => source.file === relative(cliRoot, file.path),
    );
    assert(expected);
    assert.equal(file.bytes, expected.bytes);
    assert.equal(file.sha256, expected.sha256);
  }
  const bindings = [sourceBin(root, "vize", images), sourceBin(root, "vue-tsc", images)];
  assert.deepEqual(projectBin(root, cli).resolved, cli);
  const nativeLoad = requireNativeLoadPrerequisite(
    root,
    packages,
    graph.files,
    {
      version: profile.version,
      candidate: profile.candidate,
      source: profile.source,
      workflowHead: authority.ownership.workflowHead,
      run: profile.run,
      capsule: authority.capsule,
      sdk: graph.files,
      observations: [],
      produced: authority.ownership.produced,
    },
    process.env,
    (kind, value) => record.json("native-prerequisite-" + kind + ".json", value),
  );
  record.json("native-prerequisite.json", nativeLoad);
  checkManager();
  const require = createRequire(cli),
    native = asRecord(require("@vizejs/native"));
  const loaded = Object.keys(require.cache).filter(
    (path) => path.endsWith(".node") && inside(modules, path),
  );
  assert.equal(loaded.length, 1);
  const binary = fileImage(loaded[0]!);
  assert(inside(realpathSync(join(modules, "@vizejs/native-linux-x64-gnu")), binary.path));
  const exported = asRecord(require.cache[binary.path]?.exports),
    identity = asRecord(require(join(modules, "@vizejs/native/native-binding-identity.js")));
  assert.equal(typeof native.getNativeBuildToken, "function");
  assert.equal(typeof exported.getNativeBuildToken, "function");
  assert.equal(typeof identity.getBindingIdentity, "function");
  const token: unknown = Reflect.apply(native.getNativeBuildToken as () => unknown, native, []);
  const imageToken: unknown = Reflect.apply(
    exported.getNativeBuildToken as () => unknown,
    exported,
    [],
  );
  const binding: unknown = Reflect.apply(
    identity.getBindingIdentity as (value: unknown) => unknown,
    identity,
    [exported],
  );
  record.json("native-companion.json", {
    binary,
    token,
    imageToken,
    binding,
    companionPid: process.pid,
    selectedOriginalCliPid: null,
    selectedCorsaImage: "UNKNOWN",
    backend: "UNKNOWN",
    scope: "Vize native companion only; not original CLI or Corsa compiler identity",
  });
  assertNativeIdentity(token, imageToken, binding);
  validateNative(role, authority.capsule, binary, asString(token));
  const all = new Map<string, FileImage>();
  for (const image of [
    ...images,
    ...graph.files,
    binary,
    ...manager,
    fileImage(join(root, "package-lock.json")),
  ]) {
    const previous = all.get(image.path);
    if (previous) assert.deepEqual(previous, image);
    else all.set(image.path, image);
  }
  const custody = [...all.values()];
  const verify = () => {
    checkManager();
    assertCustody(custody);
    assert.deepEqual(
      [sourceBin(root, "vize", images), sourceBin(root, "vue-tsc", images)],
      bindings,
    );
    recheckNativeLoadPrerequisite({ root, packages, sdk: graph.files, nativeLoad }, process.env);
  };
  verify();
  record.json("installed-identity.json", {
    profile,
    graph,
    custody,
    bindings,
    defaultProvider: "UNKNOWN",
    noNativeCompilerAliasAdded: true,
    completeNodeModulesFileSetAsserted: false,
    limitations,
  });
  return { root, custody, verify };
}
