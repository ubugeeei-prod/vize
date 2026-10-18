import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { bytes, hash } from "../../npm/native/scripts/formatter-history-build.mjs";
import { readActiveNativePreparation } from "../../npm/native/scripts/test-preparation.mjs";
import { requirePreparedNative } from "../../npm/builder/vite/scripts/run-tests.mjs";

// Preserve undefined, symbols and the actual callable config source. This is
// object custody, not a claim that arbitrary JS values survive JSON transport.
export function completeConfig(value, functions, location = []) {
  if (value === undefined) return { type: "undefined" };
  if (typeof value === "function") {
    functions?.push({ path: location, value });
    assert.deepEqual(
      Reflect.ownKeys(value),
      ["length", "name"],
      "unsupported callable config properties",
    );
    return {
      type: "function",
      source: value.toString(),
      properties: ["length", "name"].map((key) => ({
        key,
        descriptor: Object.getOwnPropertyDescriptor(value, key),
      })),
    };
  }
  if (value === null || typeof value !== "object") return { type: typeof value, value };
  if (Array.isArray(value))
    return {
      type: "array",
      values: value.map((item, index) => completeConfig(item, functions, [...location, index])),
    };
  assert.equal(Object.getPrototypeOf(value), Object.prototype, "unsupported public config value");
  return {
    type: "object",
    properties: Reflect.ownKeys(value).map((key) => {
      const descriptor = Object.getOwnPropertyDescriptor(value, key);
      const savedKey =
        typeof key === "symbol"
          ? { symbol: key.description ?? null, global: Symbol.keyFor(key) ?? null }
          : key;
      return {
        key: savedKey,
        enumerable: descriptor.enumerable,
        configurable: descriptor.configurable,
        ...(Object.hasOwn(descriptor, "value")
          ? {
              writable: descriptor.writable,
              value: completeConfig(descriptor.value, functions, [...location, savedKey]),
            }
          : {
              kind: "accessor",
              get: completeConfig(
                Object.getOwnPropertyDescriptor(descriptor, "get").value,
                functions,
                [...location, savedKey, "get"],
              ),
              set: completeConfig(
                Object.getOwnPropertyDescriptor(descriptor, "set").value,
                functions,
                [...location, savedKey, "set"],
              ),
            }),
      };
    }),
  };
}

export const snapshot = (directory) =>
  Object.fromEntries(
    fs
      .readdirSync(directory, { recursive: true })
      .sort((left, right) => (left < right ? -1 : left > right ? 1 : 0))
      .flatMap((name) => {
        const file = path.join(directory, String(name));
        return fs.statSync(file).isFile() ? [[String(name), bytes(fs.readFileSync(file))]] : [];
      }),
  );

