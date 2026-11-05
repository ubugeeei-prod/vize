import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import vm from "node:vm";

const filename = new URL("../../npm/typescript-plugin/virtual-modules.cjs", import.meta.url);
const require = createRequire(filename);

function fixture(watch = true, projectVersioned = true) {
  const files = new Map<string, { disk: string; buffer?: string; version: number }>();
  const reads = new Map<string, number>();
  const watchers = new Map<string, () => void>();
  let generationCount = 0,
    optionCopies = 0,
    epoch = 0,
    checkJs = false,
    closedWatchers = 0;
  const settings = new Proxy(
    {},
    {
      ownKeys() {
        optionCopies++;
        return ["checkJs"];
      },
      getOwnPropertyDescriptor() {
        return { value: checkJs, enumerable: true, configurable: true };
      },
      get(_target, key) {
        return key === "checkJs" ? checkJs : undefined;
      },
    },
  );
  const snapshot = (text: string) => ({
    getLength: () => text.length,
    getText: (a: number, b: number) => text.slice(a, b),
  });
  const host = {
    getCompilationSettings: () => settings,
    ...(projectVersioned ? { getProjectVersion: () => String(epoch) } : {}),
    getScriptVersion: (name: string) => String(files.get(name)?.version ?? 0),
    getScriptSnapshot: (name: string) => snapshot(files.get(name)?.buffer ?? ""),
    fileExists: (name: string) => files.has(name),
    readFile(name: string) {
      reads.set(name, (reads.get(name) ?? 0) + 1);
      return files.get(name)?.disk;
    },
  };
  const originalSnapshot = host.getScriptSnapshot;
  type Support = { get(name: string): { source: string; version: string }; dispose(): void };
  const module = {
    exports: {} as { installVueVirtualModules(ts: unknown, info: unknown): Support },
  };
  vm.runInNewContext(fs.readFileSync(filename, "utf8"), {
    module,
    require(name: string) {
      if (name !== "./provider.cjs") return require(name);
      return {
        loadNative: () => ({
          native: {
            generateVueLanguageCode(source: string) {
              generationCount++;
              return source;
            },
          },
          toUtf16LanguageProjection: (_source: string, text: string) => ({ text }),
        }),
      };
    },
  });
  const support = module.exports.installVueVirtualModules(
    { ScriptSnapshot: { fromString: snapshot }, sys: {} },
    {
      languageServiceHost: host,
      project: {
        projectService: {
          getScriptInfo(name: string) {
            return {
              hasMixedContent: true,
              isScriptOpen: () => files.get(name)?.buffer !== undefined,
            };
          },
        },
      },
      serverHost: watch
        ? {
            watchFile(name: string, callback: () => void) {
              watchers.set(name, callback);
              return {
                close() {
                  closedWatchers++;
                  watchers.delete(name);
                },
              };
            },
          }
        : {},
    },
  );
  return {
    files,
    reads,
    watchers,
    host,
    support,
    originalSnapshot,
    update() {
      epoch++;
    },
    options(value: boolean) {
      checkJs = value;
      epoch++;
    },
    counts: () => ({ generationCount, optionCopies, closedWatchers }),
  };
}

test("unchanged closed SFCs do no source reads or projection work across program updates", () => {
  const f = fixture();
  for (let i = 0; i < 40; i++)
    f.files.set(`/closed-${i}.vue`, { disk: `<template>${i}</template>`, version: 0 });
  f.files.set("/open.vue", { disk: "disk", buffer: "draft-0", version: 0 });
  try {
    for (const name of f.files.keys()) f.host.getScriptVersion(name);
    const before = f.counts();
    for (let edit = 1; edit <= 10; edit++) {
      f.files.get("/open.vue")!.buffer = `draft-${edit}`;
      f.files.get("/open.vue")!.version++;
      f.update();
      f.host.getProjectVersion!();
      for (const name of f.files.keys()) {
        f.host.getScriptVersion(name);
        f.host.getScriptSnapshot(name);
      }
    }
    assert.deepEqual([...f.reads.values()], Array(40).fill(1));
    assert.equal(f.counts().generationCount - before.generationCount, 0);
    assert.equal(f.support.get("/open.vue").source, "draft-10");
    assert.equal(f.counts().generationCount - before.generationCount, 1);
    assert.equal(f.counts().optionCopies - before.optionCopies, 10);
  } finally {
    f.support.dispose();
  }
});

test("closed disk events and unsaved open buffers invalidate their own projection", () => {
  const f = fixture();
  f.files.set("/component.vue", { disk: "first", version: 0 });
  try {
    assert.equal(f.support.get("/component.vue").source, "first");
    f.files.get("/component.vue")!.disk = "second";
    assert.equal(f.watchers.size, 1);
    const before = f.host.getProjectVersion!();
    f.watchers.get("/component.vue")!();
    assert.notEqual(f.host.getProjectVersion!(), before);
    assert.equal(f.support.get("/component.vue").source, "second");
    f.files.get("/component.vue")!.buffer = "";
    f.files.get("/component.vue")!.version++;
    assert.equal(f.support.get("/component.vue").source, "");
    f.files.get("/component.vue")!.disk = "third";
    f.watchers.get("/component.vue")!();
    assert.equal(f.support.get("/component.vue").source, "");
    delete f.files.get("/component.vue")!.buffer;
    f.files.get("/component.vue")!.version++;
    assert.equal(f.support.get("/component.vue").source, "third");
    f.options(true);
    const prior = f.support.get("/component.vue").version;
    f.options(false);
    assert.notEqual(f.support.get("/component.vue").version, prior);
  } finally {
    f.support.dispose();
  }
  assert.equal(f.counts().closedWatchers, 1);
  assert.equal(f.watchers.size, 0);
  assert.equal(f.host.getScriptSnapshot, f.originalSnapshot);
});

test("a host without file watches reads closed mixed content instead of trusting its fixed version", () => {
  const f = fixture(false);
  f.files.set("/component.vue", { disk: "first", version: 0 });
  try {
    assert.equal(f.support.get("/component.vue").source, "first");
    f.files.get("/component.vue")!.disk = "second";
    assert.equal(f.support.get("/component.vue").source, "second");
  } finally {
    f.support.dispose();
  }
});

test("hosts without a project version expose changed compiler options", () => {
  const f = fixture(true, false);
  f.files.set("/component.vue", { disk: "source", version: 0 });
  try {
    const before = f.support.get("/component.vue").version;
    f.options(true);
    assert.notEqual(f.support.get("/component.vue").version, before);
  } finally {
    f.support.dispose();
  }
});
