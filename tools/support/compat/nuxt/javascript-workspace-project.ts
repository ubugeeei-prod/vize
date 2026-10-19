// #8099: authored JS workspaces use existing pinned, physically installed cohorts.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { asArray, asNumber, asRecord, asString, readJson } from "../../json-value.ts";
import { nuxt3TypeProviders } from "./javascript-workspace-providers.ts";
import { parseSourceBindingCustody } from "./source-binding.ts";
import type { SourceBindingCustody } from "./source-binding.ts";

export interface FilePin {
  bytes: number;
  sha256: string;
}
export type FilePins = Record<string, FilePin>;
export interface Cohort {
  id: string;
  path: string;
  vue: string;
  nuxt?: string;
  vite?: string;
  inputs: FilePins;
}
export interface RenderedPair {
  initial: string;
  clicked: string;
}
export interface Corpus {
  schema: string;
  version: number;
  files: FilePins;
  cohorts: Cohort[];
  scope: unknown;
  expected: { Vite: RenderedPair; Nuxt: RenderedPair };
  nodeTypes: { node: string; undici: string };
}
export interface WorkspaceContext {
  project: string;
  artifacts: string;
  cohort: Cohort;
  corpus: Corpus;
  binding: SourceBindingCustody;
  bindingFile: string;
  environment: NodeJS.ProcessEnv;
  finish?: () => void;
}

export const sha = (bytes: string | Uint8Array): string =>
  createHash("sha256").update(bytes).digest("hex");
export const lexical = (a: string, b: string): number => (a < b ? -1 : a > b ? 1 : 0);
export const save = (directory: string, name: string, value: unknown): void =>
  fs.writeFileSync(path.join(directory, name), JSON.stringify(value, null, 2) + "\n");