export function qualifyReference(root, require, version) {
  const yaml = require("yaml");
  const catalog = yaml.parse(fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf8"));
  const documents = yaml.parseAllDocuments(
    fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8"),
  );
  for (const document of documents) assert.deepEqual(document.errors, []);
  const workspaceLocks = documents
    .map((document) => document.toJS())
    .filter((lock) => lock.importers?.["npm/builder/vite"]);
  assert.equal(workspaceLocks.length, 1, "ambiguous or missing workspace lock importer owner");
  const [lock] = workspaceLocks;
  assert.equal(catalog.catalogs.linting.oxfmt, version);
  assert.equal(lock.importers["."].devDependencies.oxfmt.version.split("(")[0], version);
  return lock;
}

export function qualifiedConsumerPeer(root, workspace, packageRequire, rootRequire, lock) {
  const consumerRequire = createRequire(path.join(workspace, "package.json"));
  const peerPath = fs.realpathSync(consumerRequire.resolve("vite-plus/package.json"));
  assert.equal(peerPath, fs.realpathSync(packageRequire.resolve("vite-plus/package.json")));
  const raw = fs.readFileSync(peerPath);
  const peer = JSON.parse(raw);
  const manifestRaw = fs.readFileSync(path.join(root, "npm/builder/vite/package.json"));
  const manifest = JSON.parse(manifestRaw);
  assert.equal(peer.name, "vite-plus");
  assert.equal(
    lock.importers["npm/builder/vite"].devDependencies["vite-plus"].version.split("(")[0],
    peer.version,
  );
  assert(
    rootRequire("semver").satisfies(peer.version, manifest.peerDependencies["vite-plus"]),
    "consumer Vite+ peer is outside published support",
  );
  return {
    path: peerPath,
    package: bytes(raw),
    version: peer.version,
    pluginManifest: bytes(manifestRaw),
    supportedRange: manifest.peerDependencies["vite-plus"],
  };
}

function readCurrentAddon(root, receipt, require) {
  const nativeDir = path.join(root, "npm/native");
  for (const key of ["NAPI_RS_NATIVE_LIBRARY_PATH", "NAPI_RS_FORCE_WASI"])
    assert.equal(process.env[key], undefined, `native loader override: ${key}`);
  const local = path.join(nativeDir, receipt.generated.name);
  const entry = require.resolve("@vizejs/native");
  assert.equal(fs.realpathSync(entry), fs.realpathSync(path.join(nativeDir, "index.js")));
  const preparation = readActiveNativePreparation(nativeDir);
  assert(preparation, "original preparation owner is no longer live");
  assert.equal(preparation.binding, receipt.generated.name);
  assert.equal(preparation.sha256, receipt.frozen.sha256);
  for (const key of ["head", "tree", "workingDiff"])
    assert.equal(preparation[key], receipt.source[key]);
  return { local, entry, preparation };
}

function samePreparation(root, preparation) {
  assert.deepEqual(
    JSON.parse(
      fs.readFileSync(path.join(root, "npm/native/.artifacts/native/js-test-preparation.json")),
    ),
    preparation,
    "preparation receipt changed during observation",
  );
}

function dataProperty(object, key) {
  const descriptor = Object.getOwnPropertyDescriptor(object, key);
  assert(
    descriptor && Object.hasOwn(descriptor, "value"),
    `prepared data property changed: ${key}`,
  );
  return descriptor;
}

export function currentAddon(root, receipt, require) {
  // Keep the expected build identity private from callers of later observations.
  const expected = {
    generated: { name: receipt.generated.name },
    frozen: { sha256: receipt.frozen.sha256 },
    source: Object.fromEntries(
      ["head", "tree", "workingDiff"].map((key) => [key, receipt.source[key]]),
    ),
  };
  const custody = readCurrentAddon(root, expected, require);
  requirePreparedNative(path.join(root, "npm/native"));
  const keys = ["runCli", "normalizeVizeConfig"];
  const providers = [custody.local, custody.entry].map((file) => {
    const exports = require(file);
    const realpath = fs.realpathSync(file);
    const module = dataProperty(require.cache, realpath).value;
    const properties = Object.fromEntries(
      ["loaded", "exports"].map((key) => [key, dataProperty(module, key)]),
    );
    assert.equal(properties.loaded.value, true);
    assert.equal(properties.exports.value, exports);
    const api = Object.fromEntries(keys.map((key) => [key, dataProperty(exports, key)]));
    return { file, realpath, module, properties, exports, api };
  });
  for (const key of keys)
    assert.equal(
      providers[1].api[key].value,
      providers[0].api[key].value,
      `public native API bypassed local addon: ${key}`,
    );
  // Initial loading and descriptor observation can execute code. Keep its
  // independent post-load hash and receipt check after all provider access.
  assert.equal(hash(fs.readFileSync(custody.local)), expected.frozen.sha256);
  samePreparation(root, custody.preparation);
  return {
    ...custody,
    observe() {
      for (const provider of providers) {
        assert.equal(fs.realpathSync(provider.file), provider.realpath);
        assert(
          dataProperty(require.cache, provider.realpath).value === provider.module,
          "prepared module changed",
        );
        for (const [object, properties] of [
          [provider.module, provider.properties],
          [provider.exports, provider.api],
        ]) {
          for (const [key, saved] of Object.entries(properties)) {
            const current = dataProperty(object, key);
            for (const field of ["value", "writable", "enumerable", "configurable"])
              assert(current[field] === saved[field], `prepared property changed: ${key}.${field}`);
          }
        }
      }
      // Verify handles first, then hash fresh bytes. No require/dlopen or export
      // getter runs after this hash; each child retains its own load observations.
      const current = readCurrentAddon(root, expected, require);
      samePreparation(root, current.preparation);
      return current;
    },
  };
}

export function childFrame(command, argv, env, cwd, frame) {
  return new Promise((resolve) => {
    const stdout = [],
      stderr = [];
    const child = spawn(command, argv, { cwd, env, stdio: ["ignore", "pipe", "pipe"] });
    const timeout = setTimeout(() => {
      frame.timedOut = true;
      child.kill("SIGTERM");
    }, 30_000);
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.once("error", (error) => {
      frame.processError = error.message;
    });
    child.once("close", (status, signal) => {
      clearTimeout(timeout);
      Object.assign(frame, {
        exitStatus: status,
        signal,
        stdout: bytes(Buffer.concat(stdout)),
        stderr: bytes(Buffer.concat(stderr)),
      });
      resolve(status ?? (signal === "SIGINT" ? 130 : signal === "SIGTERM" ? 143 : 1));
    });
  });
}
