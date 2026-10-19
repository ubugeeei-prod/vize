// #7959: real Nuxt setup, unchanged pinned dependencies, source-built compiler.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asRecord,
  asString,
  asStringArray,
  at,
  readJsonRecord,
} from "../../json-value.ts";
import { readSourceBindingCustody } from "./source-binding.ts";
import type { SourceBindingCustody } from "./source-binding.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const [outputArgument, custodyArgument] = process.argv.slice(2);
assert.ok(outputArgument && custodyArgument, "usage: compiler-defaults-build.ts OUTPUT CUSTODY");
const output = path.resolve(outputArgument);
const custody = readSourceBindingCustody(path.resolve(custodyArgument));
const inputs = path.join(root, "tests/_fixtures/differential/compiler/nuxt-compiler-defaults");
const corpus = readJsonRecord(path.join(inputs, "corpus.json"));
const corpusPaths = asStringArray(corpus.paths, "corpus.paths");
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
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
for (const [name, pin] of Object.entries(asRecord(corpus.files, "corpus.files"))) {
  const bytes = fs.readFileSync(path.join(inputs, name));
  assert.equal(bytes.length, at(pin, "bytes"));
  assert.equal(hash(bytes), at(pin, "sha256"));
}
assert.equal(fs.existsSync(output), false, "retain one fresh complete packet");
fs.mkdirSync(output, { recursive: true });
fs.cpSync(inputs, path.join(output, "original-inputs"), { recursive: true });
const packages: readonly (readonly [string, string])[] = [
  ["@vizejs/nuxt", "npm/framework/nuxt"],
  ["@vizejs/vite-plugin", "npm/builder/vite"],
];
const hashes = (directory: string): Record<string, string> =>
  Object.fromEntries(
    fs
      .readdirSync(directory, { recursive: true, encoding: "utf8" })
      .sort((left, right) => (left < right ? -1 : left > right ? 1 : 0))
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
  fs.mkdirSync(modules);
  const originalDist: Record<string, Record<string, string>> = {},
    candidateDist: Record<string, Record<string, string>> = {};
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
      const candidate = hashes(path.join(root, source, "dist"));
      candidateDist[name] = candidate;
      const target = path.join(modules, name);
      fs.rmSync(target, { recursive: true, force: true });
      fs.mkdirSync(target, { recursive: true });
      fs.copyFileSync(
        path.join(installed, name, "package.json"),
        path.join(target, "package.json"),
      );
      fs.cpSync(path.join(root, source, "dist"), path.join(target, "dist"), { recursive: true });
      assert.deepEqual(hashes(path.join(target, "dist")), candidate);
    }
    fs.writeFileSync(
      path.join(project, "package.json"),
      fs.readFileSync(path.join(inputs, "package.json.txt")),
    );
    const [tightRow, looseRow, app] = corpusPaths;
    assert.ok(tightRow && looseRow && app, "corpus names the three authored paths");
    const mapped = [
      ["nuxt.config.ts.txt", "nuxt.config.ts"],
      ["TightRow.vue.txt", tightRow],
      ["LooseRow.vue.txt", looseRow],
      ["app.vue.txt", app],
    ] as const;
    for (const [input, target] of mapped) {
      fs.mkdirSync(path.dirname(path.join(project, target)), { recursive: true });
      fs.copyFileSync(path.join(inputs, input), path.join(project, target));
    }
    const binding: SourceBindingCustody = {
      ...custody,
      calls: path.join(artifacts, "native-calls.jsonl"),
      fixtures: corpusPaths.map((name) => ({
        filename: path.join(project, name),
        source: fs.readFileSync(path.join(project, name), "utf8"),
      })),
    };
    const bindingFile = path.join(artifacts, "custody.json");
    fs.writeFileSync(bindingFile, JSON.stringify(binding, null, 2) + "\n");
    for (const entry of asArray(corpus.scenarios, "corpus.scenarios")) {
      const scenarioId = asString(at(entry, "id"), "scenario.id");
      const log = fs.openSync(path.join(artifacts, scenarioId + ".log"), "w");
      let run: ReturnType<typeof spawnSync>;
      try {
        run = spawnSync(
          process.execPath,
          [
            fileURLToPath(new URL("./compiler-defaults-probe.ts", import.meta.url)),
            project,
            artifacts,
            inputs,
            asString(cohort.nuxt, "cohort.nuxt"),
            scenarioId,
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
        path.join(artifacts, scenarioId + "-process.json"),
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
      assert.equal(run.status, 0, `retain complete ${cohortId}/${scenarioId}.log`);
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
