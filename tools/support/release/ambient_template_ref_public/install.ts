import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { copyFileSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
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
} from "../constructor_original27_public/authority.ts";
import { capture, completed } from "../constructor_original27_public/capture.ts";
import { release, type authenticate } from "./authority.ts";
import { hook } from "../editor_vite_stub/custody.ts";

export const providerVersions = {
  vue: "3.6.0-rc.13",
  typescript: "6.0.3",
  "vue-tsc": "3.3.13",
  "@typescript/typescript-linux-x64": "7.1.0-beta",
} as const;
export const checkerSha256 = "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70";
export function installedManifest() {
  return {
    name: "ambient-template-ref-original-10980",
    private: true,
    type: "module",
    dependencies: { vize: release.version, "@vizejs/native": release.version, ...providerVersions },
  };
}

export async function install(
  root: string,
  authority: Awaited<ReturnType<typeof authenticate>>,
  record: Recorder,
) {
  mkdirSync(root);
  writeFileSync(join(root, "package.json"), JSON.stringify(installedManifest(), null, 2) + "\n", {
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
  for (const file of ["package.json", "package-lock.json"])
    copyFileSync(join(root, file), join(record.root, "installed-" + file));
  const lock = readJsonRecord(join(root, "package-lock.json"));
  assert.equal(lock.lockfileVersion, 3);
  const modules = realpathSync(join(root, "node_modules"));
  const pkg = (name: string, version: string) => {
    const file = join(modules, name, "package.json"),
      manifest = readJsonRecord(file);
    assert.ok(inside(modules, realpathSync(file)));
    assert.equal(manifest.name, name);
    assert.equal(manifest.version, version);
    return { file, manifest };
  };
  const cliRoot = join(modules, "vize"),
    cli = join(cliRoot, "bin/vize");
  const publicCli = pkg("vize", release.version),
    nativePackage = pkg("@vizejs/native", release.version);
  for (const key of ["name", "version", "bin", "typescript"])
    assert.deepEqual(publicCli.manifest[key], authority.packageManifest[key]);
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
  const image = fileImage(loaded[0]);
  assert.equal(require.cache[image.path]?.loaded, true);
  const exported = asRecord(require.cache[image.path]?.exports);
  const helper = asRecord(require(join(modules, "@vizejs/native/native-binding-identity.js")));
  assert.equal(typeof native.getNativeBuildToken, "function");
  assert.equal(typeof exported.getNativeBuildToken, "function");
  assert.equal(typeof helper.getBindingIdentity, "function");
  const token = Reflect.apply(native.getNativeBuildToken as () => unknown, native, []);
  assertNativeIdentity(
    token,
    Reflect.apply(exported.getNativeBuildToken as () => unknown, exported, []),
    Reflect.apply(helper.getBindingIdentity as (value: unknown) => unknown, helper, [exported]),
  );
  const platformName = "@vizejs/native-linux-x64-gnu",
    platform = pkg(platformName, release.version);
  assert.equal(
    asRecord(nativePackage.manifest.optionalDependencies)[platformName],
    release.version,
  );
  assert.ok(inside(realpathSync(dirname(platform.file)), image.path));
  for (const name of ["vize", "@vizejs/native", platformName])
    assertLockedPackage(lock, authority.capsule, name, release.version);
  validateNativeReceipt(
    authority.capsule,
    { target: "linux-x64", bytes: image.bytes, sha256: image.sha256, token: asString(token) },
    release.version,
    authority.candidate,
    release.run,
  );
  const graph = captureDeclarationGraph(cliRoot);
  assert.equal(graph.files.length, 9);
  for (const expected of authority.sdk.filter((file) => file.file !== "package.json")) {
    const actual = fileImage(join(cliRoot, expected.file));
    assert.equal(actual.bytes, expected.bytes);
    assert.equal(actual.sha256, expected.sha256);
    writeFileSync(
      join(record.root, "sdk-" + expected.file.replaceAll("/", "--")),
      readFileSync(actual.path),
    );
  }
  const providers = Object.entries(providerVersions).map(([name, version]) =>
    fileImage(pkg(name, version).file),
  );
  const official = join(modules, "vue-tsc/bin/vue-tsc.js");
  assert.equal(
    realpathSync(createRequire(official).resolve("typescript/package.json")),
    realpathSync(join(modules, "typescript/package.json")),
  );
  assert.equal(
    asRecord(readJsonRecord(join(modules, "vue-tsc/package.json")).dependencies)[
      "@vue/language-core"
    ],
    "3.3.13",
  );
  const checker = realpathSync(join(modules, "@typescript/typescript-linux-x64/lib/tsc"));
  assert.ok(inside(modules, checker));
  assert.equal(fileImage(checker).sha256, checkerSha256);
  const images = [
    fileImage(join(root, "package.json")),
    fileImage(join(root, "package-lock.json")),
    fileImage(publicCli.file),
    fileImage(nativePackage.file),
    fileImage(platform.file),
    image,
    fileImage(cli),
    fileImage(join(cliRoot, "dist/cli.mjs")),
    fileImage(nativeEntry),
    fileImage(checker),
    fileImage(official),
    fileImage(join(modules, "typescript/lib/tsc.js")),
    fileImage(npm),
    fileImage(process.execPath),
    fileImage(hook),
    ...["native-binding.js", "native-targets.js", "native-binding-identity.js"].map((name) =>
      fileImage(join(modules, "@vizejs/native", name)),
    ),
    ...providers,
    ...graph.files,
  ];
  assertCustody(images);
  record.json("installed-identity.json", {
    role: "C12-original-10980",
    root,
    modules,
    cli,
    native: image,
    token,
    checker: fileImage(checker),
    providers,
    declarations: graph,
    custody: images,
  });
  return {
    root,
    modules,
    cliRoot,
    cli,
    native: image,
    token: asString(token),
    checker,
    images,
    graph,
  };
}
