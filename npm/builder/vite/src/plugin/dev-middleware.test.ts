import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import fs from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  collectInspectorSourceFiles,
  createInspectorGraphCache,
  createInspectorGraphPayload,
  installInspectorGraphMiddleware,
  isInspectorGraphRequest,
  type VizeInspectorGraphPayload,
} from "./dev-middleware.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const workspaceRoot = path.resolve(__dirname, "../../../..");
const testRoot = path.join(
  workspaceRoot,
  "target",
  "vize-tests",
  "tests",
  "vite-plugin-vize",
  "dev-middleware",
);
fs.mkdirSync(testRoot, { recursive: true });
const root = fs.mkdtempSync(path.join(testRoot, "inspector-"));

fs.mkdirSync(path.join(root, "src"), { recursive: true });
fs.writeFileSync(
  path.join(root, "src", "App.vue"),
  `<script setup>
import UsedChild from "./UsedChild.vue";
import UnusedChild from "./UnusedChild.vue";
import("./lazy");
</script>
<template>
  <UsedChild />
</template>
`,
);
fs.writeFileSync(path.join(root, "src", "UsedChild.vue"), "<template><p>Used</p></template>\n");
fs.writeFileSync(path.join(root, "src", "UnusedChild.vue"), "<template><p>Unused</p></template>\n");
fs.writeFileSync(path.join(root, "src", "lazy.ts"), "export const value = 1;\n");
fs.writeFileSync(path.join(root, "src", "ignored.d.ts"), "export interface Ignored {}\n");

const state = {
  root,
  scanPatterns: ["src/**/*.vue"],
  ignorePatterns: [],
};

assert.equal(isInspectorGraphRequest("/__vize/inspector/graph"), true);
assert.equal(isInspectorGraphRequest("/__vize/inspector/graph?fresh=1"), true);
assert.equal(isInspectorGraphRequest("/__vize/other"), false);

fs.writeFileSync(path.join(root, "outside.vue"), "<template><p>Outside</p></template>\n");
fs.symlinkSync(path.join(root, "outside.vue"), path.join(root, "src", "Escaped.vue"));

const files = await collectInspectorSourceFiles(state);
assert.deepEqual(
  files.map((file) => file.path),
  ["src/App.vue", "src/UnusedChild.vue", "src/UsedChild.vue", "src/lazy.ts"],
);

const payload = await createInspectorGraphPayload(state);
assert.equal(payload.schema, "vize.inspector.graph");
assert.equal(payload.version, 1);
assert.equal(payload.fileCount, 4);

assert.deepEqual(
  payload.graph.edges.map((edge) => [edge.from, edge.to, edge.kind, edge.specifier]),
  [
    ["src/App.vue", "src/UnusedChild.vue", "import", "./UnusedChild.vue"],
    ["src/App.vue", "src/UsedChild.vue", "component", "./UsedChild.vue"],
    ["src/App.vue", "src/UsedChild.vue", "import", "./UsedChild.vue"],
    ["src/App.vue", "src/lazy.ts", "dynamic-import", "./lazy"],
  ],
);

// The installed route: one graph per filesystem state, for same-site readers only.
type GraphMiddleware = (req: IncomingMessage, res: ServerResponse, next: () => void) => void;

function installGraphRoute(watch: object | null = {}) {
  const middlewares: GraphMiddleware[] = [];
  const watcher = new EventEmitter();
  installInspectorGraphMiddleware(
    {
      config: { server: { watch } },
      middlewares: { use: (middleware: GraphMiddleware) => middlewares.push(middleware) },
      watcher,
    } as never,
    { ...state, logger: { log() {}, info() {}, warn() {}, error() {} } as never },
  );
  assert.equal(middlewares.length, 1);
  const get = (headers: IncomingMessage["headers"] = {}) =>
    new Promise<{ status: number; body: unknown }>((resolve, reject) => {
      const res = {
        statusCode: 0,
        setHeader() {},
        end(body?: string) {
          resolve({ status: res.statusCode, body: JSON.parse(body ?? "null") });
        },
      };
      middlewares[0](
        { method: "GET", url: "/__vize/inspector/graph", headers } as IncomingMessage,
        res as unknown as ServerResponse,
        () => reject(new Error("the graph route must answer its own requests")),
      );
    });
  return { get, watcher };
}

