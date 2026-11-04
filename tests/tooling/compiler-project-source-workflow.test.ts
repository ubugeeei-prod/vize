import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";
import { planAffectedJs } from "../../tools/support/compat/github/plan-affected-js.ts";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const workflowPath = ".github/workflows/pr-js-checks.yml";
const driver = "tests/native/imported-type-project-stats.ts";
const testPath = "tests/tooling/compiler-project-source-workflow.test.ts";
const proofName = "Verify imported type projects with the built native image";
type Step = {
  name?: string;
  if?: string;
  run?: string;
  shell?: string;
  uses?: string;
  env?: Record<string, string>;
  with?: Record<string, string | number>;
  "continue-on-error"?: boolean;
};
type Workflow = { jobs: Record<string, { steps: Step[]; needs?: string[] }> };
const workflow = parse(fs.readFileSync(path.join(root, workflowPath), "utf8")) as Workflow;
const steps = workflow.jobs["pr-js-package-conformance"]!.steps;
const required = (name: string): Step => {
  const step = steps.find((entry) => entry.name === name);
  assert.ok(step, `missing source qualification step: ${name}`);
  return step;
};

test("the existing UI lane runs the strict project driver against exactly one built image", () => {
  const proof = required(proofName);
  assert.equal(proof.if, required("Check UI package conformance").if);
  assert.equal(proof.shell, "bash");
  assert.equal(proof["continue-on-error"], undefined);
  assert.deepEqual(proof.env, {
    VIZE_IMPORTED_TYPES_REQUIRE_SOURCE: "1",
    VIZE_IMPORTED_TYPES_SOURCE_SHA: "${{ github.sha }}",
  });
  assert.ok(proof.run);
  assert.match(proof.run, /shopt -s nullglob/);
  assert.match(proof.run, /test "\$\{#bindings\[@\]\}" -eq 1/);
  assert.ok(proof.run.includes(`node ${driver} "${"${bindings[0]}"}" "$capture/receipt.json"`));
  assert.doesNotMatch(proof.run, /(?:cargo |build:|\.mjs|\|\| true)/);
  assert.ok(steps.indexOf(proof) > steps.indexOf(required("Check UI package conformance")));
  assert.ok(
    steps.indexOf(proof) >
      steps.indexOf(required("Verify whole source CSS API maps with the built native image")),
  );
});

test("failed or cancelled source project execution retains raw receipts in the existing job", () => {
  const upload = required("Retain imported type project inputs and source receipts");
  assert.equal(
    upload.if,
    `\${{ always() && inputs.run-js && matrix.surface == 'ui' && fromJSON(inputs.js-plan).conformance.ui }}`,
  );
  assert.equal(upload.uses, "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a");
  assert.equal(upload.with?.path, "${{ runner.temp }}/imported-type-project/");
  assert.equal(upload.with?.["if-no-files-found"], "error");
  assert.equal(upload["continue-on-error"], undefined);
  assert.ok(steps.indexOf(upload) > steps.indexOf(required(proofName)));
  const source = parse(
    fs.readFileSync(path.join(root, ".github/workflows/pr-source-checks.yml"), "utf8"),
  ) as Workflow;
  assert.ok(source.jobs["source-report"]!.needs?.includes("pr-js-packages"));
});

test("the strict driver selects source-native UI conformance through the package dependency graph", () => {
  const selected = planAffectedJs(
    [
      { name: "@vizejs/native", path: "npm/native", dependencies: [] },
      { name: "@vizejs/ui", path: "npm/ui", dependencies: ["@vizejs/native"] },
      { name: "@vizejs/fresco-native", path: "npm/fresco-native", dependencies: [] },
    ],
    [driver],
  );
  assert.equal(selected.scope, "affected");
  assert.equal(selected.native, true);
  assert.deepEqual(selected.conformance, { fresco: false, ui: true });
});

test("workflow and strict driver changes select the source workflow contract", () => {
  for (const input of [workflowPath, driver]) {
    assert.ok(planToolingTests([input], { cwd: root }).tests.includes(testPath), input);
  }
});

test("the actual workflow command keeps the driver failure and rejects ambiguous native images", () => {
  const proof = required(proofName);
  assert.ok(proof.run);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-project-workflow-"));
  try {
    const native = path.join(directory, "npm/native");
    const bin = path.join(directory, "bin");
    fs.mkdirSync(native, { recursive: true });
    fs.mkdirSync(bin);
    fs.writeFileSync(path.join(native, "source.node"), "controlled source image");
    fs.writeFileSync(
      path.join(bin, "node"),
      '#!/bin/bash\nprintf "%s\\n" "$@" > "$MOCK_ARGS"\nprintf "controlled failure\\n" > "$3"\nexit 23\n',
      { mode: 0o755 },
    );
    const args = path.join(directory, "args.txt");
    const env = {
      ...process.env,
      PATH: `${bin}:/usr/bin:/bin`,
      GITHUB_WORKSPACE: directory,
      RUNNER_TEMP: directory,
      MOCK_ARGS: args,
    };
    const run = () =>
      spawnSync("/bin/bash", ["-e", "-o", "pipefail", "-c", proof.run!], {
        cwd: directory,
        env,
        encoding: "utf8",
      });
    assert.equal(run().status, 23);
    assert.deepEqual(fs.readFileSync(args, "utf8").trim().split("\n"), [
      driver,
      path.join(native, "source.node"),
      path.join(directory, "imported-type-project/receipt.json"),
    ]);
    assert.equal(
      fs.readFileSync(path.join(directory, "imported-type-project/receipt.json"), "utf8"),
      "controlled failure\n",
    );
    fs.unlinkSync(args);
    fs.writeFileSync(path.join(native, "foreign.node"), "second image");
    assert.equal(run().status, 1);
    assert.equal(fs.existsSync(args), false);
    fs.unlinkSync(path.join(native, "source.node"));
    fs.unlinkSync(path.join(native, "foreign.node"));
    assert.equal(run().status, 1);
    assert.equal(fs.existsSync(args), false);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
