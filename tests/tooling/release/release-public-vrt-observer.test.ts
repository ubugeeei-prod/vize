import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import { readRawBlob } from "../../../tools/support/release/public_acceptance/plan.ts";
import {
  createVrtFixture,
  materializeVrtInputs,
  sha256,
  vrtSources,
  writeVrtConfig,
} from "../../../tools/support/release/public_acceptance/vrt_fixtures.ts";
import {
  assertVrtCustody,
  cleanup,
  evidence,
} from "../../../tools/support/release/public_acceptance/vrt_artifacts.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const head = execFileSync("git", ["--no-replace-objects", "-C", root, "rev-parse", "HEAD"], {
  encoding: "utf8",
}).trim();
async function consumer(t: TestContext) {
  const directory = await mkdtemp(path.join(os.tmpdir(), "vize-vrt-guard-law-"));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const inputs = materializeVrtInputs(directory, (file) => readRawBlob(root, head, file));
  return { directory, inputs };
}

// These source/custody laws load no product, compiler, renderer, API or native provider.
test("raw release VRT seeds remain whole and sealed while the runtime workspace changes", async (t) => {
  const { directory, inputs } = await consumer(t);
  assert.deepEqual(
    inputs.map((item) => item.sourceFile),
    vrtSources,
  );
  for (const input of inputs) {
    const bytes = await readFile(path.join(directory, input.file));
    assert.deepEqual(bytes, readRawBlob(root, head, input.sourceFile));
    assert.equal(sha256(bytes), input.sha256);
  }
  const f = createVrtFixture(directory);
  assert.equal(f.settings.threshold, 100);
  assert.deepEqual(f.settings.viewports, [{ name: "authored-compact", width: 320, height: 180 }]);
  assert.equal(f.delay, 300);
  await writeFile(f.arts.left, f.authored.left.toString().replace("#0000ff", "#00ff00"));
  await rm(f.arts.right);
  writeVrtConfig(f, true);
  for (const input of inputs)
    assert.equal(sha256(await readFile(path.join(directory, input.file))), input.sha256);
  assert.ok(f.root.startsWith(directory + path.sep));
  assert.ok(inputs.every((item) => item.file.startsWith("vrt-inputs/")));
});

test("altered authored Art or configured VRT options refuse before runtime imports", async (t) => {
  for (const kind of ["art", "settings"] as const) {
    const { directory } = await consumer(t);
    const target = path.join(
      directory,
      "vrt-inputs",
      kind === "art" ? "left/Button.art.vue" : "gallery-vrt-options.json",
    );
    if (kind === "art")
      await writeFile(target, (await readFile(target, "utf8")).replace("Left", "Changed"));
    else {
      const config = JSON.parse(await readFile(target, "utf8"));
      config.threshold = 99;
      await writeFile(target, JSON.stringify(config));
    }
    assert.throws(() => createVrtFixture(directory), { name: "AssertionError" });
  }
});

test("an evidence overwrite failure still closes the actual HTTP listener and retains later bytes", async (t) => {
  const { directory } = await consumer(t);
  const e = await evidence(path.join(directory, "evidence"));
  await e.save("body.bin", Buffer.from([0, 255, 128, 10]));
  const server = createServer((_request, response) => response.end("original"));
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  t.after(() => {
    if (server.listening) server.close();
  });
  await assert.rejects(
    cleanup(e, "negative-cleanup", [
      { name: "overwrite", run: () => e.save("body.bin", "changed") },
      {
        name: "real-http-close",
        run: () =>
          new Promise<void>((resolve, reject) =>
            server.close((error) => (error ? reject(error) : resolve())),
          ),
      },
      { name: "later-evidence", run: () => e.save("later.bin", Buffer.from([13, 10, 0])) },
    ]),
    { name: "AggregateError" },
  );
  assert.equal(server.listening, false);
  assert.deepEqual(await readFile(path.join(e.output, "body.bin")), Buffer.from([0, 255, 128, 10]));
  assert.deepEqual(await readFile(path.join(e.output, "later.bin")), Buffer.from([13, 10, 0]));
  assert.deepEqual(
    e.records
      .filter((item) => (item as { phase: string }).phase === "negative-cleanup")
      .map((item) => (item as { operation: string }).operation),
    ["overwrite"],
  );
  await assert.rejects(e.save("../escape.bin", "invalid"), { name: "AssertionError" });
});

test("VRT sealing refuses incomplete success and any different installed package or provider bytes", () => {
  const native = {
    version: "guard-only",
    packages: [{ name: "guard-only", integrity: "sha512-original" }],
    loaded: { entry: "guard-only.node", sha256: "a".repeat(64) },
  };
  const vrt = { ...native, schema: "vize-public-installed-musea-vrt-v1", success: true };
  assert.doesNotThrow(() => assertVrtCustody(vrt, native));
  for (const altered of [
    {},
    { ...vrt, success: false },
    { ...vrt, version: "other" },
    { ...vrt, schema: "other" },
    { ...vrt, packages: [] },
    { ...vrt, loaded: { ...native.loaded, sha256: "b".repeat(64) } },
  ])
    assert.throws(() => assertVrtCustody(altered, native), { name: "AssertionError" });
});

test("VRT runs independently after completed original probes while the original strict seal rejects failure", async () => {
  const workflow = parse(
    await readFile(path.join(root, ".github/workflows/release-public-acceptance.yml"), "utf8"),
  );
  const job = workflow.jobs["public-consumer"];
  const step = job.steps.find((item: { id: string }) => item.id === "vrt");
  assert.equal(job.steps.filter((item: { id: string }) => item.id === "vrt").length, 1);
  assert.equal(job["timeout-minutes"], 30);
  assert.equal(job["continue-on-error"], undefined);
  assert.ok(job.steps.every((item: Record<string, unknown>) => !item["continue-on-error"]));
  assert.equal(
    job.steps.filter((item: { run?: string }) => item.run?.includes("npm install ")).length,
    1,
  );
  assert.match(step.run, /^node vrt_observer\.ts "\$PUBLIC_VERSION" "\$PUBLIC_OUTPUT\/vrt\.json"/);
  assert.equal(step["working-directory"], "${{ env.PUBLIC_CONSUMER }}");
  const prerequisiteIds = ["guard", "prepare", "install", "chromium"];
  const ids = [...prerequisiteIds, "native", "browser", "vrt"];
  const evaluate = (
    condition: string,
    overrides: Record<string, string> = {},
    cancelled = false,
    success = true,
  ) => {
    const steps = Object.fromEntries(
      ids.map((id) => [id, { outcome: overrides[id] ?? "success" }]),
    );
    return runInNewContext(
      condition.slice(3, -2),
      { steps, cancelled: () => cancelled, success: () => success },
      { timeout: 100 },
    );
  };
  for (const native of ["success", "failure"])
    for (const browser of ["success", "failure"])
      assert.equal(evaluate(step.if, { native, browser }, false, false), true);
  for (const id of ids.filter((id) => id !== "vrt"))
    for (const outcome of ["skipped", "cancelled"])
      assert.equal(evaluate(step.if, { [id]: outcome }), false);
  for (const id of prerequisiteIds) assert.equal(evaluate(step.if, { [id]: "failure" }), false);
  assert.equal(evaluate(step.if, {}, true), false);
  const seal = job.steps.find(
    (item: { name: string }) => item.name === "Seal installed public acceptance",
  );
  assert.equal(evaluate(seal.if), true);
  assert.equal(evaluate(seal.if, { vrt: "failure" }, false, false), false);
});
