import assert from "node:assert/strict";
import { test } from "node:test";
import {
  resetTypeProjectWatches,
  typeProjectDependencyRoots,
  watchTypeProject,
} from "./type-project-watches.ts";

test("one project registers its shared configuration inputs once per revision", () => {
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

test("a known cached revision is still rejected after its authority changes", () => {
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

test("runtime-only modules make no project query and stale revisions fail closed", () => {
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