const route = installGraphRoute();
const host = "localhost:5173";
for (const headers of [
  { host, "sec-fetch-site": "cross-site" },
  { host, origin: "https://other.example" },
  { host, origin: "http://localhost:3000" },
  { host, origin: "null" },
]) {
  assert.deepEqual(await route.get(headers), {
    status: 403,
    body: { error: "cross_site_request" },
  });
}

const graphReads = () => {
  const reads: string[] = [];
  const readFileSync = fs.readFileSync;
  fs.readFileSync = ((file: fs.PathOrFileDescriptor, ...rest: unknown[]) => {
    if (typeof file === "string" && file.startsWith(root)) reads.push(path.relative(root, file));
    return (readFileSync as (...args: unknown[]) => unknown)(file, ...rest);
  }) as typeof fs.readFileSync;
  return {
    reads,
    restore() {
      fs.readFileSync = readFileSync;
    },
  };
};
const scanned = ["src/App.vue", "src/UnusedChild.vue", "src/UsedChild.vue", "src/lazy.ts"];
const observed = graphReads();
try {
  // Concurrent requests share one build, and later ones reuse its graph.
  const [first, second] = await Promise.all([
    route.get({ host, "sec-fetch-site": "same-origin" }),
    route.get({ host, origin: `http://${host}` }),
  ]);
  assert.deepEqual(first, { status: 200, body: JSON.parse(JSON.stringify(payload)) });
  assert.deepEqual(second, first);
  assert.deepEqual(await route.get(), first);
  assert.deepEqual(observed.reads, scanned);

  // A watcher event starts the next graph.
  fs.writeFileSync(path.join(root, "src", "added.ts"), "export const added = 1;\n");
  assert.deepEqual(await route.get(), first);
  route.watcher.emit("all", "add", path.join(root, "src", "added.ts"));
  const added = await route.get();
  assert.equal((added.body as { fileCount: number }).fileCount, 5);
  assert.deepEqual(await route.get(), added);
  assert.deepEqual(observed.reads, [
    ...scanned,
    ...scanned.slice(0, 3),
    "src/added.ts",
    "src/lazy.ts",
  ]);
} finally {
  observed.restore();
}

// A server that does not watch has nothing to end a graph's life: each request builds.
const unwatched = installGraphRoute(null);
const unwatchedReads = graphReads();
try {
  await unwatched.get();
  await unwatched.get();
  assert.equal(unwatchedReads.reads.length, 10);
} finally {
  unwatchedReads.restore();
  fs.rmSync(path.join(root, "src", "added.ts"));
}

// The cache itself: a build in flight is shared, a change or a failure starts another.
interface PendingBuild {
  promise: Promise<VizeInspectorGraphPayload>;
  resolve(graph: VizeInspectorGraphPayload): void;
  reject(error: Error): void;
}
const builds: PendingBuild[] = [];
const startBuild = () => {
  let resolve!: PendingBuild["resolve"];
  let reject!: PendingBuild["reject"];
  const promise = new Promise<VizeInspectorGraphPayload>((onResolve, onReject) => {
    resolve = onResolve;
    reject = onReject;
  });
  builds.push({ promise, resolve, reject });
  return promise;
};
const cache = createInspectorGraphCache(startBuild);
const graphOf = (fileCount: number): VizeInspectorGraphPayload => ({ ...payload, fileCount });

const running = cache.get();
assert.equal(cache.get(), running);
builds[0].resolve(graphOf(1));
assert.equal(await running, await cache.get());
assert.equal(builds.length, 1);

// A change during a build leaves that build to its own requests.
cache.invalidate();
const stale = cache.get();
cache.invalidate();
const fresh = cache.get();
builds[1].resolve(graphOf(2));
builds[2].resolve(graphOf(3));
assert.deepEqual([(await stale).fileCount, (await fresh).fileCount], [2, 3]);
assert.equal(cache.get(), fresh);
assert.equal(builds.length, 3);

cache.invalidate();
const failing = cache.get();
builds[3].reject(new Error("unreadable"));
await assert.rejects(failing, { message: "unreadable" });
assert.notEqual(cache.get(), failing);
assert.equal(builds.length, 5);

const shared = createInspectorGraphCache(startBuild, false);
const sharedRun = shared.get();
assert.equal(shared.get(), sharedRun);
builds[5].resolve(graphOf(6));
await sharedRun;
assert.notEqual(shared.get(), sharedRun);
assert.equal(builds.length, 7);
