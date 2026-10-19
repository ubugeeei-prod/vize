// #7936: exact original Vite inputs, isolated installed cohorts, source-built adapter.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { asArray, asRecord, asString, at, readJsonRecord } from "../../json-value.ts";
import { readSourceBindingCustody } from "./source-binding.ts";
import type { SourceBindingCustody } from "./source-binding.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const [outputArgument, custodyArgument] = process.argv.slice(2);
assert.ok(outputArgument && custodyArgument, "usage: relative-glob-build.ts OUTPUT CUSTODY");
const output = path.resolve(outputArgument);
const custody = readSourceBindingCustody(path.resolve(custodyArgument));
const inputs = path.join(root, "tests/_fixtures/differential/compiler/vite-relative-glob");
const corpus = readJsonRecord(path.join(inputs, "corpus.json"));
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
assert.equal(custody.schema, "vize.nuxt.source-binding");
assert.equal(custody.source.head, process.env.GITHUB_SHA);
assert.equal(
  execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
  custody.source.head,
);
assert.equal(
  execFileSync("git", ["rev-parse", "HEAD^{tree}"], { cwd: root, encoding: "utf8" }).trim(),
  custody.source.tree,
);
assert.equal(hash(fs.readFileSync(custody.binary.path)), custody.binary.sha256);
for (const key of ["NAPI_RS_NATIVE_LIBRARY_PATH", "NAPI_RS_FORCE_WASI", "VIZE_NUXT_NATIVE_CUSTODY"])
  assert.ok(!process.env[key], `ambient ${key} cannot qualify source`);
for (const [name, pin] of Object.entries(asRecord(corpus.files, "corpus.files"))) {
  const bytes = fs.readFileSync(path.join(inputs, name));
  assert.equal(bytes.length, at(pin, "bytes"));
  assert.equal(hash(bytes), at(pin, "sha256"));
}
assert.equal(fs.existsSync(output), false, "one fresh complete packet");
fs.mkdirSync(output, { recursive: true });
fs.cpSync(inputs, path.join(output, "original-inputs"), { recursive: true });
const hashes = (directory: string): Record<string, string> =>
  Object.fromEntries(
    fs
      .readdirSync(directory, { recursive: true, encoding: "utf8" })
      .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))
      .flatMap((name) => {
        const file = path.join(directory, name);
        return fs.statSync(file).isFile() ? [[name, hash(fs.readFileSync(file))]] : [];
      }),
  );
