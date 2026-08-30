import assert from "node:assert/strict";
import { mkdtemp, readdir, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  UI_SOURCE_INSTALL_PLAN_SCHEMA_VERSION,
  createUiSourceInstallDryRunPlan,
  type UiSourceInstallPlanInput,
} from "./source-install-planner.ts";

function plan(input: Partial<UiSourceInstallPlanInput> = {}) {
  return createUiSourceInstallDryRunPlan({
    mode: "dry-run",
    requestedFamilies: ["button"],
    destinationRoot: "/workspace/ui",
    sourceFiles: [
      {
        familyName: "button",
        sourcePath: "src/button.ts",
        destinationPath: "button.ts",
        sourceDigest: "sha256:button",
        destinationDigest: null,
      },
    ],
    ...input,
  });
}

void test("emits deterministic dry-run actions in destination path order", () => {
  const input: UiSourceInstallPlanInput = {
    mode: "dry-run",
    requestedFamilies: ["switch", "button", "input", "button"],
    destinationRoot: "/workspace/ui",
    sourceFiles: [
      {
        familyName: "switch",
        sourcePath: "src/switch.ts",
        destinationPath: "form/switch.ts",
        sourceDigest: "sha256:switch",
        destinationDigest: "sha256:switch",
      },
      {
        familyName: "input",
        sourcePath: "src/input.ts",
        destinationPath: "form/input.ts",
        sourceDigest: "sha256:input-next",
        destinationDigest: "sha256:input-prev",
      },
      {
        familyName: "button",
        sourcePath: "src/button.ts",
        destinationPath: "actions/button.ts",
        sourceDigest: "sha256:button",
        destinationDigest: null,
      },
    ],
  };

  const output = createUiSourceInstallDryRunPlan(input);
  const reordered = createUiSourceInstallDryRunPlan({
    ...input,
    requestedFamilies: [...input.requestedFamilies].reverse(),
    sourceFiles: [...input.sourceFiles].reverse(),
  });

  assert.equal(JSON.stringify(output), JSON.stringify(reordered));
  assert.deepEqual(output.requestedFamilies, ["button", "input", "switch"]);
  assert.deepEqual(
    output.actions.map((action) => [action.destinationPath, action.operation]),
    [
      ["actions/button.ts", "create"],
      ["form/input.ts", "overwrite"],
      ["form/switch.ts", "skip"],
    ],
  );
});

void test("fails closed when a requested family is unknown", () => {
  const output = plan({ requestedFamilies: ["button", "ghost"] });

  assert.equal(output.ok, false);
  assert.deepEqual(output.actions, []);
  assert.deepEqual(output.diagnostics, [
    {
      code: "unknown-family",
      familyName: "ghost",
      path: null,
      message: 'Unknown UI source family "ghost"',
    },
  ]);
});

void test("fails closed for unsupported runtime modes", () => {
  const output = createUiSourceInstallDryRunPlan({
    mode: "write",
    requestedFamilies: ["button"],
    destinationRoot: "/workspace/ui",
    sourceFiles: [],
  } as unknown as UiSourceInstallPlanInput);

  assert.equal(output.ok, false);
  assert.deepEqual(output.actions, []);
  assert.deepEqual(output.diagnostics, [
    {
      code: "unsupported-mode",
      familyName: null,
      path: null,
      message: 'Unsupported UI source install mode; expected "dry-run"',
    },
  ]);
});

void test("rejects path traversal before producing a writable action", () => {
  const output = plan({
    sourceFiles: [
      {
        familyName: "button",
        sourcePath: "src/../button.ts",
        destinationPath: "../button.ts",
        sourceDigest: "sha256:button",
      },
    ],
  });

  assert.equal(output.ok, false);
  assert.deepEqual(
    output.actions.map((action) => [action.operation, action.reason, action.targetPath]),
    [["conflict", "path-traversal", null]],
  );
  assert.deepEqual(
    output.diagnostics.map((diagnostic) => [diagnostic.code, diagnostic.path]),
    [
      ["path-traversal", "../button.ts"],
      ["path-traversal", "src/../button.ts"],
    ],
  );
});

void test("reports duplicate destination paths as conflicts", () => {
  const output = plan({
    requestedFamilies: ["button", "input"],
    sourceFiles: [
      {
        familyName: "input",
        sourcePath: "src/input.ts",
        destinationPath: "field.ts",
        sourceDigest: "sha256:input",
      },
      {
        familyName: "button",
        sourcePath: "src/button.ts",
        destinationPath: "field.ts",
        sourceDigest: "sha256:button",
      },
    ],
  });

  assert.equal(output.ok, false);
  assert.deepEqual(
    output.actions.map((action) => [action.familyName, action.operation, action.reason]),
    [
      ["button", "conflict", "duplicate-destination"],
      ["input", "conflict", "duplicate-destination"],
    ],
  );
});

void test("keeps the JSON shape stable for machine consumers", () => {
  const output = plan();

  assert.equal(output.schemaVersion, UI_SOURCE_INSTALL_PLAN_SCHEMA_VERSION);
  assert.equal(
    JSON.stringify(output, null, 2),
    `{
  "schemaVersion": 1,
  "mode": "dry-run",
  "ok": true,
  "destinationRoot": "/workspace/ui",
  "requestedFamilies": [
    "button"
  ],
  "actions": [
    {
      "operation": "create",
      "reason": "destination-missing",
      "familyName": "button",
      "sourcePath": "src/button.ts",
      "destinationPath": "button.ts",
      "targetPath": "/workspace/ui/button.ts",
      "sourceDigest": "sha256:button",
      "destinationDigest": null
    }
  ],
  "diagnostics": []
}`,
  );
});

void test("does not mutate the destination root in dry-run mode", async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), "ui-source-install-plan-"));
  try {
    assert.deepEqual(await readdir(directory), []);
    const output = plan({ destinationRoot: directory });

    assert.equal(output.actions[0]?.operation, "create");
    assert.deepEqual(await readdir(directory), []);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
