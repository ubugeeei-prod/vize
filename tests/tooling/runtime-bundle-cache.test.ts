import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  renameSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { immutableRuntimeBundle, sha256 } from "./support/runtime-bundle-cache.ts";
import { fingerprintFiles, packageInputFiles } from "./support/runtime-bundle-inputs.ts";

function fixture() {
  const directory = mkdtempSync(join(tmpdir(), "vize-runtime-bundle-"));
  return { directory, close: () => rmSync(directory, { recursive: true, force: true }) };
}
const built = {
  code: "export let count = 0; export const next = () => ++count;",
  modules: ["entry.js"],
};

void test("fresh subprocesses reuse bytes and each import owns fresh module state", () => {
  const f = fixture();
  try {
    const provider = pathToFileURL(
      join(import.meta.dirname, "support/runtime-bundle-cache.ts"),
    ).href;
    const script = `import { immutableRuntimeBundle } from ${JSON.stringify(provider)};
      const bundle = await immutableRuntimeBundle({directory:process.argv[1], inputs:()=>({pin:'rc.9'}),
        build:async()=>(${JSON.stringify(built)})});
      const runtime = await import('data:text/javascript;base64,' + Buffer.from(bundle.code).toString('base64'));
      console.log(JSON.stringify({cache:bundle.cache,count:runtime.next(),pid:process.pid}));`;
    const run = () => {
      const child = spawnSync(
        process.execPath,
        ["--input-type=module", "-e", script, f.directory],
        { encoding: "utf8" },
      );
      assert.equal(child.status, 0, child.stderr);
      return JSON.parse(child.stdout);
    };
    const cold = run(),
      warm = run();
    assert.deepEqual([cold.cache, warm.cache], ["built", "hit"]);
    assert.deepEqual([cold.count, warm.count], [1, 1]);
    assert.notEqual(cold.pid, warm.pid);
  } finally {
    f.close();
  }
});

void test("input content, configuration, environment and missing files invalidate reuse", async () => {
  const f = fixture();
  try {
    const source = join(f.directory, "source.js"),
      optional = join(f.directory, ".env.production");
    writeFileSync(source, "one");
    let production = false,
      nodeEnv = "development",
      calls = 0;
    const load = () =>
      immutableRuntimeBundle({
        directory: join(f.directory, "cache"),
        inputs: () => ({ production, nodeEnv, files: fingerprintFiles([source, optional]) }),
        build: async () => {
          calls++;
          return built;
        },
      });
    assert.equal((await load()).cache, "built");
    assert.equal((await load()).cache, "hit");
    writeFileSync(source, "two");
    await load();
    production = true;
    await load();
    nodeEnv = "production";
    await load();
    writeFileSync(optional, "VITE_VALUE=present");
    await load();
    assert.equal(calls, 5);
  } finally {
    f.close();
  }
});

void test("partial, altered, incomplete and wrong-input records rebuild before code is returned", async () => {
  for (const corrupt of [
    () => "{",
    (entry: Record<string, unknown>) =>
      JSON.stringify({ ...entry, code: "throw new Error('tampered')" }),
    (entry: Record<string, unknown>) =>
      JSON.stringify({ ...entry, identity: { pin: "different" } }),
    (entry: Record<string, unknown>) => JSON.stringify({ ...entry, modules: [] }),
    (entry: Record<string, unknown>) => JSON.stringify({ ...entry, modules: ["different.js"] }),
  ]) {
    const f = fixture();
    try {
      const load = () =>
        immutableRuntimeBundle({
          directory: f.directory,
          inputs: () => ({ pin: "rc.9" }),
          build: async () => built,
        });
      await load();
      const path = join(f.directory, readdirSync(f.directory)[0]);
      writeFileSync(path, corrupt(JSON.parse(readFileSync(path, "utf8"))));
      const repaired = await load();
      assert.equal(repaired.cache, "built");
      assert.equal(repaired.code, built.code);
      assert.equal((await load()).cache, "hit");
    } finally {
      f.close();
    }
  }
});

void test("unwritable cache paths bypass storage and retain actual fresh-build failures", async () => {
  const f = fixture();
  try {
    const directory = join(f.directory, "not-a-directory");
    writeFileSync(directory, "read-only storage boundary");
    const result = await immutableRuntimeBundle({
      directory,
      inputs: () => ({ pin: "rc.9" }),
      build: async () => built,
    });
    assert.equal(result.cache, "bypassed");
    assert.equal(result.code, built.code);
    await assert.rejects(
      immutableRuntimeBundle({
        directory,
        inputs: () => ({ pin: "rc.9" }),
        build: async () => {
          throw new Error("real build failure");
        },
      }),
      /real build failure/,
    );
    assert.equal(readFileSync(directory, "utf8"), "read-only storage boundary");
  } finally {
    f.close();
  }
});

