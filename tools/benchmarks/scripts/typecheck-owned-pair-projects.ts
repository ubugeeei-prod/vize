/** Prepare one physical workload per cohort; every source role checks those same bytes. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  cpSync,
  lstatSync,
  mkdirSync,
  readdirSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { CORPORA } from "./typecheck-corpus-pair.ts";
import { generateScaleProject, PLANT_CODES, type ScaleProject } from "./scale-project-generator.ts";
import { prepareMapperPackages, type MapperHosts } from "./typecheck-mapper-hosts.ts";
import { linkPackage } from "./install-package-files.ts";
import { snapshotInputs, writeJson } from "./typecheck-owned-pair-evidence.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
export type Workload = {
  id: string;
  directory: string;
  sfcs: number;
  generated: ScaleProject | null;
};
export const OWNED_WORKLOADS = [
  { id: "misskey", sfcs: 583 },
  { id: "n8n", sfcs: 1113 },
  { id: "vue-20000", sfcs: 20000 },
  { id: "nuxt-2500", sfcs: 2500 },
  { id: "nuxt-10000", sfcs: 10000 },
] as const;
export const FOCUSED_OWNED_WORKLOADS = [
  { id: "giant-2500", sfcs: 1, kind: "giant", giantBlocks: 2500 },
  {
    id: "flat-interpolations-2500",
    sfcs: 1,
    kind: "giant",
    flatInterpolations: 2500,
  },
  {
    id: "flat-interpolations-10000",
    sfcs: 1,
    kind: "giant",
    flatInterpolations: 10000,
  },
  {
    id: "flat-interpolations-50000",
    sfcs: 1,
    kind: "giant",
    flatInterpolations: 50000,
  },
] as const;
const OWNED_WORKLOAD_CATALOG = [...OWNED_WORKLOADS, ...FOCUSED_OWNED_WORKLOADS];

export function generateFlatInterpolationProject(expressions: number): ScaleProject {
  assert.ok(Number.isInteger(expressions) && expressions > 0, "positive expression count");
  const project = generateScaleProject({ sfcs: 1, kind: "giant", giantBlocks: 1 });
  const marker = 'const planted0: number = "flat-interpolation";';
  const template = Array.from(
    { length: expressions },
    (_, index) => `    {{ value + ${index} }}`,
  ).join("\n");
  const source = `<template>\n  <main>\n${template}\n  </main>\n</template>\n\n<script setup lang="ts">\n${marker}\nconst value = 1\n</script>\n`;
  project.files.set("src/App.vue", source);
  project.features = expressions;
  project.plants = [
    {
      kind: "script",
      file: "src/App.vue",
      line: source.slice(0, source.indexOf(marker)).split("\n").length,
      codes: PLANT_CODES.script,
    },
  ];
  return project;
}

export function selectOwnedWorkloads(requested?: string) {
  if (!requested) return [...OWNED_WORKLOADS];
  const ids = requested.split(",");
  assert.ok(
    ids.every((id) => id.length > 0 && id.trim() === id),
    "invalid workload list",
  );
  assert.equal(new Set(ids).size, ids.length, "duplicate workload id");
  return ids.map((id) => {
    const specification = OWNED_WORKLOAD_CATALOG.find((entry) => entry.id === id);
    assert.ok(specification, `unknown workload: ${id}`);
    return specification;
  });
}

export function installRole(workload: Workload, hosts: MapperHosts) {
  prepareMapperPackages(workload.directory, hosts);
  if (workload.generated?.kind === "nuxt") {
    // Batch and mapper deliberately use the exact same app program, not the solution root.
    writeJson(join(workload.directory, "tsconfig.json"), {
      extends: "./.nuxt/tsconfig.app.json",
      compilerOptions: { incremental: false },
    });
  }
}

export function prepareOwnedWorkload(
  id: string,
  work: string,
  out: string,
  hosts: MapperHosts,
): Workload {
  const specification = OWNED_WORKLOAD_CATALOG.find((entry) => entry.id === id);
  assert.ok(specification);
  const directory = join(work, id);
  assert.ok(!lstatSync(directory, { throwIfNoEntry: false }), "workload directory must be new");
  mkdirSync(directory, { recursive: true });
  const real = CORPORA.find((entry) => entry.id === id);
  let generated: ScaleProject | null = null;
  const kind =
    "kind" in specification ? specification.kind : id.startsWith("nuxt-") ? "nuxt" : "vue";
  if (real) {
    for (const entry of real.copy)
      cpSync(join(root, real.source, entry), join(directory, entry), { recursive: true });
    writeJson(join(directory, "tsconfig.json"), real.tsconfig);
  } else {
    generated =
      "flatInterpolations" in specification
        ? generateFlatInterpolationProject(specification.flatInterpolations)
        : generateScaleProject({
            kind,
            sfcs: specification.sfcs,
            seed: 0,
            plants: true,
            ...(kind === "giant" && "giantBlocks" in specification
              ? { giantBlocks: specification.giantBlocks }
              : {}),
          });
    assert.equal(generated.sfcs, specification.sfcs);
    for (const [file, contents] of generated.files) {
      mkdirSync(dirname(join(directory, file)), { recursive: true });
      writeFileSync(join(directory, file), contents);
    }
  }
  const packages = join(
    root,
    "tools/benchmarks",
    kind === "nuxt" && !real ? "nuxt-app" : "typecheck-corpora",
    "node_modules",
  );
  const modules = join(directory, "node_modules");
  mkdirSync(modules);
  for (const name of readdirSync(packages).sort()) {
    if (name.startsWith(".")) continue;
    if (name.startsWith("@")) {
      for (const child of readdirSync(join(packages, name)).sort())
        linkPackage(realpathSync(join(packages, name, child)), join(modules, name, child));
    } else linkPackage(realpathSync(join(packages, name)), join(modules, name));
  }
  if (kind === "nuxt" && generated) {
    const command = [join(packages, "nuxt/bin/nuxt.mjs"), "prepare", directory];
    const prepared = spawnSync(process.execPath, command, {
      cwd: directory,
      env: { ...process.env, CI: "1", NUXT_TELEMETRY_DISABLED: "1" },
      maxBuffer: 64 << 20,
    });
    mkdirSync(join(out, "prepare", id), { recursive: true });
    writeFileSync(join(out, "prepare", id, "stdout.raw"), prepared.stdout ?? Buffer.alloc(0));
    writeFileSync(join(out, "prepare", id, "stderr.raw"), prepared.stderr ?? Buffer.alloc(0));
    writeJson(join(out, "prepare", id, "run.json"), {
      command,
      status: prepared.status,
      error: prepared.error?.message ?? null,
    });
    assert.equal(prepared.status, 0, "Nuxt preparation failed; retained complete raw output");
  }
  const workload = { id, directory, sfcs: specification.sfcs, generated };
  installRole(workload, hosts);
  clearProjectCaches(directory);
  writeJson(join(out, `${id}-inputs.json`), snapshotInputs(directory));
  if (generated) writeJson(join(out, `${id}-plants.json`), generated.plants);
  return workload;
}

/** Delete only declared caches in this owned workload, never a linked dependency store. */
export function clearProjectCaches(directory: string) {
  const removed: string[] = [];
  const remove = (file: string) => {
    const absolute = join(directory, file),
      entry = lstatSync(absolute, { throwIfNoEntry: false });
    if (!entry) return;
    const owner = realpathSync(directory),
      parent = realpathSync(dirname(absolute));
    assert.ok(
      parent === owner || parent.startsWith(`${owner}/`),
      `cache owner escapes workload: ${file}`,
    );
    assert.ok(!entry.isSymbolicLink(), `refuse linked cache: ${file}`);
    rmSync(absolute, { recursive: true, force: true });
    removed.push(file);
  };
  for (const file of [
    "node_modules/.cache",
    "node_modules/.vize",
    "node_modules/.vite",
    ".nuxt/cache",
    ".nuxt/.cache",
  ])
    remove(file);
  const walk = (prefix: string) => {
    for (const entry of readdirSync(join(directory, prefix), { withFileTypes: true })) {
      if (!prefix && entry.name === "node_modules") continue;
      const file = prefix ? `${prefix}/${entry.name}` : entry.name;
      if (entry.name.endsWith(".tsbuildinfo")) remove(file);
      else if (entry.isDirectory()) walk(file);
    }
  };
  walk("");
  return removed;
}
