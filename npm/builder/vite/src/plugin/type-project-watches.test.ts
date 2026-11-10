import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  resetTypeProjectWatches,
  typeProjectDependencyRoots,
  watchTypeProject,
} from "./type-project-watches.ts";

void test("one project registers its shared configuration inputs once per revision", () => {
  const cache = new Map();
  const watched: string[] = [];
  let reads = 0;
  let checks = 0;
  const api = {
    typeProjectInputs(root: string, revision: string, knownRevision?: string) {
      checks++;
      if (knownRevision === revision) return [];
      reads++;
      return revision === "1"
        ? [root, "/work/projects/other/tsconfig.json"]
        : [root, "/work/projects/new/tsconfig.json"];
    },
    typeProjectOwners(file: string) {
      return file.endsWith("tsconfig.json") ? ["/work/tsconfig.json"] : [];
    },
  };
  for (let i = 0; i < 512; i++)
    watchTypeProject(
      cache,
      { root: "/work/tsconfig.json", revision: "1" },
      (file) => watched.push(file),
      api,
    );
  assert.equal(reads, 1);
  assert.equal(checks, 512, "repeated modules validate only the scalar revision");
  assert.deepEqual(watched, ["/work/tsconfig.json", "/work/projects/other/tsconfig.json"]);
  assert.deepEqual(typeProjectDependencyRoots(cache, "/work/projects/other/tsconfig.json", api), [
    "/work/tsconfig.json",
  ]);
  watchTypeProject(
    cache,
    { root: "/work/tsconfig.json", revision: "2" },
    (file) => watched.push(file),
    api,
  );
  assert.equal(reads, 2);
  assert.deepEqual(watched.slice(2), ["/work/tsconfig.json", "/work/projects/new/tsconfig.json"]);
  resetTypeProjectWatches(cache);
  assert.deepEqual(typeProjectDependencyRoots(cache, "/work/projects/new/tsconfig.json", api), []);
  watchTypeProject(
    cache,
    { root: "/work/tsconfig.json", revision: "2" },
    (file) => watched.push(file),
    api,
  );
  assert.equal(reads, 3, "a new build must register watches again");
});

void test("a known cached revision is still rejected after its authority changes", () => {
  const cache = new Map();
  let current = "1";
  const api = {
    typeProjectInputs(root: string, revision: string, knownRevision?: string) {
      return revision !== current ? null : knownRevision === revision ? [] : [root];
    },
    typeProjectOwners() {
      return ["/work/tsconfig.json"];
    },
  };
  const context = { root: "/work/tsconfig.json", revision: "1" };
  watchTypeProject(cache, context, () => {}, api);
  current = "2";
  assert.throws(
    () => watchTypeProject(cache, context, () => {}, api),
    /TypeScript project changed/,
  );
});

void test("runtime-only modules make no project query and stale revisions fail closed", () => {
  const cache = new Map();
  const api = {
    typeProjectInputs() {
      return null;
    },
    typeProjectOwners() {
      throw new Error("runtime-only HMR must not call native project metadata");
    },
  };
  watchTypeProject(cache, undefined, () => assert.fail("no project inputs"), api);
  assert.deepEqual(typeProjectDependencyRoots(cache, "/work/Ordinary.vue", api), []);
  assert.throws(
    () =>
      watchTypeProject(
        cache,
        { root: "/work/tsconfig.json", revision: "old" },
        () => assert.fail("stale input"),
        api,
      ),
    /TypeScript project changed while compiling/,
  );
  assert.deepEqual(typeProjectDependencyRoots(cache, "/work/Ordinary.vue", api), []);
});

void test("symlinked monorepo inputs retain every physical TypeScript project owner", (t) => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-type-project-link-"));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const physical = path.join(temporary, "packages", "types");
  const logical = path.join(temporary, "apps", "web", "node_modules", "types");
  fs.mkdirSync(physical, { recursive: true });
  fs.mkdirSync(path.dirname(logical), { recursive: true });
  const physicalInput = path.join(physical, "shared.d.ts");
  fs.writeFileSync(physicalInput, "export interface Shared {}\n");
  try {
    fs.symlinkSync(physical, logical, process.platform === "win32" ? "junction" : "dir");
  } catch {
    t.skip("directory symlinks are unavailable");
    return;
  }
  const logicalInput = path.join(logical, "shared.d.ts");
  const physicalIdentity = fs.realpathSync.native(physicalInput);
  const roots = [
    path.join(temporary, "apps", "web", "tsconfig.json"),
    path.join(temporary, "packages", "ui", "tsconfig.json"),
  ];
  const cache = new Map();
  const queried: string[] = [];
  const api = {
    typeProjectInputs(root: string) {
      return [root, logicalInput];
    },
    typeProjectOwners(file: string) {
      queried.push(file);
      return file === physicalIdentity ? roots : [];
    },
  };
  watchTypeProject(cache, { root: roots[0], revision: "1" }, () => {}, api);

  assert.deepEqual(typeProjectDependencyRoots(cache, logicalInput, api), roots);
  assert(queried.includes(logicalInput));
  assert(queried.includes(physicalIdentity));
});

void test("verbatim Windows project inputs are watched once in their plain spelling", () => {
  const watched: string[] = [];
  const api = {
    typeProjectInputs(root: string) {
      return [root, "\\\\?\\C:\\app\\tsconfig.json", "\\\\?\\C:\\app\\src\\tsconfig.json"];
    },
    typeProjectOwners() {
      return [];
    },
  };
  watchTypeProject(
    new Map(),
    { root: "C:\\app\\tsconfig.json", revision: "1" },
    (file) => watched.push(file),
    api,
  );
  assert.deepEqual(watched, ["C:\\app\\tsconfig.json", "C:\\app\\src\\tsconfig.json"]);
});
