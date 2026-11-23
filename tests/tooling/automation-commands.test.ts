import type { ConfigValue } from "../../tools/support/ci/repository-yaml.ts";
import assert from "node:assert/strict";
import { test } from "node:test";
import {
  isProgramInvocation,
  validateAutomationSteps,
} from "../../tools/support/ci/automation-commands.ts";

test("automation steps accept one program invocation with arguments", () => {
  for (const command of [
    'node tools/check.ts --input "$RESULT"',
    "node tools/compiler.ts -p tsconfig.json",
    "vp run --no-cache --filter './npm/native' build:debug",
    '. "$GITHUB_ACTION_PATH/scripts/install.sh"',
    '. "${{ env.VIZE_CI_FILES }}/measure.ps1"',
    'node tools/check.ts "${{ inputs.value || github.sha }}"',
    "git log --format='left|right'",
  ])
    assert.ok(isProgramInvocation(command), command);
});

test("automation steps reject compound commands and embedded interpreter programs", () => {
  for (const command of [
    "prepare && check",
    "run | tee log",
    "first; second",
    "echo ok > result",
    "go test\ncheck",
    "if true; then exit 1; fi",
    "CHECK=true node script.mjs",
    "node --input-type=module -e 'console.log(1)'",
    "node -p '42'",
    "python3 - <<'PY'\nprint(42)\nPY",
    "bash -ec 'check'",
    "env bash -c 'run'",
    "pwsh -Command 'Get-Process'",
    'eval "$CODE"',
    'echo "$(generate)"',
  ])
    assert.equal(isProgramInvocation(command), false, command);
  const embedded: ConfigValue[] = [
    { uses: "actions/github-script@v8" },
    { with: { script: "return 1" } },
  ];
  for (const step of embedded) {
    assert.throws(
      () => validateAutomationSteps("action.yml", { runs: { steps: [step] } }),
      /inline code/u,
    );
  }
});
