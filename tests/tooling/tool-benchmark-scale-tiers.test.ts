import assert from "node:assert/strict";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { createTypecheckToolVariants } from "../../tools/benchmarks/scripts/compare-tools-typecheck.mjs";
import {
  ENGINE_CLASSES_BY_SURFACE as CLASSES_BY_SURFACE,
  createSurface,
} from "../../tools/benchmarks/scripts/compare-tools-report.mjs";
import { measureVariants } from "../../tools/benchmarks/scripts/compare-tools-measure.mjs";
import {
  measureScaleNuxt,
  nuxtTiers,
  smallTierSfcs,
  typecheckTiers,
  writeScaleCheckProject,
} from "../../tools/benchmarks/scripts/compare-tools-scale.ts";
import { SURFACE_ORDER } from "../../tools/benchmarks/scripts/published-snapshot-locales.mjs";
import {
  generateScaleProject,
  minSfcs,
} from "../../tools/benchmarks/scripts/scale-project-generator.ts";

const ENGINE_CLASSES_BY_SURFACE: Record<string, Record<string, string>> = CLASSES_BY_SURFACE;

void test("each headline tier has a small tier a quarter of its size, never below the generator minimum", () => {
  assert.equal(smallTierSfcs(10_000, "vue"), 2_500);
  assert.equal(smallTierSfcs(10_000, "nuxt"), 2_500);
  assert.equal(smallTierSfcs(40, "vue"), minSfcs("vue"));
  assert.equal(smallTierSfcs(40, "nuxt"), minSfcs("nuxt"));
  const ids = [...typecheckTiers(10_000, 2_500), ...nuxtTiers(10_000, 2_500)].map(
    (tier) => tier.id,
  );
  assert.deepEqual(ids, ["check", "check-small", "nuxt", "nuxt-small"]);
  for (const id of ids) assert.ok(SURFACE_ORDER.includes(id), id);
  for (const id of ["check", "check-small"]) assert.ok(ENGINE_CLASSES_BY_SURFACE[id], id);
});

