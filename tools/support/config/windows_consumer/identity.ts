import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { asRecord, asString } from "../../json-value.ts";
const app = process.argv[2];
assert.ok(app);
const fromApp = createRequire(join(resolve(app), "package.json"));
const cliEntry = fromApp.resolve("vize");
const cliRoot = resolve(dirname(cliEntry), "..");
const fromCli = createRequire(cliEntry);
const nativeEntry = fromCli.resolve("@vizejs/native");
const fromNative = createRequire(nativeEntry);
const native: { getNativeBuildToken(): string } = fromCli("@vizejs/native");
const identity = asRecord(fromNative("./native-binding-identity.js").getBindingIdentity(native));
assert.equal(identity.kind, "native");
assert.equal(identity.token, native.getNativeBuildToken());
const suffix =
  process.platform === "win32"
    ? "win32-x64-msvc"
    : process.platform === "darwin"
      ? "darwin-" + process.arch
      : "linux-x64-gnu";
const binary = fromNative.resolve("@vizejs/native-" + suffix);
const digest = (file: string) => ({
  path: file,
  bytes: readFileSync(file).length,
  sha256: createHash("sha256").update(readFileSync(file)).digest("hex"),
});
const wrappers: ReturnType<typeof digest>[] = [];
for (const directory of [join(cliRoot, "bin"), join(cliRoot, "dist"), dirname(nativeEntry)])
  for (const item of readdirSync(directory, { withFileTypes: true }))
    if (item.isFile() && !item.name.endsWith(".node") && !item.name.endsWith(".wasm"))
      wrappers.push(digest(join(directory, item.name)));
console.log(
  JSON.stringify({
    node: process.version,
    execPath: process.execPath,
    identity,
    binary: digest(binary),
    wrappers,
    cliVersion: asString(asRecord(fromCli("../package.json")).version),
    nativeVersion: asString(asRecord(fromNative("./package.json")).version),
    platformVersion: asString(
      asRecord(fromNative("@vizejs/native-" + suffix + "/package.json")).version,
    ),
    vueVersion: fromApp("vue/package.json").version,
    pluginVueVersion: fromApp("@vitejs/plugin-vue/package.json").version,
    vitePlusVersion: fromApp("vite-plus/package.json").version,
    targetPlatform:
      process.platform === "win32"
        ? "win32-x64"
        : process.platform === "darwin"
          ? "darwin-" + process.arch
          : "linux-x64",
  }),
);
