import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { copyFileSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { asRecord, asString, readJsonRecord } from "../../json-value.ts";
import { Recorder } from "../../config/windows_consumer/common.ts";
import { assertNativeIdentity, findNpmCli } from "../computed_event_names.ts";
import {
  assertCustody,
  assertDeclarationCustody,
  captureDeclarationGraph,
  fileImage,
  inside,
  sha256,
} from "../computed_event_custody.ts";
import { assertLockedPackage, validateNativeReceipt, type authenticate } from "./authority.ts";
import { installedManifest, type loadOriginal27, type Inputs } from "./fixtures.ts";
import { capture, completed } from "./capture.ts";
import { assertVueProviderLineage, captureVueNamespace } from "./host-versions.ts";
import { nativePrerequisite } from "./native-prerequisite.ts";

/** The original27 role already supplies an explicit checker. Rebase only its
 * frozen historical image; no public resolver module is imported or executed. */
export function originalChecker(root: string, historical: Inputs["historical"]) {
  const target = "@typescript/typescript-linux-x64";
  const owned = "node_modules/" + target + "/lib/tsc";
  assert.ok(
    historical.checker.path.endsWith("/" + owned),
    "exact historical checker suffix required",
  );
  const aliases = historical.identities.filter(
    (row) => row.file === "node_modules/typescript-native/package.json",
  );
  assert.equal(aliases.length, 1);
  const raw = readFileSync(join(root, aliases[0].file));
  assert.equal(raw.length, aliases[0].bytes);
  assert.equal(sha256(raw), aliases[0].sha256);
  const alias = asRecord(JSON.parse(raw.toString("utf8")));
  assert.equal(alias.name, "typescript");
  assert.equal(alias.version, "7.1.0-beta");
  assert.equal(asRecord(alias.optionalDependencies)[target], "7.1.0-beta");
  const targetManifest = readJsonRecord(join(root, "node_modules", target, "package.json"));
  assert.equal(targetManifest.name, target);
  assert.equal(targetManifest.version, "7.1.0-beta");
  const image = fileImage(join(root, owned));
  assert.ok(inside(realpathSync(join(root, "node_modules", target)), image.path));
  assert.ok(inside(realpathSync(join(root, "node_modules")), image.path));
  assert.equal(image.bytes, historical.checker.bytes);
  assert.equal(image.sha256, historical.checker.sha256);
  return {
    image,
    historicalPath: historical.checker.path,
    optionalTarget: target,
    resolverExecuted: false,
    defaultResolverSelectionObserved: false,
  };
}

export async function install(
  root: string,
  version: string,
  authority: Awaited<ReturnType<typeof authenticate>>,
  original: Awaited<ReturnType<typeof loadOriginal27>>,
  record: Recorder,
) {
  mkdirSync(root);
  const manifest = installedManifest(original.provider, version);
  writeFileSync(join(root, "package.json"), JSON.stringify(manifest, null, 2) + "\n", {
    flag: "wx",
  });
  const user = join(record.root, "npm-user.config"),
    global = join(record.root, "npm-global.config");
  writeFileSync(user, "", { flag: "wx" });
  writeFileSync(global, "", { flag: "wx" });
  const npm = findNpmCli();
  completed(
    capture(
      record,
      "public-install",
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
    0,
    false,
  );
  copyFileSync(join(root, "package.json"), join(record.root, "installed-package.json"));
  copyFileSync(join(root, "package-lock.json"), join(record.root, "installed-package-lock.json"));
  const lock = readJsonRecord(join(root, "package-lock.json"));
  assert.equal(lock.lockfileVersion, 3);
  const modules = realpathSync(join(root, "node_modules"));
  const pkg = (name: string, expected: string, packageName = name) => {
    const file = join(modules, name, "package.json"),
      actual = readJsonRecord(file);
    assert.ok(inside(modules, realpathSync(file)));
    assert.equal(actual.name, packageName);
    assert.equal(actual.version, expected);
    return { file, manifest: actual };
  };
  const cliRoot = join(modules, "vize"),
    cli = join(cliRoot, "bin/vize");
  const cliPackage = pkg("vize", version),
    nativePackage = pkg("@vizejs/native", version);
  for (const field of ["name", "version", "typescript", "bin"])
    assert.deepEqual(cliPackage.manifest[field], authority.packageManifest[field], field);
  assert.deepEqual(asRecord(asRecord(cliPackage.manifest.typescript).contentMapper).exec, [
    "node",
    "./bin/vize",
    "content-mapper",
  ]);
  const platformName = "@vizejs/native-linux-x64-gnu";
  const platform = pkg(platformName, version);
  assert.equal(asRecord(nativePackage.manifest.optionalDependencies)[platformName], version);
  for (const name of ["vize", "@vizejs/native", platformName])
    assertLockedPackage(lock, authority.capsule, name, version);
  const graph = captureDeclarationGraph(cliRoot);
  assert.equal(graph.files.length, 10);
  assert.equal(graph.edges.length, 13);
  for (const expected of authority.sdk.filter((file) => file.file !== "package.json")) {
    const actual = fileImage(join(cliRoot, expected.file));
    assert.equal(actual.bytes, expected.bytes);
    assert.equal(actual.sha256, expected.sha256, expected.file);
    writeFileSync(
      join(record.root, "sdk-" + expected.file.replaceAll("/", "--")),
      readFileSync(actual.path),
    );
  }
  const prerequisite = await nativePrerequisite(root, lock, authority, graph.files, record);
  prerequisite.verify(); // No product wrapper or candidate may execute before this guard.
  const fromCli = createRequire(cli);
  const nativeEntry = realpathSync(fromCli.resolve("@vizejs/native"));
  assert.equal(
    realpathSync(join(dirname(nativeEntry), "package.json")),
    realpathSync(nativePackage.file),
  );
  const native = asRecord(fromCli("@vizejs/native"));
  const loaded = Object.keys(fromCli.cache).filter(
    (file) => file.endsWith(".node") && inside(modules, file),
  );
  assert.equal(loaded.length, 1);
  const image = fileImage(loaded[0]);
  assert.ok(inside(realpathSync(dirname(platform.file)), image.path));
  assert.equal(fromCli.cache[image.path]?.loaded, true);
  const imageExports = asRecord(fromCli.cache[image.path]?.exports);
  const helper = asRecord(fromCli(join(modules, "@vizejs/native/native-binding-identity.js")));
  assert.equal(typeof native.getNativeBuildToken, "function");
  assert.equal(typeof imageExports.getNativeBuildToken, "function");
  assert.equal(typeof helper.getBindingIdentity, "function");
  const token = Reflect.apply(native.getNativeBuildToken as () => unknown, native, []);
  const imageToken = Reflect.apply(
    imageExports.getNativeBuildToken as () => unknown,
    imageExports,
    [],
  );
  const binding = Reflect.apply(helper.getBindingIdentity as (value: unknown) => unknown, helper, [
    imageExports,
  ]);
  assertNativeIdentity(token, imageToken, binding);
  validateNativeReceipt(
    authority.capsule,
    {
      target: "linux-x64",
      bytes: image.bytes,
      sha256: image.sha256,
      token: asString(token),
    },
    version,
    authority.candidate,
    authority.ownership.runId,
  );
  const hostImages = captureVueNamespace(modules, record);
  assertVueProviderLineage(
    manifest.overrides,
    readJsonRecord(join(modules, "vue-tsc/package.json")),
  );
  for (const [name, expected] of [
    ["vue", "3.5.42"],
    ["typescript", "6.0.3"],
    ["typescript-native", "7.1.0-beta"],
    ["vue-tsc", "3.3.13"],
  ])
    hostImages.push(
      fileImage(pkg(name, expected, name === "typescript-native" ? "typescript" : name).file),
    );
  const selectedChecker = originalChecker(root, original.inputs.historical);
  record.json("explicit-original27-checker.json", selectedChecker);
  const checker = selectedChecker.image.path;
  const stock = join(modules, "typescript/lib/tsc.js"),
    official = join(modules, "vue-tsc/bin/vue-tsc.js");
  assert.ok(inside(modules, realpathSync(checker)));
  assert.equal(
    realpathSync(createRequire(official).resolve("typescript/package.json")),
    realpathSync(join(modules, "typescript/package.json")),
  );
  for (const [file, expected] of [
    [checker, original.inputs.historical.checker],
    [stock, original.inputs.historical.stock],
    [official, original.inputs.historical.vueTsc],
  ] as const) {
    const actual = fileImage(file);
    assert.equal(actual.bytes, expected.bytes);
    assert.equal(actual.sha256, expected.sha256, file);
  }
  for (const expected of original.inputs.historical.identities) {
    if (!/^node_modules\/(?:vue|typescript|typescript-native|vue-tsc)\//u.test(expected.file))
      continue;
    const actual = fileImage(join(root, expected.file));
    assert.equal(actual.bytes, expected.bytes);
    assert.equal(actual.sha256, expected.sha256, expected.file);
  }
  const candidates = [
    ...prerequisite.images,
    fileImage(join(root, "package.json")),
    fileImage(join(root, "package-lock.json")),
    fileImage(cliPackage.file),
    fileImage(nativePackage.file),
    fileImage(platform.file),
    image,
    fileImage(cli),
    fileImage(join(cliRoot, "dist/cli.mjs")),
    fileImage(checker),
    fileImage(stock),
    fileImage(official),
    fileImage(join(modules, "@typescript/typescript-linux-x64/package.json")),
    fileImage(npm),
    fileImage(nativeEntry),
    ...["native-binding.js", "native-targets.js", "native-binding-identity.js"].map((name) =>
      fileImage(join(modules, "@vizejs/native", name)),
    ),
    ...hostImages,
    ...graph.files,
  ];
  const unique = new Map<string, (typeof candidates)[number]>();
  for (const image of candidates) {
    const previous = unique.get(image.path);
    if (previous)
      assert.deepEqual(image, previous, "same installed image must retain its authenticated bytes");
    unique.set(image.path, image);
  }
  const images = [...unique.values()];
  assertCustody(images);
  record.json("installed-identity.json", {
    version,
    root,
    modules,
    cli,
    native: image,
    token,
    binding,
    checker: fileImage(checker),
    stock: fileImage(stock),
    official: fileImage(official),
    sdkManifest: cliPackage.manifest,
    declarations: graph,
    custody: images,
  });
  const verifyNative = () => {
    assertCustody(images);
    assertDeclarationCustody(cliRoot, graph.files, graph.edges);
    prerequisite.verify();
  };
  return {
    root,
    modules,
    cliRoot,
    cli,
    native: image,
    token,
    checker,
    stock,
    official,
    images,
    graph,
    verifyNative,
  };
}
