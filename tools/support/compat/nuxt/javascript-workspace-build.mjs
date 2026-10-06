// #8099: reuse the original Nuxt build cell and its genuine current native receipt.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { inputCorpus, prepareProject, save, sha } from "./javascript-workspace-project.mjs";
const root = fileURLToPath(new URL("../../../../", import.meta.url));
const [destination, custodyPath] = process.argv.slice(2);
assert.ok(destination && custodyPath);
const output = path.resolve(destination);
assert.equal(fs.existsSync(output), false);
fs.mkdirSync(output, { recursive: true });
const custody = JSON.parse(fs.readFileSync(custodyPath));
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
assert.equal(sha(fs.readFileSync(custody.binary.path)), custody.binary.sha256);
for (const name of [
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_NUXT_NATIVE_CUSTODY",
])
  assert.ok(!process.env[name], `unbound ambient ${name}`);
const { corpus } = inputCorpus(root);
const failures = [];
for (const cohort of corpus.cohorts) {
  let context;
  try {
    context = prepareProject(root, output, cohort, custody);
    const fd = fs.openSync(path.join(context.artifacts, "process.log"), "w");
    let result;
    try {
      const serialized = JSON.stringify({ ...context, environment: undefined, finish: undefined });
      result = spawnSync(
        process.execPath,
        [
          fileURLToPath(new URL("./javascript-workspace-probe.mjs", import.meta.url)),
          root,
          serialized,
        ],
        {
          cwd: context.project,
          env: context.environment,
          stdio: ["ignore", fd, fd],
          timeout: 360_000,
        },
      );
    } finally {
      fs.closeSync(fd);
    }
    save(context.artifacts, "process.json", {
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, `all failed raw observations retained for ${cohort.id}`);
  } catch (error) {
    failures.push({
      cohort: cohort.id,
      error: error instanceof Error ? (error.stack ?? error.message) : String(error),
    });
  } finally {
    try {
      context?.finish();
    } catch (error) {
      failures.push({
        cohort: cohort.id,
        cleanup: error instanceof Error ? (error.stack ?? error.message) : String(error),
      });
    }
  }
}
save(output, "result.json", { source: custody.source, corpus, failures });
assert.equal(sha(fs.readFileSync(custody.binary.path)), custody.binary.sha256);
assert.deepEqual(failures, [], "every whole cohort must pass; no failing cohort is skipped");
