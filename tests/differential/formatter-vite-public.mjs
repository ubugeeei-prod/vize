import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { bytes, hash } from "../../npm/native/scripts/formatter-history-build.mjs";
import { completeConfig } from "./formatter-vite-observation.mjs";

// Independently read from the lock-pinned public core's compatibility source,
// never generated from a captured configuration or invoked accessor values.
export const publicCoreReference = {
  version: "0.2.9",
  moduleSha256: "be70e81f465a56213a5f8bcbcb9cea70bcea315bf02b4cba85bace5335e7352d",
  keys: ["protocol", "host", "port", "clientPort", "path", "timeout", "server"],
  getter: "get() {\n\t\t\treturn serverConfig.ws?.[key];\n\t\t}",
  setter:
    'set(newValue) {\n\t\t\thmrWsOptionsDeprecationCall();\n\t\t\tif (typeof serverConfig.ws === "object") serverConfig.ws[key] = newValue;\n\t\t}',
};

export function qualifyPublicCore(peer, lock) {
  const require = createRequire(peer.path);
  const name = "@voidzero-dev/vite-plus-core";
  const manifestPath = fs.realpathSync(require.resolve(`${name}/package.json`));
  const raw = fs.readFileSync(manifestPath);
  const manifest = JSON.parse(raw);
  const peerManifest = JSON.parse(Buffer.from(peer.package.base64, "base64"));
  assert.equal(manifest.version, publicCoreReference.version);
  assert.equal(peerManifest.dependencies[name], manifest.version);
  const snapshots = Object.entries(lock.snapshots).filter(([key]) =>
    key.startsWith(`vite-plus@${peer.version}(`),
  );
  assert.equal(snapshots.length, 1);
  assert.equal(snapshots[0][1].dependencies[name].split("(")[0], manifest.version);
  const entry = fs.realpathSync(require.resolve(name));
  const module = path.join(path.dirname(manifestPath), "dist/vite/node/chunks/node.js");
  const source = fs.readFileSync(module);
  assert.equal(hash(source), publicCoreReference.moduleSha256);
  const text = source.toString();
  const start = text.indexOf("function setupHmrWsOptionCompat(serverConfig) {");
  const end = text.indexOf("\nfunction mergeConfigRecursively(", start);
  assert(start >= 0 && end > start);
  assert.equal(text.indexOf("function setupHmrWsOptionCompat(serverConfig) {", start + 1), -1);
  const span = source.subarray(
    Buffer.byteLength(text.slice(0, start)),
    Buffer.byteLength(text.slice(0, end)),
  );
  assert(span.toString().includes(publicCoreReference.getter));
  assert(span.toString().includes(publicCoreReference.setter));
  assert.equal(require("vite-plus").mergeConfig, require(name).mergeConfig);
  return {
    version: manifest.version,
    package: bytes(raw),
    manifestPath,
    entry,
    entryBytes: bytes(fs.readFileSync(entry)),
    module,
    moduleBytes: source.length,
    moduleSha256: hash(source),
    compatibilitySpan: {
      start: Buffer.byteLength(text.slice(0, start)),
      end: Buffer.byteLength(text.slice(0, end)),
      bytes: bytes(span),
    },
    samePublicMergeFunction: true,
  };
}

function callable(source, name, length) {
  return {
    type: "function",
    source,
    properties: [
      {
        key: "length",
        descriptor: { value: length, writable: false, enumerable: false, configurable: true },
      },
      {
        key: "name",
        descriptor: { value: name, writable: false, enumerable: false, configurable: true },
      },
    ],
  };
}
const data = (key, value) => ({ key, enumerable: true, configurable: true, writable: true, value });

export function expectedPublic(fixture) {
  const authored = fixture.public;
  // Preserve every authored value, in the property order of the public source.
  const result = completeConfig({
    ...(fixture.base ? { server: authored.server } : {}),
    plugins: authored.plugins,
    lint: authored.lint,
    fmt: authored.fmt,
    pack: authored.pack,
    run: authored.run,
  });
  if (!fixture.base) return result;
  const server = result.properties.find(({ key }) => key === "server").value;
  const port = server.properties.find(({ key }) => key === "port");
  const host = server.properties.find(({ key }) => key === "host");
  assert.equal(server.properties.length, 2);
  server.properties = [
    port,
    data("ws", completeConfig({})),
    data("hmr", {
      type: "object",
      properties: publicCoreReference.keys.map((key) => ({
        key,
        enumerable: true,
        configurable: true,
        kind: "accessor",
        get: callable(publicCoreReference.getter, "get", 0),
        set: callable(publicCoreReference.setter, "set", 1),
      })),
    }),
    host,
  ];
  return result;
}

export function observePublic(value) {
  const functions = [];
  return { value, functions, snapshot: completeConfig(value, functions) };
}
export function publicStrings(snapshot) {
  return {
    ...snapshot,
    properties: snapshot.properties.filter(({ key }) => typeof key === "string"),
  };
}
export function publicIdentity(value, before) {
  const after = observePublic(value);
  assert.equal(value, before.value);
  assert.deepEqual(after.snapshot, before.snapshot);
  assert.equal(after.functions.length, before.functions.length);
  const functions = after.functions.map((current, index) => {
    const previous = before.functions[index];
    assert.deepEqual(current.path, previous.path);
    assert.equal(
      current.value,
      previous.value,
      `public callable identity changed: ${JSON.stringify(current.path)}`,
    );
    return { path: current.path, sameFunction: true };
  });
  return { sameObject: true, functions, snapshot: after.snapshot };
}

export function assertMetadata(metadata, fixture, row) {
  const actual = { ...metadata, config: undefined };
  const expected = {
    config: undefined,
    options: fixture.options,
    lintTypecheck: false,
    lintLocale: undefined,
    lintHelpLevel: undefined,
    fmtIgnorePatterns: undefined,
  };
  row.metadataComparison = { actual: completeConfig(actual), expected: completeConfig(expected) };
  assert.deepEqual(actual, expected);
  assert.equal(typeof metadata.config, "function");
}