void test("the type-check tier is the generator's Vue project with strict templates and no generator plants", (t) => {
  const root = mkdtempSync(join(tmpdir(), "vize-scale-check-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const [tier] = typecheckTiers(120, 40);
  const { dir, bytes } = writeScaleCheckProject(root, tier!);
  assert.equal(dir, join(root, "check"));
  const tsconfig = JSON.parse(readFileSync(join(dir, "tsconfig.json"), "utf8"));
  assert.deepEqual(tsconfig.vueCompilerOptions, { strictTemplates: true });
  const vue: string[] = [];
  const walk = (at: string) => {
    for (const entry of readdirSync(at, { withFileTypes: true })) {
      if (entry.isDirectory()) walk(join(at, entry.name));
      else if (entry.name.endsWith(".vue")) vue.push(join(at, entry.name));
    }
  };
  walk(dir);
  assert.equal(vue.length, 120);
  assert.equal(
    bytes,
    vue.reduce((sum, file) => sum + readFileSync(file).byteLength, 0),
  );
  // The gate plants its own copy; the timed project carries no generator plants.
  const clean = generateScaleProject({ sfcs: 120, kind: "vue", plants: false });
  const planted = generateScaleProject({ sfcs: 120, kind: "vue" });
  assert.equal(clean.plants.length, 0);
  assert.ok(planted.plants.length > 0);
  for (const [path, text] of clean.files) assert.equal(readFileSync(join(dir, path), "utf8"), text);
});

void test("vue-tsc gets a heap that holds the headline tier", () => {
  const calls: { binary: string; env?: Record<string, string> }[] = [];
  const [vueTsc] = createTypecheckToolVariants({
    fileCount: 1,
    vizeBin: "vize",
    corsaPath: "tsgo",
    resolveWorkspaceBin: (name: string) => name,
    cpus: 4,
    runCommand: (binary: string, _args: string[], options: { env?: Record<string, string> }) => {
      calls.push({ binary, env: options.env });
      return {};
    },
  });
  vueTsc.run("/project");
  assert.deepEqual(calls, [
    { binary: "vue-tsc", env: { NODE_OPTIONS: "--max-old-space-size=65536" } },
  ]);
});

void test("every vize check run starts from an empty cache, so no run replays incremental results", () => {
  const caches: { dir: string; empty: boolean }[] = [];
  const lanes = createTypecheckToolVariants({
    fileCount: 1,
    vizeBin: "vize",
    corsaPath: "tsgo",
    resolveWorkspaceBin: (name: string) => name,
    cpus: 4,
    runCommand: (_binary: string, _args: string[], options: { env?: Record<string, string> }) => {
      const dir = options.env?.XDG_CACHE_HOME;
      if (dir) {
        caches.push({ dir, empty: readdirSync(dir).length === 0 });
        // What a run leaves behind must not reach the next one.
        writeFileSync(join(dir, "tsconfig.tsbuildinfo"), "{}");
      }
      return {};
    },
  }).filter((lane: { tool: string }) => lane.tool === "vize check");
  assert.ok(lanes.length >= 2);
  for (const lane of lanes) {
    lane.run("/project");
    lane.run("/project");
  }
  assert.equal(caches.length, lanes.length * 2);
  assert.equal(new Set(caches.map((cache) => cache.dir)).size, caches.length);
  assert.ok(caches.every((cache) => cache.empty));
  assert.ok(caches.every((cache) => !existsSync(cache.dir)));
});

function fakeInstall(root: string) {
  const modules = join(root, "nuxt-app", "node_modules");
  mkdirSync(join(modules, ".vite"), { recursive: true });
  return { modules, nuxtBin: "nuxt", dependencies: { nuxt: "4.0.0" } };
}

void test("every Nuxt build is a fresh copy with cold caches, gated on the server entry and one chunk per route", async (t) => {
  const root = mkdtempSync(join(tmpdir(), "vize-scale-nuxt-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const install = fakeInstall(root);
  const builds: { cwd: string; vize: boolean; cold: boolean }[] = [];
  const [tier] = nuxtTiers(minSfcs("nuxt"), minSfcs("nuxt"));
  const surface = (await measureScaleNuxt({
    tier: tier!,
    workRoot: join(root, "work"),
    install,
    runCommand: (_binary, args, { cwd }) => {
      assert.deepEqual(args, ["build"]);
      builds.push({
        cwd,
        vize: readFileSync(join(cwd, "nuxt.config.ts"), "utf8").includes("@vizejs/nuxt"),
        cold: !existsSync(join(cwd, "node_modules", ".vite")),
      });
      assert.equal(
        JSON.parse(readFileSync(join(cwd, "package.json"), "utf8")).dependencies.nuxt,
        "4.0.0",
      );
      const routes = readdirSync(join(cwd, "app", "pages", "features")).length;
      mkdirSync(join(cwd, ".output", "server"), { recursive: true });
      writeFileSync(join(cwd, ".output", "server", "index.mjs"), "");
      mkdirSync(join(cwd, ".output", "public", "_nuxt"), { recursive: true });
      for (let route = 0; route < routes; route++) {
        writeFileSync(join(cwd, ".output", "public", "_nuxt", `r${route}.js`), "");
      }
      // A later build must not see this one's caches.
      mkdirSync(join(cwd, "node_modules", ".vite"), { recursive: true });
      return { ms: 10, cpuMs: 20 };
    },
    measureVariants,
    createSurface,
    options: { warmups: 1, runs: 3 },
  })) as { id: string; files: number; variants: { id: string; runs: number[] }[] };

  assert.equal(surface.id, "nuxt");
  assert.equal(surface.files, minSfcs("nuxt"));
  assert.deepEqual(
    surface.variants.map((variant) => [variant.id, variant.runs.length]),
    [
      ["nuxt-default", 3],
      ["vize-nuxt", 3],
    ],
  );
  assert.equal(builds.length, 8);
  assert.equal(new Set(builds.map((build) => build.cwd)).size, 8);
  assert.ok(builds.every((build) => build.cold));
  assert.equal(builds.filter((build) => build.vize).length, 4);
  // Copies and the template are removed once measured.
  assert.deepEqual(readdirSync(join(root, "work", "nuxt")), []);
});

void test("a Nuxt build that drops a route chunk or the server entry is refused", async (t) => {
  const root = mkdtempSync(join(tmpdir(), "vize-scale-nuxt-gate-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  // Several routes, so one chunk is too few.
  const [tier] = nuxtTiers(120, 40);
  for (const [server, chunks] of [
    [false, 1_000],
    [true, 1],
  ] as const) {
    await assert.rejects(
      measureScaleNuxt({
        tier: tier!,
        workRoot: join(root, "work"),
        install: fakeInstall(root),
        runCommand: (_binary, _args, { cwd }) => {
          if (server) {
            mkdirSync(join(cwd, ".output", "server"), { recursive: true });
            writeFileSync(join(cwd, ".output", "server", "index.mjs"), "");
          }
          mkdirSync(join(cwd, ".output", "public", "_nuxt"), { recursive: true });
          for (let chunk = 0; chunk < chunks; chunk++) {
            writeFileSync(join(cwd, ".output", "public", "_nuxt", `c${chunk}.js`), "");
          }
          return { ms: 10, cpuMs: 20 };
        },
        measureVariants,
        createSurface,
        options: { warmups: 1, runs: 3 },
      }),
      server ? /emitted 1 client scripts for \d+ routes$/u : /and no server entry/u,
    );
  }
});