for (const value of asArray(corpus.cohorts, "corpus.cohorts")) {
  const cohort = asRecord(value, "cohort");
  const cohortId = asString(cohort.id, "cohort.id");
  const installed = path.join(root, asString(cohort.path, "cohort.path"), "node_modules");
  const artifacts = path.join(output, cohortId);
  fs.mkdirSync(artifacts);
  const project = fs.mkdtempSync(path.join(artifacts, "project-"));
  const modules = path.join(project, "node_modules");
  const originalDist: Record<string, Record<string, string>> = {},
    candidateDist: Record<string, Record<string, string>> = {};
  const mapping = {
    "package.json.txt": "package.json",
    "vite.config.ts.txt": "vite.config.ts",
    "vite.src-root.config.ts.txt": "vite.src-root.config.ts",
    "index.html.txt": "index.html",
    "src-index.html.txt": "src/index.html",
    "main.ts.txt": "src/main.ts",
    "Plain.vue.txt": "src/pages/demo/Plain.vue",
    "Alpha.vue.txt": "fixtures/Alpha.vue",
    "Lookup.vue.txt": "src/pages/demo/Lookup.vue",
    "Options.vue.txt": "src/pages/demo/Options.vue",
    "First.vue.txt": "control-fixtures/First.vue",
    "Skip.vue.txt": "control-fixtures/Skip.vue",
    "nuxt.config.ts.txt": "nuxt.config.ts",
    "ssr-entry.ts.txt": "src/ssr-plain.ts",
  };
  try {
    fs.mkdirSync(modules);
    for (const name of fs.readdirSync(installed)) {
      if (name.startsWith("@")) {
        fs.mkdirSync(path.join(modules, name));
        for (const child of fs.readdirSync(path.join(installed, name)))
          fs.symlinkSync(path.join(installed, name, child), path.join(modules, name, child));
      } else fs.symlinkSync(path.join(installed, name), path.join(modules, name));
    }
    const packages: [string, string][] = [["@vizejs/vite-plugin", "npm/builder/vite"]];
    if (cohort.nuxt) packages.push(["@vizejs/nuxt", "npm/framework/nuxt"]);
    for (const [name, source] of packages) {
      originalDist[name] = hashes(path.join(installed, name, "dist"));
      const candidate = hashes(path.join(root, source, "dist"));
      candidateDist[name] = candidate;
      const target = path.join(modules, name);
      fs.rmSync(target, { recursive: true, force: true });
      fs.mkdirSync(target);
      fs.copyFileSync(
        path.join(installed, name, "package.json"),
        path.join(target, "package.json"),
      );
      fs.cpSync(path.join(root, source, "dist"), path.join(target, "dist"), { recursive: true });
      assert.deepEqual(hashes(path.join(target, "dist")), candidate);
    }
    for (const [input, target] of Object.entries(mapping)) {
      fs.mkdirSync(path.dirname(path.join(project, target)), { recursive: true });
      fs.copyFileSync(path.join(inputs, input), path.join(project, target));
    }
    for (const scenario of ["lookup", "options"]) {
      const component = scenario === "lookup" ? "Lookup" : "Options";
      fs.writeFileSync(
        path.join(project, `src/${scenario}-main.ts`),
        `import { createApp } from "vue";\nimport Component from "./pages/demo/${component}.vue";\ncreateApp(Component).mount("#app");\n`,
      );
      fs.writeFileSync(
        path.join(project, `${scenario}.html`),
        `<!doctype html>\n<div id="app"></div>\n<script type="module" src="/src/${scenario}-main.ts"></script>\n`,
      );
      fs.writeFileSync(
        path.join(project, `src/${scenario}.html`),
        `<!doctype html>\n<div id="app"></div>\n<script type="module" src="./${scenario}-main.ts"></script>\n`,
      );
      fs.writeFileSync(
        path.join(project, `src/ssr-${scenario}.ts`),
        `export { default } from "./pages/demo/${component}.vue";\n`,
      );
    }
    const frozenWork = hashes(project);
    fs.writeFileSync(
      path.join(artifacts, "before-inputs.json"),
      JSON.stringify(frozenWork, null, 2) + "\n",
    );
    const fixturePaths = [
      "src/pages/demo/Plain.vue",
      "fixtures/Alpha.vue",
      "src/pages/demo/Lookup.vue",
      "src/pages/demo/Options.vue",
      "control-fixtures/First.vue",
    ];
    const binding: SourceBindingCustody = {
      ...custody,
      calls: path.join(artifacts, "native-calls.jsonl"),
      fixtures: fixturePaths.map((name) => ({
        filename: path.join(project, name),
        source: fs.readFileSync(path.join(project, name), "utf8"),
      })),
    };
    const bindingFile = path.join(artifacts, "custody.json");
    fs.writeFileSync(bindingFile, JSON.stringify(binding, null, 2) + "\n");
    const log = fs.openSync(path.join(artifacts, "process.log"), "w");
    let run: ReturnType<typeof spawnSync>;
    try {
      run = spawnSync(
        process.execPath,
        [
          fileURLToPath(new URL("./relative-glob-probe.ts", import.meta.url)),
          project,
          artifacts,
          inputs,
          JSON.stringify(cohort),
        ],
        {
          cwd: project,
          stdio: ["ignore", log, log],
          timeout: 180_000,
          env: {
            ...process.env,
            NO_COLOR: "1",
            NUXT_TELEMETRY_DISABLED: "1",
            VIZE_NUXT_NATIVE_CUSTODY: bindingFile,
            NODE_OPTIONS:
              `${process.env.NODE_OPTIONS ?? ""} --require=${JSON.stringify(fileURLToPath(new URL("./source-binding-preload.cjs", import.meta.url)))}`.trim(),
          },
        },
      );
    } finally {
      fs.closeSync(log);
    }
    const afterInputs = Object.fromEntries(
      Object.keys(frozenWork).map((name) => [
        name,
        hash(fs.readFileSync(path.join(project, name))),
      ]),
    );
    fs.writeFileSync(
      path.join(artifacts, "after-inputs.json"),
      JSON.stringify(afterInputs, null, 2) + "\n",
    );
    fs.writeFileSync(
      path.join(artifacts, "after-files.json"),
      JSON.stringify(hashes(project), null, 2) + "\n",
    );
    fs.writeFileSync(
      path.join(artifacts, "process.json"),
      JSON.stringify(
        {
          source: custody.source,
          candidateDist,
          originalDist,
          status: run.status,
          signal: run.signal,
          error: run.error?.message ?? null,
        },
        null,
        2,
      ) + "\n",
    );
    assert.deepEqual(afterInputs, frozenWork);
    for (const [name] of packages)
      assert.deepEqual(hashes(path.join(installed, name, "dist")), originalDist[name]);
    assert.equal(run.error, undefined);
    assert.equal(run.signal, null);
    assert.equal(run.status, 0, `retain complete ${cohortId}/process.log`);
    for (const [input, target] of Object.entries(mapping))
      assert.deepEqual(
        fs.readFileSync(path.join(project, target)),
        fs.readFileSync(path.join(inputs, input)),
      );
  } finally {
    fs.rmSync(project, { recursive: true, force: true });
  }
}
assert.equal(hash(fs.readFileSync(custody.binary.path)), custody.binary.sha256);
console.log(
  "Original #7936 Vite + genuine Nuxt3/4 project/src roots, client/browser and whole SSR controls passed",
);