void test("unexpected key-path directories and symlinks remain intact while fresh builds bypass storage", async () => {
  const f = fixture();
  try {
    const identity = { pin: "rc.9" },
      path = join(f.directory, `${sha256(JSON.stringify(identity))}.json`);
    mkdirSync(path);
    writeFileSync(join(path, "sentinel"), "preserve directory contents");
    const load = () =>
      immutableRuntimeBundle({
        directory: f.directory,
        inputs: () => identity,
        build: async () => built,
      });
    assert.equal((await load()).cache, "bypassed");
    assert.equal(readFileSync(join(path, "sentinel"), "utf8"), "preserve directory contents");
    rmSync(path, { recursive: true });
    const sentinel = join(f.directory, "sentinel.json");
    assert.equal((await load()).cache, "built");
    renameSync(path, sentinel);
    const originalBytes = readFileSync(sentinel, "utf8");
    symlinkSync(sentinel, path);
    assert.equal((await load()).cache, "bypassed");
    assert.equal(realpathSync(path), realpathSync(sentinel));
    assert.equal(readFileSync(sentinel, "utf8"), originalBytes);
  } finally {
    f.close();
  }
});

void test("failed builds and inputs changed during a build publish no entry", async () => {
  const f = fixture();
  try {
    let pin = "before";
    await assert.rejects(
      immutableRuntimeBundle({
        directory: f.directory,
        inputs: () => ({ pin }),
        build: async () => {
          throw new Error("actual bundler failure");
        },
      }),
      /actual bundler failure/,
    );
    await assert.rejects(
      immutableRuntimeBundle({
        directory: f.directory,
        inputs: () => ({ pin }),
        build: async () => {
          pin = "after";
          return built;
        },
      }),
      /inputs changed during build/,
    );
    assert.deepEqual(readdirSync(f.directory), []);
  } finally {
    f.close();
  }
});

void test("concurrent cold writers publish one complete immutable record and reject conflicts", async () => {
  const f = fixture();
  try {
    let release!: () => void;
    const ready = new Promise<void>((resolve) => {
      release = resolve;
    });
    const load = (code: string) =>
      immutableRuntimeBundle({
        directory: f.directory,
        inputs: () => ({ pin: "rc.9" }),
        build: async () => {
          await ready;
          return { ...built, code };
        },
      });
    const writers = [load(built.code), load(built.code), load("export const different = true;")];
    release();
    const results = await Promise.allSettled(writers);
    assert.deepEqual(
      results.map((result) => result.status),
      ["fulfilled", "fulfilled", "rejected"],
    );
    assert.equal(readdirSync(f.directory).length, 1);
    const cached = await load(built.code);
    assert.equal(cached.cache, "hit");
    assert.equal(cached.code, built.code);
  } finally {
    f.close();
  }
});

void test("unbounded contexts bypass storage without swallowing actual build failures", async () => {
  const f = fixture();
  try {
    for (let n = 0; n < 2; n++)
      assert.equal(
        (
          await immutableRuntimeBundle({
            directory: f.directory,
            inputs: () => null,
            build: async () => built,
          })
        ).cache,
        "bypassed",
      );
    await assert.rejects(
      immutableRuntimeBundle({
        directory: f.directory,
        inputs: () => null,
        build: async () => {
          throw new Error("failure");
        },
      }),
      /failure/,
    );
    assert.deepEqual(readdirSync(f.directory), []);
  } finally {
    f.close();
  }
});

void test("unproven module coverage never reuses or stores otherwise valid bundle bytes", async () => {
  const f = fixture();
  try {
    let covered = true,
      calls = 0;
    const load = () =>
      immutableRuntimeBundle({
        directory: f.directory,
        inputs: () => ({ pin: "rc.9" }),
        canStore: () => covered,
        build: async () => {
          calls++;
          return built;
        },
      });
    assert.equal((await load()).cache, "built");
    covered = false;
    assert.equal((await load()).cache, "bypassed");
    assert.equal((await load()).cache, "bypassed");
    assert.equal(calls, 3);
    covered = true;
    assert.equal((await load()).cache, "hit");
    assert.equal(calls, 3);
  } finally {
    f.close();
  }
});

void test("installed transitive package bytes and optional dependency presence are inputs", () => {
  const f = fixture();
  try {
    const pkg = join(f.directory, "pkg"),
      dep = join(pkg, "node_modules/dep");
    mkdirSync(dep, { recursive: true });
    writeFileSync(
      join(pkg, "package.json"),
      JSON.stringify({
        name: "pkg",
        version: "1",
        main: "entry.js",
        dependencies: { dep: "1" },
        optionalDependencies: { absent: "1" },
      }),
    );
    writeFileSync(join(pkg, "entry.js"), "entry");
    writeFileSync(
      join(dep, "package.json"),
      JSON.stringify({ name: "dep", version: "1", main: "source.js" }),
    );
    writeFileSync(join(dep, "source.js"), "dependency");
    const before = packageInputFiles([join(pkg, "entry.js")]);
    assert.ok(before.files.includes(realpathSync(join(dep, "source.js"))));
    assert.equal(before.missing.length, 1);
    const hashes = fingerprintFiles(before.files);
    writeFileSync(join(dep, "source.js"), "changed dependency");
    assert.notDeepEqual(fingerprintFiles(packageInputFiles([join(pkg, "entry.js")]).files), hashes);
    const optional = join(pkg, "node_modules/absent");
    mkdirSync(optional);
    writeFileSync(
      join(optional, "package.json"),
      JSON.stringify({ name: "absent", version: "1", main: "entry.js" }),
    );
    writeFileSync(join(optional, "entry.js"), "now installed");
    assert.equal(packageInputFiles([join(pkg, "entry.js")]).missing.length, 0);
  } finally {
    f.close();
  }
});
