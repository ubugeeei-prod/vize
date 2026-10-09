import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

export const runtimeOverrides = [
  "NODE_OPTIONS",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "VIZE_PUBLIC_NATIVE_CUSTODY",
  "VIZE_OXLINT_NATIVE_CUSTODY",
];

export function rejectOverrides(environment = process.env) {
  for (const name of runtimeOverrides) {
    assert.ok(!environment[name], `runtime override must be absent: ${name}`);
  }
}

export function installedPackages(version: string) {
  rejectOverrides();
  assert.match(version, /^0\.[1-9][0-9]*\.0$/u);
  const consumer = realpathSync(process.cwd());
  const manifest = JSON.parse(readFileSync(path.join(consumer, "package.json"), "utf8"));
  const lock = JSON.parse(readFileSync(path.join(consumer, "package-lock.json"), "utf8"));
  assert.equal(manifest.private, true);
  assert.equal(lock.lockfileVersion, 3);
  const records = [];
  type Entry = {
    version: string;
    link?: boolean;
    optional?: boolean;
    resolved: string;
    integrity: string;
  };
  for (const [location, entry] of Object.entries(lock.packages) as [string, Entry][]) {
    const name = location.split("node_modules/").at(-1)!;
    if (!name.startsWith("@vizejs/") && name !== "vize") continue;
    assert.equal(entry.version, version, name);
    assert.equal(entry.link, undefined, `${name} must come from public npm`);
    assert.equal(new URL(entry.resolved).origin, "https://registry.npmjs.org");
    assert.match(entry.integrity, /^sha512-[A-Za-z0-9+/]+={0,2}$/u);
    const identity = { name, version, resolved: entry.resolved, integrity: entry.integrity };
    if (!existsSync(path.join(consumer, location))) {
      assert.equal(entry.optional, true, `${name} missing nonoptional public installation`);
      records.push({ ...identity, installed: false as const });
      continue;
    }
    const directory = realpathSync(path.join(consumer, location));
    assert.ok(directory.startsWith(consumer + path.sep + "node_modules" + path.sep));
    const bytes = readFileSync(path.join(directory, "package.json"));
    const actual = JSON.parse(bytes.toString("utf8"));
    assert.equal(actual.name, name);
    assert.equal(actual.version, version);
    records.push({
      ...identity,
      installed: true as const,
      directory,
      manifestSha256: createHash("sha256").update(bytes).digest("hex"),
    });
  }
  for (const name of [
    "@vizejs/native",
    "@vizejs/native-linux-x64-gnu",
    "@vizejs/vite-plugin",
    "@vizejs/vite-plugin-musea",
  ]) {
    assert.equal(
      manifest.dependencies[name],
      version,
      `${name} must be an exact public dependency`,
    );
    assert.ok(
      records.some((record) => record.name === name && record.installed),
      name,
    );
  }
  return records;
}

export function publicNative(version: string) {
  const packages = installedPackages(version);
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  const require = createRequire(path.join(process.cwd(), "package.json"));
  const native = require("@vizejs/native");
  const entry = realpathSync(require.resolve("@vizejs/native-linux-x64-gnu"));
  const provider = packages.find((record) => record.name === "@vizejs/native-linux-x64-gnu");
  assert.ok(provider?.installed, "mandatory Linux native provider must be installed");
  assert.ok(entry.startsWith(provider.directory + path.sep));
  assert.equal(path.extname(entry), ".node");
  assert.equal(require.cache[entry]?.loaded, true, "public native provider must load successfully");
  assert.equal(
    require.cache[entry]?.exports,
    native,
    "umbrella exports must be the loaded provider",
  );
  return {
    native,
    packages,
    loaded: {
      entry,
      sha256: createHash("sha256").update(readFileSync(entry)).digest("hex"),
      successfulReturnObserved: true,
    },
  };
}
