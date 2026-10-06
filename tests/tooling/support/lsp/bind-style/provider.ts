import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const sha = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function inventory(directory: string): Record<string, { bytes: number; sha256: string }> {
  const result: Record<string, { bytes: number; sha256: string }> = {};
  for (const entry of fs.readdirSync(directory).sort()) {
    if (entry === "node_modules") continue;
    const filename = path.join(directory, entry);
    const stat = fs.lstatSync(filename);
    assert.equal(stat.isSymbolicLink(), false, "provider package member must be physical");
    if (stat.isDirectory())
      for (const [child, pin] of Object.entries(inventory(filename)))
        result[`${entry}/${child}`] = pin;
    else {
      assert.ok(stat.isFile());
      const bytes = fs.readFileSync(filename);
      result[entry] = { bytes: bytes.length, sha256: sha(bytes) };
    }
  }
  return result;
}
function packageRoot(from: string, name: string, manifestName = name): string {
  const require = createRequire(path.join(from, "package.json"));
  let resolved: string;
  try {
    resolved = require.resolve(`${name}/package.json`);
  } catch (error) {
    assert.equal((error as NodeJS.ErrnoException).code, "ERR_PACKAGE_PATH_NOT_EXPORTED");
    resolved = require.resolve(name);
  }
  let directory = path.dirname(fs.realpathSync(resolved));
  while (true) {
    const manifest = path.join(directory, "package.json");
    if (
      fs.existsSync(manifest) &&
      JSON.parse(fs.readFileSync(manifest, "utf8")).name === manifestName
    )
      return directory;
    const parent = path.dirname(directory);
    assert.notEqual(parent, directory, `physical provider root missing: ${name}`);
    directory = parent;
  }
}

export function prepareProviders(root: string, project: string, publicScope: boolean) {
  const requested = process.env.VIZE_BIND_STYLE_VUE_ROOT;
  if (publicScope) assert.ok(requested, "public original requires the exact original Vue cohort");
  const vue = requested
    ? fs.realpathSync(requested)
    : packageRoot(path.join(root, "tests"), "vue-bind-style-oracle", "vue");
  const packages = new Map<string, { root: string; manifest: unknown; members: unknown }>();
  function add(directory: string) {
    directory = fs.realpathSync(directory);
    const manifest = JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8"));
    const prior = packages.get(manifest.name);
    if (prior) {
      assert.equal(prior.root, directory, "one physical provider per package name");
      return;
    }
    packages.set(manifest.name, { root: directory, manifest, members: inventory(directory) });
    for (const name of Object.keys(manifest.dependencies ?? {})) add(packageRoot(directory, name));
  }
  add(vue);
  const vueManifest = JSON.parse(fs.readFileSync(path.join(vue, "package.json"), "utf8"));
  assert.equal(vueManifest.name, "vue");
  assert.equal(vueManifest.version, "3.5.43");
  for (const [name, value] of packages)
    if (name.startsWith("@vue/"))
      assert.equal((value.manifest as { version: string }).version, vueManifest.version);
  const suffix = `${process.platform === "darwin" ? "darwin" : process.platform}-${process.arch}`;
  const sdkName = `@typescript/typescript-${suffix}`;
  const sdk = process.env.VIZE_BIND_STYLE_SDK_ROOT
    ? fs.realpathSync(process.env.VIZE_BIND_STYLE_SDK_ROOT)
    : packageRoot(root, sdkName);
  const manifestBytes = fs.readFileSync(path.join(sdk, "package.json"));
  const sdkManifest = JSON.parse(manifestBytes.toString());
  assert.equal(sdkManifest.name, sdkName);
  assert.equal(sdkManifest.version, "7.0.2");
  const binary = fs.realpathSync(path.join(sdk, "lib/tsc"));
  assert.equal(path.dirname(path.dirname(binary)), sdk);
  const binarySha256 = sha(fs.readFileSync(binary));
  if (publicScope) {
    assert.equal(suffix, "darwin-arm64");
    assert.equal(binarySha256, "a82f731365ad69d5c4c15f5e18fba4584bf3b7b839960172a76c3462b5114bf2");
    assert.equal(
      sha(manifestBytes),
      "e88558e22e3db6c4da920e15a60eb2bbea801732b94aedb67e972be2a7f485b8",
    );
  }
  for (const key of ["CORSA_PATH", "CORSA_EXECUTABLE", "TSGO_PATH", "TSGO_EXECUTABLE"])
    assert.ok(!process.env[key], `original default discovery refuses ambient ${key}`);
  const links: Array<{ requested: string; physical: string }> = [];
  for (const [name, value] of packages) link(name, value.root);
  link(sdkName, sdk);
  fs.mkdirSync(path.join(project, "node_modules/.bin"), { recursive: true });
  const wrapper = path.join(project, "node_modules/.bin/tsgo");
  fs.symlinkSync(binary, wrapper);
  assert.equal(fs.realpathSync(wrapper), binary);
  links.push({ requested: wrapper, physical: binary });
  return {
    reportedVue: "3.5.43",
    actualVue: vueManifest.version,
    node: process.version,
    platform: process.platform,
    arch: process.arch,
    packages: Object.fromEntries(packages),
    sdk: {
      root: sdk,
      manifest: sdkManifest,
      manifestSha256: sha(manifestBytes),
      binary,
      binarySha256,
      members: inventory(sdk),
    },
    links,
    route: "original config and default project-local native discovery; no explicit runtime option",
  };
  function link(name: string, physical: string) {
    const requested = path.join(project, "node_modules", name);
    fs.mkdirSync(path.dirname(requested), { recursive: true });
    fs.symlinkSync(physical, requested);
    assert.equal(fs.realpathSync(requested), physical);
    links.push({ requested, physical });
  }
}
