import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { copyFileSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { pathToFileURL } from "node:url";
import { asRecord, asString, parseJson, readJsonRecord } from "../../json-value.ts";
import type { Recorder } from "../../config/windows_consumer/common.ts";
import { assertNativeIdentity, findNpmCli } from "../computed_event_names.ts";
import {
  assertCustody,
  captureDeclarationGraph,
  fileImage,
  inside,
} from "../computed_event_custody.ts";
import { hook, observed, verifyJournal } from "../editor_vite_stub/custody.ts";
import { authenticate, assertLockedPackage, validateNative } from "./authority.ts";
import { assertGraph, profiles, type Role } from "./contract.ts";
import { assertJournalPid, capture } from "./process.ts";
import { libraryImages, libraryVersion, mergeImages } from "./libraries.ts";
import { authenticatePackedManifest, captureInstalledManifests } from "./packed-manifest.ts";

export function providerManifest(role: Role) {
  const vueNames = [
    "compiler-core",
    "compiler-dom",
    "compiler-sfc",
    "compiler-ssr",
    "reactivity",
    "runtime-core",
    "runtime-dom",
    "server-renderer",
    "shared",
  ];
  return {
    private: true,
    type: "module",
    dependencies: {
      vize: profiles[role].version,
      "@vizejs/native": profiles[role].version,
      vue: "3.5.42",
      typescript: "6.0.3",
      "typescript-native": "npm:typescript@7.1.0-beta",
    },
    overrides: Object.fromEntries(vueNames.map((name) => ["@vue/" + name, "3.5.42"])),
  };
}
function successful(value: ReturnType<typeof capture>) {
  assert.equal(value.error, null);
  assert.equal(value.signal, null);
  assert.equal(value.status, 0, value.stdout + value.stderr);
  assert.ok(Number.isSafeInteger(value.pid) && value.pid > 0);
}
export async function install(
  root: string,
  role: Role,
  authority: Awaited<ReturnType<typeof authenticate>>,
  record: Recorder,
) {
  mkdirSync(root);
  const version = profiles[role].version;
  writeFileSync(
    join(root, "package.json"),
    JSON.stringify(providerManifest(role), null, 2) + "\n",
    { flag: "wx" },
  );
  const user = join(record.root, "npm-user.config"),
    global = join(record.root, "npm-global.config");
  writeFileSync(user, "", { flag: "wx" });
  writeFileSync(global, "", { flag: "wx" });
  const npm = findNpmCli();
  successful(
    capture(
      record,
      "registry-install",
      [
        process.execPath,
        npm,
        "install",
        "--ignore-scripts",
        "--include=optional",
        "--no-audit",
        "--no-fund",
        "--package-lock=true",
        "--install-strategy=hoisted",
        "--registry=https://registry.npmjs.org",
        "--userconfig=" + user,
        "--globalconfig=" + global,
      ],
      root,
    ),
  );
  for (const file of ["package.json", "package-lock.json"])
    copyFileSync(join(root, file), join(record.root, "installed-" + file));
  const lock = readJsonRecord(join(root, "package-lock.json"));
  assert.equal(lock.lockfileVersion, 3);
  assert.deepEqual(
    asRecord(asRecord(lock.packages)[""]).dependencies,
    providerManifest(role).dependencies,
  );
  const modules = realpathSync(join(root, "node_modules"));
  const capturedManifests = captureInstalledManifests(modules, record);
  record.json("installation-phase.json", { phase: "validate-installed-package-identities" });
  const pkg = (name: string, wanted: string, packageName = name) => {
    const file = join(modules, name, "package.json"),
      raw = readFileSync(file),
      manifest = asRecord(parseJson(new TextDecoder("utf-8", { fatal: true }).decode(raw)));
    const captured = capturedManifests.get(name);
    if (captured) assert.deepEqual(raw, captured.raw, "installed manifest changed after capture");
    assert.ok(inside(modules, realpathSync(file)));
    assert.equal(manifest.name, packageName);
    assert.equal(manifest.version, wanted);
    return { file, manifest };
  };
  const cliRoot = join(modules, "vize"),
    cli = join(cliRoot, "bin/vize");
  const cliPackage = pkg("vize", version),
    nativePackage = pkg("@vizejs/native", version),
    platformName = "@vizejs/native-linux-x64-gnu",
    platform = pkg(platformName, version);
  for (const name of ["vize", "@vizejs/native", platformName])
    assertLockedPackage(lock, authority.capsule, name, version);
  await authenticatePackedManifest(
    authority.capsule,
    authority.packageManifest,
    capturedManifests.get("vize")!.raw,
    record,
  );
  record.json("installation-phase.json", { phase: "validate-native-source-sdk-and-libraries" });
  assert.deepEqual(asRecord(asRecord(cliPackage.manifest.typescript).contentMapper).exec, [
    "node",
    "./bin/vize",
    "content-mapper",
  ]);
  assert.equal(asRecord(nativePackage.manifest.optionalDependencies)[platformName], version);
  const require = createRequire(cli),
    nativeEntry = realpathSync(require.resolve("@vizejs/native"));
  assert.equal(
    realpathSync(join(dirname(nativeEntry), "package.json")),
    realpathSync(nativePackage.file),
  );
  const native = asRecord(require("@vizejs/native"));
  const loaded = Object.keys(require.cache).filter(
    (file) => file.endsWith(".node") && inside(modules, file),
  );
  assert.equal(loaded.length, 1);
  const binary = fileImage(loaded[0]);
  assert.ok(inside(realpathSync(dirname(platform.file)), binary.path));
  const imageExports = asRecord(require.cache[binary.path]?.exports),
    helper = asRecord(require(join(modules, "@vizejs/native/native-binding-identity.js")));
  assert.equal(require.cache[binary.path]?.loaded, true);
  assert.equal(typeof native.getNativeBuildToken, "function");
  assert.equal(typeof imageExports.getNativeBuildToken, "function");
  assert.equal(typeof helper.getBindingIdentity, "function");
  const token = Reflect.apply(native.getNativeBuildToken as () => unknown, native, []),
    imageToken = Reflect.apply(imageExports.getNativeBuildToken as () => unknown, imageExports, []),
    binding = Reflect.apply(helper.getBindingIdentity as (value: unknown) => unknown, helper, [
      imageExports,
    ]);
  assertNativeIdentity(token, imageToken, binding);
  validateNative(role, authority.capsule, binary, asString(token));
  const graph = captureDeclarationGraph(cliRoot);
  assertGraph(role, cliRoot, graph);
  for (const file of graph.files)
    writeFileSync(
      join(record.root, "sdk--" + file.path.slice(cliRoot.length + 1).replaceAll("/", "--")),
      readFileSync(file.path),
      { flag: "wx" },
    );
  const libraries = libraryImages(modules);
  record.json("physical-library-trees.json", libraries);
  for (const name of libraries.names) {
    const row = asRecord(asRecord(lock.packages)["node_modules/" + name]);
    assert.equal(row.version, libraryVersion(name));
    assert.equal(row.link, undefined);
    assert.match(asString(row.integrity), /^sha512-[A-Za-z0-9+/]+={0,2}$/u);
    assert.equal(new URL(asString(row.resolved)).origin, "https://registry.npmjs.org");
  }
  const tsPackage = pkg("typescript", "6.0.3"),
    nativeCheckerPackage = pkg("typescript-native", "7.1.0-beta", "typescript");
  const resolver = join(modules, "typescript-native/lib/getExePath.js"),
    nativeChecker = asRecord(await import(pathToFileURL(resolver).href));
  assert.equal(typeof nativeChecker.default, "function");
  const checker = realpathSync(
    asString(Reflect.apply(nativeChecker.default as () => unknown, undefined, [])),
  );
  assert.ok(inside(modules, checker));
  const checkerPackageFile = join(dirname(dirname(checker)), "package.json");
  const checkerPackage = readJsonRecord(checkerPackageFile);
  assert.equal(checkerPackage.name, "@typescript/typescript-linux-x64");
  assert.equal(checkerPackage.version, "7.1.0-beta");
  const checkerLock = asRecord(
    asRecord(lock.packages)[relative(root, dirname(checkerPackageFile)).replaceAll("\\", "/")],
  );
  assert.equal(checkerLock.version, "7.1.0-beta");
  assert.match(asString(checkerLock.integrity), /^sha512-[A-Za-z0-9+/]+={0,2}$/u);
  assert.equal(new URL(asString(checkerLock.resolved)).origin, "https://registry.npmjs.org");
  const compiler = join(modules, "typescript/bin/tsc"),
    apiPath = join(modules, "typescript/lib/typescript.js");
  const ts = require(
    apiPath,
  ) as typeof import("../../../../npm/cli/node_modules/typescript/lib/typescript.js");
  assert.equal(ts.version, "6.0.3");
  for (const [label, argv, stdout] of [
    ["typescript-version", [process.execPath, compiler, "--version"], "Version 6.0.3\n"],
    ["native-checker-version", [checker, "--version"], "Version 7.1.0-beta\n"],
  ] as const) {
    const packet = capture(record, label, [...argv], root);
    successful(packet);
    assert.equal(packet.stderr, "");
    assert.equal(packet.stdout, stdout);
  }
  const journal = observed(root, { binary }, record, "installed-cli-version");
  const cliVersion = capture(
    record,
    "installed-cli-version",
    [process.execPath, "--require", hook, cli, "--version"],
    root,
    journal.env,
  );
  successful(cliVersion);
  verifyJournal(journal.journal, { binary }, cliVersion.status);
  assertJournalPid(readFileSync(journal.journal, "utf8"), cliVersion.pid);
  assert.ok(cliVersion.stdout.includes(version));
  const images = mergeImages([
    ...[...capturedManifests.values()].map((row) => row.image),
    fileImage(join(root, "package.json")),
    fileImage(join(root, "package-lock.json")),
    fileImage(cliPackage.file),
    fileImage(nativePackage.file),
    fileImage(platform.file),
    binary,
    fileImage(cli),
    fileImage(join(cliRoot, "dist/cli.mjs")),
    fileImage(compiler),
    fileImage(apiPath),
    fileImage(checker),
    fileImage(checkerPackageFile),
    fileImage(resolver),
    fileImage(tsPackage.file),
    fileImage(nativeCheckerPackage.file),
    fileImage(npm),
    fileImage(nativeEntry),
    ...["native-binding.js", "native-targets.js", "native-binding-identity.js"].map((name) =>
      fileImage(join(modules, "@vizejs/native", name)),
    ),
    ...graph.files,
    ...libraries.files,
  ]);
  assertCustody(images);
  record.json("installed-identity.json", {
    role,
    profile: profiles[role],
    root,
    modules,
    cli,
    token,
    binding,
    controllerPid: process.pid,
    binary,
    graph,
    images,
    physicalLibraries: libraries.names,
    typescriptApi: fileImage(apiPath),
    checker: fileImage(checker),
  });
  return { role, root, modules, cliRoot, graph, images, ts, compiler, checker };
}
