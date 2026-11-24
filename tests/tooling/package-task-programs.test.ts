import { taskCatalog } from "../../tools/config/vite-plus/task-groups.ts";
import { workspaceTaskSteps, steps } from "../../tools/config/vite-plus/task-plan.ts";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, realpathSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  isProgramCommand,
  isProgramInvocation,
} from "../../tools/support/ci/automation-commands.ts";
import { validatePackageScripts } from "../../tools/support/ci/package-programs.ts";
import { runPackageTask } from "../../tools/support/tasks/package-task.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));

test("every repository package script invokes one program", () => {
  const paths = execFileSync("git", ["ls-files", "-z", "--", "package.json", "**/package.json"], {
    cwd: root,
    encoding: "utf8",
  })
    .split("\0")
    .filter(Boolean);
  for (const file of paths) {
    if (file.split("/").some((part) => ["_fixtures", "fixtures", "upstream"].includes(part)))
      continue;
    const manifest = JSON.parse(readFileSync(path.join(root, file), "utf8")) as {
      scripts?: Record<string, string>;
    };
    for (const [name, command] of Object.entries(manifest.scripts ?? {}))
      assert.ok(isProgramInvocation(command), file + " / " + name + ": " + command);
  }
});

test("packages can introduce their first command and must declare their first generated task", (t) => {
  const root = mkdtempSync(path.join(tmpdir(), "vize-package-manifest-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const manifest = path.join(root, "package.json");
  writeFileSync(manifest, JSON.stringify({ scripts: { clean: "node clean.ts" } }));
  validatePackageScripts(root, ".", []);
  writeFileSync(manifest, JSON.stringify({ scripts: { clean: "clean && verify" } }));
  assert.throws(() => validatePackageScripts(root, ".", []), /Pkl task definition/u);
  writeFileSync(
    manifest,
    JSON.stringify({ scripts: { clean: "node ./scripts/package-tasks.cjs clean" } }),
  );
  assert.throws(() => validatePackageScripts(root, ".", []), /undefined package task clean/u);
  validatePackageScripts(root, ".", ["clean"]);
});

test("every workspace task invokes one named entrypoint", () => {
  for (const [name, task] of Object.entries(taskCatalog)) {
    assert.ok(
      typeof task === "object" &&
        "command" in task &&
        typeof task.command === "string" &&
        isProgramInvocation(task.command),
      name,
    );
    for (const step of workspaceTaskSteps(task))
      assert.ok(isProgramCommand([step.program, ...(step.arguments ?? [])]), name);
  }
  assert.deepEqual(steps("node probe.js ''"), [{ program: "node", arguments: ["probe.js", ""] }]);
  assert.throws(() => steps({ program: "sh", arguments: ["-c", "echo hi"] }), /named program/);
});
test("package sequences preserve cwd, environment, forwarded arguments and the first failure", (t) => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-package-task-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const other = path.join(directory, "working directory");
  mkdirSync(other);
  const recorder = fileURLToPath(new URL("./support/package-task-recorder.ts", import.meta.url));
  const output = path.join(directory, "receipts");
  const command = (work: string, status: number) => ({
    program: process.execPath,
    arguments: [recorder, output, String(status)],
    directory: work,
    defaults: { VIZE_PACKAGE_TASK_PROBE: "fallback" },
  });
  const first = { ...command(directory, 0), environment: { VIZE_PACKAGE_TASK_PROBE: "provided" } };
  const extra = ["path with spaces", "a&b", 'quote"and%percent'];
  assert.equal(runPackageTask({ run: [first, command(other, 0)] }, ["run", ...extra]), 0);
  const records = () =>
    readFileSync(output, "utf8")
      .trimEnd()
      .split("\n")
      .map((line) => JSON.parse(line) as unknown);
  assert.deepEqual(records(), [
    [realpathSync(directory), "provided", []],
    [realpathSync(other), "fallback", extra],
  ]);
  assert.equal(runPackageTask({ run: [command(directory, 23), first] }, ["run"]), 23);
  assert.equal(records().length, 3);
  assert.throws(() => runPackageTask({}, ["toString"]), /Unknown package task/u);
});
