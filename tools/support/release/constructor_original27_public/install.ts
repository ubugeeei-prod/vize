import assert from "node:assert/strict";
import { createRequire } from "node:module";
import {
  copyFileSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import { asRecord, asString, readJsonRecord } from "../../json-value.ts";
import { Recorder } from "../../config/windows_consumer/common.ts";
import { assertNativeIdentity, findNpmCli } from "../computed_event_names.ts";
import {
  assertCustody,
  captureDeclarationGraph,
  fileImage,
  inside,
} from "../computed_event_custody.ts";
import {
  assertLockedPackage,
  validateNativeReceipt,
  type authenticate,
} from "./authority.ts";
import { installedManifest, type loadOriginal27 } from "./fixtures.ts";
import { capture, completed } from "./capture.ts";

export async function install(
  root: string,
  version: string,
  authority: Awaited<ReturnType<typeof authenticate>>,
  original: Awaited<ReturnType<typeof loadOriginal27>>,
  record: Recorder,
) {
  mkdirSync(root);
  const manifest = installedManifest(original.provider, version);
  writeFileSync(
    join(root, "package.json"),
    JSON.stringify(manifest, null, 2) + "\n",
    { flag: "wx" },
  );
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
  copyFileSync(
    join(root, "package.json"),
    join(record.root, "installed-package.json"),
  );
  copyFileSync(
    join(root, "package-lock.json"),
    join(record.root, "installed-package-lock.json"),
  );
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
    assert.deepEqual(
      cliPackage.manifest[field],
      authority.packageManifest[field],
      field,
    );
  assert.deepEqual(
    asRecord(asRecord(cliPackage.manifest.typescript).contentMapper).exec,
    ["node", "./bin/vize", "content-mapper"],
  );
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
  assert.equal(fromCli.cache[image.path]?.loaded, true);
  const imageExports = asRecord(fromCli.cache[image.path]?.exports);
  const helper = asRecord(
    fromCli(join(modules, "@vizejs/native/native-binding-identity.js")),
  );
  assert.equal(typeof native.getNativeBuildToken, "function");
  assert.equal(typeof imageExports.getNativeBuildToken, "function");
  assert.equal(typeof helper.getBindingIdentity, "function");
  const token = Reflect.apply(
    native.getNativeBuildToken as () => unknown,
    native,
    [],
  );
  const imageToken = Reflect.apply(
    imageExports.getNativeBuildToken as () => unknown,
    imageExports,
    [],
  );
  const binding = Reflect.apply(
    helper.getBindingIdentity as (value: unknown) => unknown,
    helper,
    [imageExports],
  );
  assertNativeIdentity(token, imageToken, binding);
  const platformName = "@vizejs/native-linux-x64-gnu";
  const platform = pkg(platformName, version);
  assert.equal(
    asRecord(nativePackage.manifest.optionalDependencies)[platformName],
    version,
  );
  assert.ok(inside(realpathSync(dirname(platform.file)), image.path));
  for (const name of ["vize", "@vizejs/native", platformName])
    assertLockedPackage(lock, authority.capsule, name, version);
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
  const graph = captureDeclarationGraph(cliRoot);
  assert.equal(graph.files.length, 9);
  for (const expected of authority.sdk.filter(
    (file) => file.file !== "package.json",
  )) {
    const actual = fileImage(join(cliRoot, expected.file));
    assert.equal(actual.bytes, expected.bytes);
    assert.equal(actual.sha256, expected.sha256, expected.file);
    writeFileSync(
      join(record.root, "sdk-" + expected.file.replaceAll("/", "--")),
      readFileSync(actual.path),
    );
  }
  const hostImages = [];
  for (const [name, expected] of [
    ["vue", "3.5.42"],
    ["typescript", "6.0.3"],
    ["typescript-native", "7.1.0-beta"],
    ["vue-tsc", "3.3.13"],
  ])
    hostImages.push(
      fileImage(
        pkg(name, expected, name === "typescript-native" ? "typescript" : name)
          .file,
      ),
    );
  for (const name of readdirSync(join(modules, "@vue")))
    hostImages.push(fileImage(pkg("@vue/" + name, "3.5.42").file));
  const resolver = join(modules, "typescript-native/lib/getExePath.js");
  const nativeChecker = asRecord(await import(pathToFileURL(resolver).href));
  assert.equal(typeof nativeChecker.default, "function");
  const checker = asString(
    Reflect.apply(nativeChecker.default as () => unknown, undefined, []),
  );
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
    if (
      !/^node_modules\/(?:vue|typescript|typescript-native|vue-tsc)\//u.test(
        expected.file,
      )
    )
      continue;
    const actual = fileImage(join(root, expected.file));
    assert.equal(actual.bytes, expected.bytes);
    assert.equal(actual.sha256, expected.sha256, expected.file);
  }
  const images = [
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
    fileImage(resolver),
    fileImage(npm),
    fileImage(nativeEntry),
    ...[
      "native-binding.js",
      "native-targets.js",
      "native-binding-identity.js",
    ].map((name) => fileImage(join(modules, "@vizejs/native", name))),
    ...hostImages,
    ...graph.files,
  ];
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
  };
}
