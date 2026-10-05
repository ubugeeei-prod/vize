// #7959: real Nuxt setup, unchanged pinned dependencies, source-built compiler.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const output = path.resolve(process.argv[2]);
const custody = JSON.parse(fs.readFileSync(path.resolve(process.argv[3]), "utf8"));
const inputs = path.join(root, "tests/_fixtures/differential/compiler/nuxt-compiler-defaults");
const corpus = JSON.parse(fs.readFileSync(path.join(inputs, "corpus.json"), "utf8"));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
assert.equal(custody.schema, "vize.nuxt.source-binding");
assert.equal(custody.version, 1);
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
for (const [name, pin] of Object.entries(corpus.files)) {
  const bytes = fs.readFileSync(path.join(inputs, name));
  assert.equal(bytes.length, pin.bytes);
  assert.equal(hash(bytes), pin.sha256);
}
assert.equal(fs.existsSync(output), false, "retain one fresh complete packet");
fs.mkdirSync(output, { recursive: true });
fs.cpSync(inputs, path.join(output, "original-inputs"), { recursive: true });
const packages = [
  ["@vizejs/nuxt", "npm/framework/nuxt"],
  ["@vizejs/vite-plugin", "npm/builder/vite"],
];
const hashes = (directory) =>
  Object.fromEntries(
    fs
      .readdirSync(directory, { recursive: true })
      .sort()
      .flatMap((name) => {
        const file = path.join(directory, name);
        return fs.statSync(file).isFile() ? [[name, hash(fs.readFileSync(file))]] : [];
      }),
  );
for (const cohort of corpus.cohorts) {
  const installed = path.join(root, cohort.path, "node_modules");
  const artifacts = path.join(output, cohort.id);
  fs.mkdirSync(artifacts);
  const project = fs.mkdtempSync(path.join(artifacts, "project-"));
  const modules = path.join(project, "node_modules");
  fs.mkdirSync(modules);
  const originalDist = {},
    candidateDist = {};
  try {
    for (const name of fs.readdirSync(installed)) {
      if (name.startsWith("@")) {
        fs.mkdirSync(path.join(modules, name));
        for (const child of fs.readdirSync(path.join(installed, name)))
          fs.symlinkSync(path.join(installed, name, child), path.join(modules, name, child));
      } else fs.symlinkSync(path.join(installed, name), path.join(modules, name));
    }
    for (const [name, source] of packages) {
      originalDist[name] = hashes(path.join(installed, name, "dist"));
      candidateDist[name] = hashes(path.join(root, source, "dist"));
      const target = path.join(modules, name);
      fs.rmSync(target, { recursive: true, force: true });
      fs.mkdirSync(target, { recursive: true });
      fs.copyFileSync(
        path.join(installed, name, "package.json"),
        path.join(target, "package.json"),
      );
      fs.cpSync(path.join(root, source, "dist"), path.join(target, "dist"), { recursive: true });
      assert.deepEqual(hashes(path.join(target, "dist")), candidateDist[name]);
    }
    fs.writeFileSync(
      path.join(project, "package.json"),
      fs.readFileSync(path.join(inputs, "package.json.txt")),
    );
    const mapped = [
      ["nuxt.config.ts.txt", "nuxt.config.ts"],
      ["TightRow.vue.txt", corpus.paths[0]],
      ["LooseRow.vue.txt", corpus.paths[1]],
      ["app.vue.txt", corpus.paths[2]],
    ];
    for (const [input, target] of mapped) {
      fs.mkdirSync(path.dirname(path.join(project, target)), { recursive: true });
      fs.copyFileSync(path.join(inputs, input), path.join(project, target));
    }
    const binding = {
      ...custody,
      calls: path.join(artifacts, "native-calls.jsonl"),
      fixtures: corpus.paths.map((name) => ({
        filename: path.join(project, name),
        source: fs.readFileSync(path.join(project, name), "utf8"),
      })),
    };
    const bindingFile = path.join(artifacts, "custody.json");
    fs.writeFileSync(bindingFile, JSON.stringify(binding, null, 2) + "\n");
    for (const scenario of corpus.scenarios) {
      const log = fs.openSync(path.join(artifacts, scenario.id + ".log"), "w");
      let run;
      try {
        run = spawnSync(
          process.execPath,
          [
            fileURLToPath(new URL("./compiler-defaults-probe.mjs", import.meta.url)),
            project,
            artifacts,
            inputs,
            cohort.nuxt,
            scenario.id,
          ],
          {
            cwd: project,
            stdio: ["ignore", log, log],
            timeout: 90_000,
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
      fs.writeFileSync(
        path.join(artifacts, scenario.id + "-process.json"),
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
      assert.equal(run.error, undefined);
      assert.equal(run.signal, null);
      assert.equal(run.status, 0, `retain complete ${cohort.id}/${scenario.id}.log`);
    }
    for (const [input, target] of mapped)
      assert.deepEqual(
        fs.readFileSync(path.join(project, target)),
        fs.readFileSync(path.join(inputs, input)),
      );
    for (const [name] of packages)
      assert.deepEqual(hashes(path.join(installed, name, "dist")), originalDist[name]);
  } finally {
    fs.rmSync(project, { recursive: true, force: true });
  }
}
assert.equal(hash(fs.readFileSync(custody.binary.path)), custody.binary.sha256);
console.log(
  "Original #7959 Nuxt3/4 setup, whole Vite client/SSR output and exact rendered HTML passed",
);
