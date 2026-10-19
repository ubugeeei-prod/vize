// #8099: reuse the original Nuxt build cell and its genuine current native receipt.
import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { inputCorpus, prepareProject, save, sha } from "./javascript-workspace-project.ts";
import type { Cohort, WorkspaceContext } from "./javascript-workspace-project.ts";
import { readSourceBindingCustody } from "./source-binding.ts";
const root = fileURLToPath(new URL("../../../../", import.meta.url));
const [destination, custodyPath] = process.argv.slice(2);
assert.ok(destination && custodyPath);
const output = path.resolve(destination);
assert.equal(fs.existsSync(output), false);
fs.mkdirSync(output, { recursive: true });
const custody = readSourceBindingCustody(custodyPath);
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

// Each cohort owns its staged project and artifacts, so their probes overlap.
interface ProbeResult {
  status: number | null;
  signal: string | null;
  error: Error | undefined;
}
function probe(context: WorkspaceContext): Promise<ProbeResult> {
  const fd = fs.openSync(path.join(context.artifacts, "process.log"), "w");
  const serialized = JSON.stringify({ ...context, environment: undefined, finish: undefined });
  return new Promise<ProbeResult>((resolve) => {
    const child = spawn(
      process.execPath,
      [
        fileURLToPath(new URL("./javascript-workspace-probe.ts", import.meta.url)),
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
    child.once("error", (error) => resolve({ status: null, signal: null, error }));
    child.once("close", (status, signal) => resolve({ status, signal, error: undefined }));
  }).finally(() => fs.closeSync(fd));
}

async function run(cohort: Cohort): Promise<Record<string, string>[]> {
  const failures: Record<string, string>[] = [];
  let context: WorkspaceContext | undefined;
  try {
    context = prepareProject(root, output, cohort, custody);
    const result = await probe(context);
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
      context?.finish?.();
    } catch (error) {
      failures.push({
        cohort: cohort.id,
        cleanup: error instanceof Error ? (error.stack ?? error.message) : String(error),
      });
    }
  }
  return failures;
}

const { corpus } = inputCorpus(root);
const failures = (await Promise.all(corpus.cohorts.map(run))).flat();
save(output, "result.json", { source: custody.source, corpus, failures });
assert.equal(sha(fs.readFileSync(custody.binary.path)), custody.binary.sha256);
assert.deepEqual(failures, [], "every whole cohort must pass; no failing cohort is skipped");
