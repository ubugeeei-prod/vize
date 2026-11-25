import type { ConfigValue } from "../../tools/support/ci/repository-yaml.ts";
import assert from "node:assert/strict";
import { test } from "node:test";
import {
  automationSizeViolations,
  isProgramInvocation,
  validateAutomationSteps,
  validateShellProgram,
  automationProgramReferences,
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
    "test -s result.json",
    "printf ok",
    "exit 1",
    "return 3",
    "unset NAME",
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
    "bash.exe -s",
    "deno.exe eval '1'",
    "node --import 'data:text/javascript,0' check.ts",
    "node --loader=data:text/javascript,0 check.ts",
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

test("standalone drivers reject embedded interpreters while accepting real program arguments", () => {
  for (const command of [
    "node --input-type=module -e '42'",
    "python3 <<'PY'\nprint(1)\nPY",
    "docker run example sh -euxc 'test -f result'",
    "pwsh -NoProfile -Command run",
    "bun eval '1'",
  ]) {
    assert.throws(() => validateShellProgram("driver.sh", command), /named script/u);
  }
  for (const command of [
    "docker run example sh -eux /work/smoke.sh",
    'node "$VIZE_CI_FILES/check.ts" -p tsconfig.json',
    "cat <<EOF > data.json\n{}\nEOF",
  ])
    validateShellProgram("driver.sh", command);
  assert.deepEqual(automationProgramReferences('node "$VIZE_CI_FILES/../shared/receipts.ts"'), [
    "../shared/receipts.ts",
  ]);
});

test("operational text filters load a named file", () => {
  const driver = ".github/scripts/workflows/report/summary.sh";
  for (const command of [
    "awk '{print $1}'",
    "jq -r .id",
    "sed -n '1,2p'",
    "gh api repos/example/project --jq .id",
    "gh.exe api repos/example/project -q .id",
  ]) {
    assert.equal(isProgramInvocation(command), false);
    assert.throws(() => validateShellProgram(driver, "cat input | " + command), /named script/u);
  }
  for (const command of [
    "awk -v column=1 -f threads.awk",
    "jq --arg name value -ef select.jq",
    "sed -n -f header.sed",
  ]) {
    assert.ok(isProgramInvocation(command));
    validateShellProgram(driver, command);
  }
  assert.deepEqual(automationProgramReferences('jq -f "$VIZE_CI_FILES/../shared/artifact.jq"'), [
    "../shared/artifact.jq",
  ]);
});

test("long inline commands, test lists and conditions belong in named programs", () => {
  const long = `node tools/run.ts ${"--flag ".repeat(30)}`.trim();
  const document = {
    jobs: {
      build: {
        if: `\${{ ${"github.event_name == 'push' || ".repeat(8)}false }}`,
        steps: [
          { name: "Short", run: "node tools/run.ts --flag" },
          { name: "Long", run: long },
          { name: "Tests", run: "node --test a.test.ts b.test.ts c.test.ts" },
          { name: "Two tests", run: "node --test a.test.ts b.test.mjs" },
          { name: "Named", run: '. "${{ env.VIZE_CI_FILES }}/long-command.sh"' },
        ],
      },
    },
  };
  assert.deepEqual(automationSizeViolations("ci.yml", document), [
    "ci.yml / build: condition longer than 200 characters",
    "ci.yml / build / Long: command longer than 160 characters",
    "ci.yml / build / Tests: more than two inline test files",
  ]);
});
