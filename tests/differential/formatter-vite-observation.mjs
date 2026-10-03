import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { bytes, hash } from "../../npm/native/scripts/formatter-history-build.mjs";
import { nativePreparationIsActive } from "../../npm/native/scripts/test-preparation.mjs";
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
              get: completeConfig(descriptor.get, functions, [...location, savedKey, "get"]),
              set: completeConfig(descriptor.set, functions, [...location, savedKey, "set"]),
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

export function currentAddon(root, receipt, require) {
  const nativeDir = path.join(root, "npm/native");
  assert(nativePreparationIsActive(nativeDir), "original preparation owner is no longer live");
  requirePreparedNative(nativeDir);
  for (const key of ["NAPI_RS_NATIVE_LIBRARY_PATH", "NAPI_RS_FORCE_WASI"])
    assert.equal(process.env[key], undefined, `native loader override: ${key}`);
  const local = path.join(nativeDir, receipt.generated.name);
  assert.equal(hash(fs.readFileSync(local)), receipt.frozen.sha256);
  const entry = require.resolve("@vizejs/native");
  assert.equal(fs.realpathSync(entry), fs.realpathSync(path.join(nativeDir, "index.js")));
  const direct = require(local);
  const publicNative = require(entry);
  for (const key of ["runCli", "normalizeVizeConfig"])
    assert.equal(publicNative[key], direct[key], `public native API bypassed local addon: ${key}`);
  const preparation = JSON.parse(
    fs.readFileSync(path.join(nativeDir, ".artifacts/native/js-test-preparation.json")),
  );
  assert.equal(preparation.sha256, receipt.frozen.sha256);
  for (const key of ["head", "tree", "workingDiff"])
    assert.equal(preparation[key], receipt.source[key]);
  return { local, entry, preparation };
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
