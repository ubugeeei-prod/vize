import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { hash, json, packageIdentity, type HostSpec } from "../hosts/environment.ts";

export interface RuntimeSpec {
  id: "stable" | "rc9" | "rc11";
  fixture: string;
  version: string;
  target: "vdom" | "vapor";
}
interface RuntimeSettings {
  source: string;
  tarball: string;
  nativeRoot: string;
  output: string;
  tarballSha256: string;
}
interface PackageManifest {
  name: string;
  version: string;
  dependencies: Record<string, string>;
  exports: Record<string, { import: { default: string }; require: { default: string } }>;
}

export { hash, json };
export const here = path.dirname(fileURLToPath(import.meta.url));
export const runtimeSpecs: RuntimeSpec[] = [
  { id: "stable", fixture: "vdom", version: "3.5.44", target: "vdom" },
  { id: "rc9", fixture: "vapor-rc9", version: "3.6.0-rc.9", target: "vapor" },
  { id: "rc11", fixture: "vapor-rc11", version: "3.6.0-rc.11", target: "vapor" },
];
export const hostSpecs: HostSpec[] = [
  { major: 7, version: "7.29.7" },
  { major: 8, version: "8.0.9" },
];
export function exists(file: string) {
  try {
    fs.lstatSync(file);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return false;
    throw error;
  }
}
export function settings(): RuntimeSettings {
  assert.match(process.env.SOURCE_SHA ?? "", /^[a-f0-9]{40}$/, "actual source revision required");
  assert(process.env.VIZE_BABEL_PACKED_PLUGIN, "real packed Babel artifact required");
  assert(process.env.VIZE_BABEL_NATIVE_ROOT, "explicit genuine source-native preparation required");
  const tarball = fs.realpathSync(process.env.VIZE_BABEL_PACKED_PLUGIN);
  const nativeRoot = fs.realpathSync(process.env.VIZE_BABEL_NATIVE_ROOT);
  const output = path.resolve(
    process.env.VIZE_BABEL_RUNTIME_OUTPUT ?? path.join(here, ".artifacts", process.env.SOURCE_SHA!),
  );
  assert(!exists(output), "fresh runtime output preserves earlier whole packets");
  fs.mkdirSync(output, { recursive: true });
  return {
    source: process.env.SOURCE_SHA!,
    tarball,
    nativeRoot,
    output,
    tarballSha256: hash(fs.readFileSync(tarball)),
  };
}
export function runtimeIdentity(spec: RuntimeSpec) {
  const root = path.join(here, "fixtures", spec.fixture);
  const requireRuntime = createRequire(path.join(root, "package.json"));
  const vue = packageIdentity(requireRuntime, "vue");
  const renderer = packageIdentity(requireRuntime, "@vue/server-renderer");
  const playwright = packageIdentity(requireRuntime, "playwright");
  assert.equal(vue.version, spec.version);
  assert.equal(renderer.version, spec.version);
  assert.equal(playwright.version, "1.62.1");
  const browserEntry = path.join(
    vue.root,
    "dist",
    spec.target === "vapor"
      ? "vue.runtime-with-vapor.esm-browser.js"
      : "vue.runtime.esm-browser.js",
  );
  assert(fs.statSync(browserEntry).isFile());
  const fromSfc = createRequire(requireRuntime.resolve("@vue/compiler-sfc"));
  const compilerSsr = packageIdentity(fromSfc, "@vue/compiler-ssr");
  assert.equal(compilerSsr.version, spec.version);
  const compilerVapor =
    spec.target === "vapor" ? packageIdentity(fromSfc, "@vue/compiler-vapor") : null;
  if (compilerVapor) assert.equal(compilerVapor.version, spec.version);
  return {
    ...spec,
    root,
    vue,
    renderer,
    playwright,
    compilerSsr,
    compilerVapor,
    browserEntry,
    browserSha256: hash(fs.readFileSync(browserEntry)),
    lockSha256: hash(fs.readFileSync(path.join(root, "package-lock.json"))),
  };
}
export function prepareCompiler(spec: HostSpec, config: RuntimeSettings) {
  const sourceHost = path.resolve(here, "../hosts/fixtures/babel" + spec.major);
  const fromHost = createRequire(path.join(sourceHost, "package.json"));
  const hostManifest = JSON.parse(
    fs.readFileSync(path.join(sourceHost, "package.json"), "utf8"),
  ) as PackageManifest;
  const root = path.join(config.output, "compiler" + spec.major);
  const modules = path.join(root, "node_modules");
  fs.mkdirSync(modules, { recursive: true });
  fs.writeFileSync(path.join(root, "package.json"), '{"private":true,"type":"module"}\n');
  const dependencies = [
    "@babel/core",
    "@babel/plugin-transform-typescript",
    "@jridgewell/remapping",
    "convert-source-map",
  ].map((name) => {
    const identity = packageIdentity(fromHost, name);
    assert.equal(
      identity.version,
      hostManifest.dependencies[name],
      "exact installed compiler dependency " + name,
    );
    const target = path.join(modules, name);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    assert(!exists(target));
    fs.symlinkSync(identity.root, target, process.platform === "win32" ? "junction" : "dir");
    return identity;
  });
  assert.equal(dependencies[0].version, spec.version);
  assert.equal(Number(dependencies[1].version.split(".")[0]), spec.major);
  const members = execFileSync("tar", ["-tzf", config.tarball], { encoding: "utf8" })
    .trim()
    .split("\n");
  for (const member of members) {
    assert(member.startsWith("package/") && !member.split("/").includes(".."));
    assert(!member.startsWith("package/src/"), "execute packed public distribution only");
  }
  for (const name of ["index.mjs", "index.cjs", "index.d.mts", "index.d.cts"])
    assert(members.includes("package/dist/" + name));
  const temporary = fs.mkdtempSync(path.join(root, "unpack-"));
  const plugin = path.join(modules, "@vizejs/babel-plugin");
  try {
    execFileSync("tar", ["-xzf", config.tarball, "-C", temporary]);
    const original = path.join(temporary, "package");
    const inspect = (directory: string): void => {
      for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
        assert(!entry.isSymbolicLink(), "no source symlink in packed runtime");
        if (entry.isDirectory()) inspect(path.join(directory, entry.name));
      }
    };
    inspect(original);
    fs.mkdirSync(path.dirname(plugin), { recursive: true });
    fs.cpSync(original, plugin, { recursive: true, errorOnExist: true });
  } finally {
    fs.rmSync(temporary, { recursive: true });
  }
  const pkg = JSON.parse(
    fs.readFileSync(path.join(plugin, "package.json"), "utf8"),
  ) as PackageManifest;
  const native = JSON.parse(
    fs.readFileSync(path.join(config.nativeRoot, "package.json"), "utf8"),
  ) as PackageManifest;
  assert.equal(pkg.name, "@vizejs/babel-plugin");
  assert.equal(native.name, "@vizejs/native");
  assert.equal(pkg.version, native.version);
  assert.equal(pkg.exports["."].import.default, "./dist/index.mjs");
  assert.equal(pkg.exports["."].require.default, "./dist/index.cjs");
  assert(["workspace:*", pkg.version].includes(pkg.dependencies["@vizejs/native"]));
  fs.symlinkSync(
    config.nativeRoot,
    path.join(modules, "@vizejs/native"),
    process.platform === "win32" ? "junction" : "dir",
  );
  const identity = {
    ...spec,
    root,
    dependencies,
    plugin: {
      version: pkg.version,
      esmSha256: hash(fs.readFileSync(path.join(plugin, "dist/index.mjs"))),
      cjsSha256: hash(fs.readFileSync(path.join(plugin, "dist/index.cjs"))),
    },
    hostLockSha256: hash(fs.readFileSync(path.join(sourceHost, "package-lock.json"))),
  };
  json(path.join(root, "identity.json"), identity);
  return identity;
}
