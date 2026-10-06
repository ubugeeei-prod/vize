// #8099: authored JS workspaces use existing pinned, physically installed cohorts.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const lexical = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
export const save = (directory, name, value) =>
  fs.writeFileSync(path.join(directory, name), JSON.stringify(value, null, 2) + "\n");
export function files(directory) {
  const result = {};
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
export function inputCorpus(root) {
  const directory = path.join(
    root,
    "tests/_fixtures/differential/compat/javascript-workspace-products",
  );
  const corpus = JSON.parse(fs.readFileSync(path.join(directory, "corpus.json"), "utf8"));
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
function linkModules(installed, target) {
  fs.mkdirSync(target, { recursive: true });
  for (const name of fs.readdirSync(installed)) {
    if (name === ".bin") continue;
    if (name.startsWith("@")) {
      fs.mkdirSync(path.join(target, name), { recursive: true });
      for (const child of fs.readdirSync(path.join(installed, name)))
        fs.symlinkSync(path.join(installed, name, child), path.join(target, name, child));
    } else fs.symlinkSync(path.join(installed, name), path.join(target, name));
  }
}
export function prepareProject(root, output, cohort, custody) {
  const { corpus, directory } = inputCorpus(root);
  const installed = path.join(root, cohort.path, "node_modules");
  for (const [name, version] of [
    ["vue", cohort.vue],
    [cohort.nuxt ? "nuxt" : "vite", cohort.nuxt ?? cohort.vite],
  ])
    assert.equal(
      JSON.parse(fs.readFileSync(path.join(installed, name, "package.json"))).version,
      version,
    );
  const artifacts = path.join(output, cohort.id);
  fs.mkdirSync(artifacts);
  const project = path.join(artifacts, "project");
  fs.mkdirSync(project);
  linkModules(installed, path.join(project, "node_modules"));
  const sources = {
    "package.json.txt": "package.json",
    "pnpm-workspace.yaml.txt": "pnpm-workspace.yaml",
    "root-tsconfig.json.txt": "tsconfig.json",
    "vite-package.json.txt": "apps/vite/package.json",
    "vite-tsconfig.json.txt": "apps/vite/tsconfig.json",
    "vite.config.mjs.txt": "apps/vite/vite.config.mjs",
    "index.html.txt": "apps/vite/index.html",
    "vite-main.js.txt": "apps/vite/src/main.js",
    "vite-ssr.mjs.txt": "apps/vite/src/ssr.mjs",
    "ViteApp.vue.txt": "apps/vite/src/App.vue",
    "nuxt-package.json.txt": "apps/nuxt/package.json",
    [cohort.id === "nuxt4" ? "nuxt-reference-tsconfig.json.txt" : "nuxt-tsconfig.json.txt"]:
      "apps/nuxt/tsconfig.json",
    "nuxt.config.js.txt": "apps/nuxt/nuxt.config.js",
    "NuxtApp.vue.txt": "apps/nuxt/src/app.vue",
    "NuxtIndex.vue.txt": "apps/nuxt/src/pages/index.vue",
    "useWorkspaceLabel.js.txt": "apps/nuxt/src/composables/useWorkspaceLabel.js",
    "pricing-package.json.txt": "packages/pricing/package.json",
    "pricing-index.js.txt": "packages/pricing/src/index.js",
    "pricing-label.mjs.txt": "packages/pricing/src/label.mjs",
    "ui-package.json.txt": "packages/ui/package.json",
    "BadgeCard.vue.txt": "packages/ui/src/BadgeCard.vue",
  };
  for (const [source, target] of Object.entries(sources)) {
    fs.mkdirSync(path.dirname(path.join(project, target)), { recursive: true });
    fs.copyFileSync(path.join(directory, source), path.join(project, target));
  }
  const originalDist = {},
    candidateDist = {};
  const packages = [["@vizejs/vite-plugin", "npm/builder/vite"]];
  if (cohort.nuxt) packages.push(["@vizejs/nuxt", "npm/framework/nuxt"]);
  for (const [name, source] of packages) {
    originalDist[name] = files(path.join(installed, name, "dist"));
    candidateDist[name] = files(path.join(root, source, "dist"));
    const target = path.join(project, "node_modules", name);
    fs.unlinkSync(target);
    fs.mkdirSync(target);
    fs.copyFileSync(path.join(installed, name, "package.json"), path.join(target, "package.json"));
    fs.cpSync(path.join(root, source, "dist"), path.join(target, "dist"), { recursive: true });
    assert.deepEqual(files(path.join(target, "dist")), candidateDist[name]);
  }
  const links = JSON.parse(fs.readFileSync(path.join(directory, "links.json")));
  const witnessedLinks = [];
  for (const [name, target] of Object.entries(links)) {
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
  const binding = {
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
