import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { readFileSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { asRecord, asString } from "../../json-value.ts";
import { digest, inside } from "./filesystem.ts";

const [rootArg, output] = process.argv.slice(2);
assert.ok(rootArg && output);
const root = realpathSync(rootArg),
  fromApp = createRequire(join(root, "package.json"));
const cliEntry = fromApp.resolve("vize"),
  cliRoot = resolve(dirname(cliEntry), "..");
const fromCli = createRequire(cliEntry),
  nativeEntry = fromCli.resolve("@vizejs/native");
const fromNative = createRequire(nativeEntry),
  native = asRecord(fromCli("@vizejs/native"));
const tokenFn = native.getNativeBuildToken;
assert.equal(typeof tokenFn, "function");
const token: unknown = Reflect.apply(tokenFn as (...args: unknown[]) => unknown, native, []);
const helper = asRecord(fromNative("./native-binding-identity.js"));
assert.equal(typeof helper.getBindingIdentity, "function");
const identity = asRecord(
  Reflect.apply(helper.getBindingIdentity as (...args: unknown[]) => unknown, helper, [native]),
);
assert.equal(identity.kind, "native");
assert.equal(identity.token, token);
const suffix =
  process.platform === "win32"
    ? "win32-x64-msvc"
    : process.platform === "darwin"
      ? "darwin-" + process.arch
      : "linux-x64-gnu";
const platformPackage = "@vizejs/native-" + suffix;
const binary = realpathSync(fromNative.resolve(platformPackage));
const cli = realpathSync(join(cliRoot, "bin/vize"));
const manifest = (file: string) => asRecord(JSON.parse(readFileSync(file, "utf8")) as unknown);
const plugin = realpathSync(fromApp.resolve("@vizejs/vite-plugin/vite-plus"));
const hostVersions = Object.fromEntries(
  ["vite-plus", "vue", "typescript"].map((name) => [
    name,
    asString(manifest(join(root, "node_modules", name, "package.json")).version),
  ]),
);
for (const file of [cliEntry, cli, nativeEntry, binary, plugin])
  assert.ok(inside(root, realpathSync(file)), "public package escaped fresh installation: " + file);
assert.equal(fromNative.cache[binary]?.loaded, true, "actual .node loader returned");
const result = {
  node: process.version,
  execPath: process.execPath,
  platform: process.platform,
  arch: process.arch,
  targetPlatform:
    process.platform === "win32"
      ? "win32-x64"
      : process.platform === "darwin"
        ? "darwin-" + process.arch
        : "linux-x64",
  suffix,
  platformPackage,
  cli,
  cliEntry,
  nativeEntry,
  binary: { path: binary, ...digest(binary) },
  identity,
  cliVersion: asString(manifest(join(cliRoot, "package.json")).version),
  nativeVersion: asString(manifest(join(dirname(nativeEntry), "package.json")).version),
  platformVersion: asString(asRecord(fromNative(platformPackage + "/package.json")).version),
  pluginVersion: asString(manifest(resolve(dirname(plugin), "../package.json")).version),
  hostVersions,
  plugin: { path: plugin, ...digest(plugin) },
  cliWrapper: digest(cli),
  nativeWrapper: digest(nativeEntry),
};
writeFileSync(output, JSON.stringify(result, null, 2) + "\n");
