import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import type { TestContext } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import { readRepoFile } from "./github-workflows.ts";

type Step = { id?: string; name?: string; if?: string; "continue-on-error"?: boolean };
type Workflow = { jobs: { "public-consumer": { steps: Step[]; "continue-on-error"?: boolean } } };
type Outcome = "success" | "failure" | "skipped" | "cancelled";

/** Exercise the actual workflow conditions; these inert laws make no public replay claim. */
export function independentPublicObservationLaw() {
  const workflow = parse(
    readRepoFile(".github", "workflows", "release-public-acceptance.yml"),
  ) as Workflow;
  const job = workflow.jobs["public-consumer"];
  const prerequisiteIds = ["guard", "prepare", "install", "chromium"];
  const ids = [...prerequisiteIds, "native", "browser"];
  for (const id of ids) assert.equal(job.steps.filter((step) => step.id === id).length, 1);
  assert.ok(!job["continue-on-error"]);
  assert.ok(job.steps.every((step) => !step["continue-on-error"]));
  const browser = job.steps.find((step) => step.id === "browser")!;
  const seal = job.steps.find((step) => step.name === "Seal installed public acceptance")!;
  const evaluate = (
    condition: string | undefined,
    overrides: Partial<Record<string, Outcome>> = {},
    cancelled = false,
    success = true,
  ) => {
    assert.ok(condition?.startsWith("${{") && condition.endsWith("}}"));
    const steps = Object.fromEntries(
      ids.map((id) => [id, { outcome: overrides[id] ?? "success" }]),
    );
    return Boolean(
      runInNewContext(
        condition.slice(3, -2),
        { steps, cancelled: () => cancelled, success: () => success },
        { timeout: 100 },
      ),
    );
  };
  assert.equal(evaluate(browser.if), true);
  assert.equal(evaluate(browser.if, { native: "failure" }, false, false), true);
  for (const id of prerequisiteIds) {
    for (const outcome of ["failure", "skipped", "cancelled"] as const)
      assert.equal(evaluate(browser.if, { [id]: outcome, native: "failure" }, false, false), false);
  }
  for (const outcome of ["skipped", "cancelled"] as const)
    assert.equal(evaluate(browser.if, { native: outcome }, false, false), false);
  assert.equal(evaluate(browser.if, { native: "failure" }, true, false), false);
  assert.equal(evaluate(browser.if, {}, true), false);
  assert.equal(evaluate(seal.if), true);
  assert.equal(evaluate(seal.if, {}, false, false), false);
  for (const id of ["native", "browser"]) {
    for (const outcome of ["failure", "skipped", "cancelled"] as const)
      assert.equal(evaluate(seal.if, { [id]: outcome }), false);
  }
}

/** Execute the unchanged native entry with an inert failing provider, without loading a binding. */
export function nativeProviderFailureReceiptLaw(t: TestContext) {
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-native-observation-law-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const version = "0.999.0";
  const packages = [{ name: "inert-provider-fixture", version }];
  const loaded = { entry: path.join(directory, "inert.node"), sha256: "0".repeat(64) };
  const output = path.join(directory, "native.json");
  writeFileSync(
    path.join(directory, "native.ts"),
    readRepoFile("tools", "support", "release", "public_acceptance", "native.ts"),
  );
  writeFileSync(
    path.join(directory, "installed.ts"),
    `import { appendFileSync } from "node:fs";
export function publicNative() {
  return {
    packages: ${JSON.stringify(packages)}, loaded: ${JSON.stringify(loaded)},
    native: { compile(source: string) {
      appendFileSync("calls.jsonl", JSON.stringify(source) + "\\n");
      return { code: source, preamble: "resolveComponent", helpers: [] };
    } },
  };
}
`,
  );
  const run = () =>
    spawnSync(process.execPath, ["native.ts", version, output], {
      cwd: directory,
      env: { ...process.env, NODE_OPTIONS: "" },
      encoding: "utf8",
      timeout: 10_000,
    });
  const failed = run();
  assert.equal(failed.error, undefined);
  assert.notEqual(failed.status, 0);
  assert.match(failed.stderr, /AssertionError/u);
  assert.equal(existsSync(output), false);
  const receiptPath = path.join(directory, "native-provider.json");
  const bytes = readFileSync(receiptPath, "utf8");
  const receipt = JSON.parse(bytes) as Record<string, unknown>;
  assert.deepEqual(receipt, {
    issue: 8328,
    version,
    status: "provider-loaded-before-cases",
    casesVerified: false,
    packages,
    loaded,
  });
  assert.deepEqual(JSON.parse(failed.stdout.trim()), receipt);
  const calls = readFileSync(path.join(directory, "calls.jsonl"), "utf8");
  assert.deepEqual(
    calls
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line)),
    ['<div><div is="vue:my-thing">x</div></div>', "<div><my-thing>x</my-thing></div>"],
  );
  assert.match(run().stderr, /EEXIST/u);
  assert.equal(readFileSync(receiptPath, "utf8"), bytes);
  assert.equal(readFileSync(path.join(directory, "calls.jsonl"), "utf8"), calls);
}