export function files(directory: string): FilePins {
  const result: FilePins = {};
  for (const name of fs.readdirSync(directory).sort(lexical)) {
    const file = path.join(directory, name);
    const stat = fs.lstatSync(file);
    if (stat.isSymbolicLink()) continue;
    if (stat.isDirectory()) {
      if (name !== "node_modules")
        for (const [child, pin] of Object.entries(files(file))) result[`${name}/${child}`] = pin;
    } else result[name] = { bytes: stat.size, sha256: sha(fs.readFileSync(file)) };
  }
  return result;
}
function filePins(value: unknown, label: string): FilePins {
  const pins: FilePins = {};
  for (const [name, pin] of Object.entries(asRecord(value, label))) {
    const { bytes, sha256 } = asRecord(pin, `${label}.${name}`);
    assert.ok(typeof bytes === "number" && typeof sha256 === "string", `${label}.${name}`);
    pins[name] = { bytes, sha256 };
  }
  return pins;
}
function optionalString(value: unknown, label: string): string | undefined {
  return value === undefined ? undefined : asString(value, label);
}
function renderedPair(value: unknown, label: string): RenderedPair {
  const pair = asRecord(value, label);
  return { initial: asString(pair.initial, label), clicked: asString(pair.clicked, label) };
}
function parseCorpus(value: unknown): Corpus {
  const corpus = asRecord(value, "corpus");
  const expected = asRecord(corpus.expected, "corpus.expected");
  const nodeTypes = asRecord(corpus.nodeTypes, "corpus.nodeTypes");
  return {
    ...corpus,
    schema: asString(corpus.schema, "corpus.schema"),
    version: asNumber(corpus.version, "corpus.version"),
    files: filePins(corpus.files, "corpus.files"),
    cohorts: asArray(corpus.cohorts, "corpus.cohorts").map((entry: unknown, index: number) => {
      const cohort = asRecord(entry, `cohort ${index}`);
      return {
        ...cohort,
        id: asString(cohort.id, "cohort.id"),
        path: asString(cohort.path, "cohort.path"),
        vue: asString(cohort.vue, "cohort.vue"),
        nuxt: optionalString(cohort.nuxt, "cohort.nuxt"),
        vite: optionalString(cohort.vite, "cohort.vite"),
        inputs: filePins(cohort.inputs, "cohort.inputs"),
      };
    }),
    scope: corpus.scope,
    expected: {
      Vite: renderedPair(expected.Vite, "expected.Vite"),
      Nuxt: renderedPair(expected.Nuxt, "expected.Nuxt"),
    },
    nodeTypes: {
      node: asString(nodeTypes.node, "nodeTypes.node"),
      undici: asString(nodeTypes.undici, "nodeTypes.undici"),
    },
  };
}
/** Rebuild a context that a parent process serialized for a probe child. */
export function parseWorkspaceContext(
  value: unknown,
  environment: NodeJS.ProcessEnv,
): WorkspaceContext {
  const context = asRecord(value, "context");
  const corpus = parseCorpus(context.corpus);
  const id = asString(asRecord(context.cohort, "context.cohort").id, "context.cohort.id");
  const cohort = corpus.cohorts.find((entry) => entry.id === id);
  assert.ok(cohort, `cohort ${id} belongs to the corpus`);
  return {
    project: asString(context.project, "context.project"),
    artifacts: asString(context.artifacts, "context.artifacts"),
    cohort,
    corpus,
    binding: parseSourceBindingCustody(context.binding),
    bindingFile: asString(context.bindingFile, "context.bindingFile"),
    environment,
  };
}
export function inputCorpus(root: string): { corpus: Corpus; directory: string } {
  const directory = path.join(
    root,
    "tests/_fixtures/differential/compat/javascript-workspace-products",
  );
  const raw = readJson(path.join(directory, "corpus.json"));
  assert.equal(asRecord(raw, "corpus").version, 1);
  const corpus = parseCorpus(raw);
  assert.equal(corpus.schema, "vize.compat.javascript-workspace-products");
  assert.equal(corpus.version, 1);
  for (const [name, pin] of Object.entries(corpus.files)) {
    const bytes = fs.readFileSync(path.join(directory, name));
    assert.equal(bytes.length, pin.bytes);
    assert.equal(sha(bytes), pin.sha256);
  }
  for (const cohort of corpus.cohorts)
    for (const [name, pin] of Object.entries(cohort.inputs)) {
      const bytes = fs.readFileSync(path.join(root, cohort.path, name));
      assert.equal(bytes.length, pin.bytes);
      assert.equal(sha(bytes), pin.sha256);
    }
  return { corpus, directory };
}
function copyModules(installed: string, target: string): void {
  // Preserve the installed dependency layout inside this owned workspace. External
  // absolute links make Nuxt-generated includes escape its node_modules exclusions.
  fs.cpSync(installed, target, { recursive: true, dereference: false, verbatimSymlinks: true });
}
export function prepareProject(
  root: string,
  output: string,
  cohort: Cohort,
  custody: SourceBindingCustody,
): WorkspaceContext {
  const { corpus, directory } = inputCorpus(root);
  const installed = path.join(root, cohort.path, "node_modules");
  for (const [name, version] of [
    ["vue", cohort.vue],
    [cohort.nuxt ? "nuxt" : "vite", cohort.nuxt ?? cohort.vite],
  ] as const)
    assert.equal(asRecord(readJson(path.join(installed, name, "package.json"))).version, version);
  const artifacts = path.join(output, cohort.id);
  fs.mkdirSync(artifacts);
  const project = path.join(artifacts, "project");
  fs.mkdirSync(project);
  copyModules(installed, path.join(project, "node_modules"));
  const sources: Record<string, string> = {
    "package.json.txt": "package.json",
    "pnpm-workspace.yaml.txt": "pnpm-workspace.yaml",
    "root-tsconfig.json.txt": "tsconfig.json",
    "workspace-declarations-tsconfig.json.txt": "workspace-declarations.tsconfig.json",
    "vite-package.json.txt": "apps/vite/package.json",
    "vite-tsconfig.json.txt": "apps/vite/tsconfig.json",
    "vite.config.mjs.txt": "apps/vite/vite.config.mjs",
    "index.html.txt": "apps/vite/index.html",
    "vite-main.js.txt": "apps/vite/src/main.js",
    "vite-ssr.mjs.txt": "apps/vite/src/ssr.mjs",
    "ViteApp.vue.txt": "apps/vite/src/App.vue",
    [cohort.id === "nuxt3" ? "nuxt3-package-typed.json.txt" : "nuxt-package.json.txt"]:
      "apps/nuxt/package.json",
    [cohort.id === "nuxt4" ? "nuxt-reference-tsconfig.json.txt" : "nuxt-tsconfig.json.txt"]:
      "apps/nuxt/tsconfig.json",
    [cohort.id === "nuxt3" ? "nuxt3-owned-schema.config.js.txt" : "nuxt.config.js.txt"]:
      "apps/nuxt/nuxt.config.js",
    "NuxtApp.vue.txt": "apps/nuxt/src/app.vue",
    "NuxtIndex.vue.txt": "apps/nuxt/src/pages/index.vue",
    "useWorkspaceLabel.js.txt": "apps/nuxt/src/composables/useWorkspaceLabel.js",
    "pricing-package.json.txt": "packages/pricing/package.json",
    "pricing-index.js.txt": "packages/pricing/src/index.js",
    "pricing-label.mjs.txt": "packages/pricing/src/label.mjs",
    "ui-package-typed.json.txt": "packages/ui/package.json",
    "BadgeCard.vue.txt": "packages/ui/src/BadgeCard.vue",
  };
  for (const [source, target] of Object.entries(sources)) {
    fs.mkdirSync(path.dirname(path.join(project, target)), { recursive: true });
    fs.copyFileSync(path.join(directory, source), path.join(project, target));
  }
  const verifyTypes = nuxt3TypeProviders(root, project, artifacts, cohort, corpus);
  const originalDist: Record<string, FilePins> = {},
    candidateDist: Record<string, FilePins> = {};
  const packages: [string, string][] = [["@vizejs/vite-plugin", "npm/builder/vite"]];
  if (cohort.nuxt) packages.push(["@vizejs/nuxt", "npm/framework/nuxt"]);
  for (const [name, source] of packages) {
    originalDist[name] = files(path.join(installed, name, "dist"));
    candidateDist[name] = files(path.join(root, source, "dist"));
    const target = path.join(project, "node_modules", name);
    fs.rmSync(target, { recursive: true, force: true });
    fs.mkdirSync(target);
    fs.copyFileSync(path.join(installed, name, "package.json"), path.join(target, "package.json"));
    fs.cpSync(path.join(root, source, "dist"), path.join(target, "dist"), { recursive: true });
    assert.deepEqual(files(path.join(target, "dist")), candidateDist[name]);
  }
  const links = asRecord(readJson(path.join(directory, "links.json")), "links.json");
  const witnessedLinks: { name: string; target: string; physical: string }[] = [];
  for (const [name, value] of Object.entries(links)) {
    const target = asString(value, `links.${name}`);
    const filename = path.join(project, name);
    fs.mkdirSync(path.dirname(filename), { recursive: true });
    fs.symlinkSync(target, filename);
    const physical = fs.realpathSync(filename);
    assert.equal(physical, fs.realpathSync(path.resolve(path.dirname(filename), target)));
    assert.ok(physical.startsWith(project + path.sep));
    witnessedLinks.push({ name, target, physical });
  }
  // Each real workspace package resolves Vue through the cohort at the root.
  const targets = cohort.nuxt
    ? ["apps/nuxt/src/app.vue", "apps/nuxt/src/pages/index.vue", "packages/ui/src/BadgeCard.vue"]
    : ["apps/vite/src/App.vue", "packages/ui/src/BadgeCard.vue"];
  const binding: SourceBindingCustody = {
    ...custody,
    calls: path.join(artifacts, "native-calls.jsonl"),
    fixtures: targets.map((name) => ({
      filename: path.join(project, name),
      source: fs.readFileSync(path.join(project, name), "utf8"),
    })),
  };
  const bindingFile = path.join(artifacts, "custody.json");
  save(artifacts, "custody.json", binding);
  const before = files(project);
  save(artifacts, "staged-source.json", {
    cohort,
    sources,
    before,
    witnessedLinks,
    originalDist,
    candidateDist,
  });
  fs.cpSync(directory, path.join(artifacts, "authored-inputs"), { recursive: true });
  return {
    project,
    artifacts,
    cohort,
    corpus,
    binding,
    bindingFile,
    environment: {
      ...process.env,
      NO_COLOR: "1",
      NUXT_TELEMETRY_DISABLED: "1",
      VIZE_NUXT_NATIVE_CUSTODY: bindingFile,
      NODE_OPTIONS:
        `${process.env.NODE_OPTIONS ?? ""} --require=${JSON.stringify(path.join(root, "tools/support/compat/nuxt/source-binding-preload.cjs"))}`.trim(),
    },
    finish() {
      verifyTypes?.();
      for (const [name, source] of Object.entries(sources))
        assert.deepEqual(
          fs.readFileSync(path.join(project, source)),
          fs.readFileSync(path.join(directory, name)),
        );
      for (const [name] of packages)
        assert.deepEqual(files(path.join(installed, name, "dist")), originalDist[name]);
      const generated = path.join(project, "apps/nuxt/.nuxt");
      if (fs.existsSync(generated))
        fs.cpSync(generated, path.join(artifacts, "generated-nuxt"), {
          recursive: true,
          dereference: false,
        });
      save(artifacts, "final-source.json", files(project));
      // Only this helper's owned links are removed after every consumer exits.
      for (const name of [
        "node_modules",
        "apps/vite/node_modules",
        "apps/nuxt/node_modules",
        "packages/ui/node_modules",
      ])
        fs.rmSync(path.join(project, name), { recursive: true, force: true });
    },
  };
}
