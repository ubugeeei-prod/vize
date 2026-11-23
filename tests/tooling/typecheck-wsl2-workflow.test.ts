import { readAutomationPrograms, workflowProgram } from "./support/automation-source.ts";
import assert from "node:assert/strict";
import test from "node:test";

const workflow = readAutomationPrograms(
  new URL("../../.github/workflows/typecheck-wsl2.yml", import.meta.url),
  "utf8",
);

test("the Windows mount path survives PowerShell and WSL argument parsing", () => {
  assert.match(workflow, /\$windowsPath = \[System\.IO\.Path\]::GetFullPath/);
  assert.match(workflow, /\$tail = \$windowsPath\.Substring\(2\)\.Replace\('\\', '\/'\)/);
  assert.match(workflow, /\$mount = "\/mnt\/\$drive\$tail"/);
  assert.match(workflow, /wsl\.exe test -d \$mount/);
  assert.doesNotMatch(workflow, /wsl\.exe wslpath/);
});

test("workflow inputs and generated paths cross the Windows to WSL boundary", () => {
  assert.match(
    workflow,
    /WSLENV: "VIZE_SPEC\/u:SIZES\/u:GIANT_BLOCKS\/u:MONOREPO_SFCS\/u:RUNS\/u:WSL_MOUNT_ROOT\/u:GITHUB_STEP_SUMMARY\/u:VIZE_CI_FILES\/p"/,
  );
  assert.match(
    workflow,
    /WSLENV: "VIZE_SPEC\/u:LSP_FIXTURES\/u:WSL_MOUNT_ROOT\/u:GITHUB_STEP_SUMMARY\/u:VIZE_CI_FILES\/p"/,
  );
});

test("editor qualification keeps native TypeScript and tsrs fast as distinct lanes", () => {
  const program = "measure-editor-load-edits-requests-and-memory.cjs";
  assert.ok(workflow.includes('tsrs_spec=$(node "$VIZE_CI_FILES/' + program + '")'));
  assert.match(
    workflowProgram("typecheck-wsl2", workflow, program),
    /typecheck-hosts\/package\.json"\)\.dependencies\.tsrs/,
  );
  assert.match(workflow, /--vize "published-tsgo=\$tools\/node_modules\/\.bin\/vize"/);
  assert.match(workflow, /published-tsgo:CORSA_PATH=\$tsgo/);
  assert.match(workflow, /--vize "published-tsrs-fast=\$tools\/node_modules\/\.bin\/vize"/);
  assert.match(workflow, /published-tsrs-fast:CORSA_PATH=\$tsrs_fast/);
  assert.match(workflow, /export TSRS_FAST=1/);
  assert.match(workflow, /exec "%s" "\$@"/);
  assert.doesNotMatch(workflow, /exec "%s" --fast "\$@"/);
});
