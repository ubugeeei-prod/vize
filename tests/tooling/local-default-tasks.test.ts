import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { buildTasks } from "../../config/vite-plus/tasks/build.ts";
import { checkTasks } from "../../config/vite-plus/tasks/check.ts";
import { testAndBenchmarkTasks } from "../../config/vite-plus/tasks/test-benchmark.ts";
import { inTestbox, testboxTasks } from "../../config/vite-plus/tasks/testbox.ts";
import { shellQuote } from "../../config/vite-plus/task-helpers.ts";
import { testedPackages } from "../../config/vite-plus/task-inputs.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const pushCurrentBranchCommand = 'git push --set-upstream origin "$(git branch --show-current)"';

const createFakeBlacksmith = (body: string) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-blacksmith-"));
  const executable = path.join(directory, "blacksmith");
  fs.writeFileSync(executable, `#!/bin/sh\n${body}\n`, { mode: 0o755 });
  return { directory, executable };
};

const extractTestboxLifecycle = (guide: string): string => {
  const lifecycle = guide.match(
    /```sh\n(run_testbox_checks\(\) \{[\s\S]*?\nrun_testbox_checks)\n```/,
  )?.[1];
  assert.ok(lifecycle, "guarded Testbox lifecycle");
  return lifecycle;
};

type TaskShape = {
  cache?: boolean;
  command: string;
};

const taskShape = (value: unknown) => value as TaskShape;

const assertLocalDefault = (
  localValue: unknown,
  testboxValue: unknown,
  nestedTasks: readonly string[],
) => {
  const local = taskShape(localValue);
  const testbox = taskShape(testboxValue);

  assert.equal(local.cache, false);
  assert.doesNotMatch(local.command, /blacksmith|BLACKSMITH_TESTBOX_ID/);
  assert.equal(testbox.cache, false);
  assert.match(
    testbox.command,
    /"\$VIZE_BLACKSMITH_BIN" testbox run --id "\$BLACKSMITH_TESTBOX_ID"/,
  );

  const expectedCommand = nestedTasks.map((task) => `vp run --workspace-root ${task}`).join(" && ");
  assert.ok(local.command.includes(expectedCommand));
  assert.ok(testbox.command.includes(shellQuote(expectedCommand)));

  for (const nestedTask of nestedTasks) {
    const command = `vp run --workspace-root ${nestedTask}`;
    assert.ok(local.command.includes(command));
    assert.ok(testbox.command.includes(command));
  }
};

test("workspace build, test, and lint default to their local task graphs", () => {
  assertLocalDefault(buildTasks.build, buildTasks["build:testbox"], ["build:rust", "build:all"]);
  assertLocalDefault(testAndBenchmarkTasks.test, testAndBenchmarkTasks["test:testbox"], [
    "test:rust",
    "test:js",
    "test:scripts",
    "test:zed-extension:unit",
  ]);
  assertLocalDefault(checkTasks.lint, checkTasks["lint:testbox"], ["check"]);

  const ci = taskShape(checkTasks.ci);
  assert.equal(ci.cache, false);
  assert.match(ci.command, /vp run --workspace-root test(?:\s|$)/);
  assert.doesNotMatch(ci.command, /blacksmith|test:testbox/);
});

test("workspace JS tests include the Fresco component suite", () => {
  assert.ok(testedPackages.includes("./npm/fresco"));

  const jsTests = taskShape(testAndBenchmarkTasks["test:js"]);
  assert.match(jsTests.command, /--filter '\.\/npm\/fresco'(?:\s|$)/);
});

test("branch coverage reports every metric before enforcing thresholds", () => {
  const command = taskShape(testAndBenchmarkTasks["coverage:source:branch"]).command;
  const cleanIndex = command.indexOf("cargo clean --target-dir target/llvm-cov-target");
  const reportIndex = command.indexOf("cargo +nightly llvm-cov -p vize_carton");
  const enforcementIndex = command.indexOf("enforce_rust_source_coverage");

  assert.ok(cleanIndex >= 0);
  assert.match(command, /cargo \+nightly llvm-cov/);
  assert.match(command, /--branch(?:\s|$)/);
  assert.doesNotMatch(command, /--fail-under-/);
  assert.match(command, /enforce_rust_source_coverage/);
  assert.match(command, /--min-lines 55/);
  assert.match(command, /--min-functions 70/);
  assert.match(command, /--min-regions 55/);
  assert.match(command, /--min-branches 40/);
  assert.ok(cleanIndex < reportIndex);
  assert.ok(reportIndex < enforcementIndex);
});

test("Testbox commands fail with an actionable lifecycle when the box id is absent", () => {
  const result = spawnSync(
    "/bin/sh",
    ["-c", `blacksmith() { :; }\n${inTestbox("printf should-not-run")}`],
    {
      encoding: "utf8",
      env: { PATH: "", VIZE_TESTBOX_SHELL: "1", VIZE_BLACKSMITH_BIN: process.execPath },
    },
  );

  assert.equal(result.status, 2);
  assert.equal(result.stdout, "");
  assert.match(result.stderr, /BLACKSMITH_TESTBOX_ID is unset/);
  assert.match(result.stderr, /"\$VIZE_BLACKSMITH_BIN" auth login/);
  assert.match(result.stderr, /git push --set-upstream origin/);
  assert.match(result.stderr, /vp run --workspace-root testbox:warmup/);
  assert.match(result.stderr, /guarded lifecycle in CONTRIBUTING\.md/);
});

test("Testbox commands require the dedicated Nix shell", () => {
  const result = spawnSync("/bin/sh", ["-c", inTestbox("printf should-not-run")], {
    encoding: "utf8",
    env: { PATH: process.env.PATH ?? "" },
  });

  assert.equal(result.status, 2);
  assert.equal(result.stdout, "");
  assert.match(result.stderr, /require the pinned Blacksmith environment/);
  assert.match(result.stderr, /nix develop \.#testbox/);
  assert.doesNotMatch(result.stderr, /BLACKSMITH_TESTBOX_ID is unset/);
});

test("Testbox commands reject an inherited marker without the pinned executable", () => {
  const result = spawnSync("/bin/sh", ["-c", inTestbox("printf should-not-run")], {
    encoding: "utf8",
    env: { PATH: process.env.PATH ?? "", VIZE_TESTBOX_SHELL: "1" },
  });

  assert.equal(result.status, 2);
  assert.match(result.stderr, /Pinned Blacksmith CLI is unavailable/);
  assert.doesNotMatch(result.stderr, /BLACKSMITH_TESTBOX_ID is unset/);
});

test("Testbox warmup diagnoses branch and authentication prerequisites", () => {
  const command = taskShape(testboxTasks["testbox:warmup"]).command;
  const syntax = spawnSync("sh", ["-n"], { encoding: "utf8", input: command });

  assert.equal(syntax.status, 0, syntax.stderr);
  assert.match(command, /git branch --show-current/);
  assert.match(command, /git ls-remote --heads origin/);
  assert.match(command, /git rev-parse HEAD/);
  assert.match(command, /local_sha/);
  assert.match(command, /remote_sha/);
  assert.match(command, /could not query origin/);
  assert.match(command, /git push --set-upstream origin/);
  assert.match(command, /"\$VIZE_BLACKSMITH_BIN" testbox warmup \.github\/workflows\/e2e\.yml/);
  assert.match(command, /--ref "\$branch" --job testbox/);
  assert.match(
    command,
    new RegExp(pushCurrentBranchCommand.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
  );
});

test("Testbox warmup distinguishes origin failures from an unpushed branch", () => {
  const command = taskShape(testboxTasks["testbox:warmup"]).command;
  const fake = createFakeBlacksmith(
    "printf '%s\\n' \"$*\" >> \"$FAKE_BLACKSMITH_LOG\"\nprintf '%s\\n' tbx_testbox_id",
  );
  const blacksmithLog = path.join(fake.directory, "calls.log");
  const harness = [
    "git() {",
    '  if [ "$1" = "branch" ]; then printf \'%s\\n\' "$FAKE_BRANCH"; return; fi',
    '  if [ "$1" = "rev-parse" ]; then printf \'%s\\n\' "$FAKE_LOCAL_SHA"; return; fi',
    "  if [ \"$FAKE_REMOTE\" = \"error\" ]; then printf '%s\\n' 'credential detail that must stay hidden' >&2; return 3; fi",
    '  if [ "$FAKE_REMOTE" != "missing" ]; then printf \'%s\\n\' "$FAKE_REMOTE_SHA refs/heads/$FAKE_BRANCH"; fi',
    "}",
    "blacksmith() { printf '%s\\n' 'host Blacksmith must not run' >&2; return 91; }",
    command,
  ].join("\n");

  const runWarmup = (extraEnv: NodeJS.ProcessEnv) =>
    spawnSync("sh", ["-c", harness], {
      encoding: "utf8",
      env: {
        PATH: process.env.PATH ?? "",
        VIZE_TESTBOX_SHELL: "1",
        VIZE_BLACKSMITH_BIN: fake.executable,
        FAKE_BLACKSMITH_LOG: blacksmithLog,
        FAKE_BRANCH: "feature/testbox",
        FAKE_LOCAL_SHA: "deadbeef",
        FAKE_REMOTE_SHA: "deadbeef",
        ...extraEnv,
      },
    });

  try {
    const remoteError = runWarmup({ FAKE_REMOTE: "error" });
    assert.equal(remoteError.status, 2);
    assert.match(remoteError.stderr, /could not query origin/);
    assert.doesNotMatch(remoteError.stderr, /credential detail/);

    const hostileBranch = "feature/'$(printf-pwned)'";
    assert.equal(spawnSync("git", ["check-ref-format", "--branch", hostileBranch]).status, 0);
    const missingBranch = runWarmup({ FAKE_REMOTE: "missing", FAKE_BRANCH: hostileBranch });
    assert.equal(missingBranch.status, 2);
    assert.match(missingBranch.stderr, /cannot find the current branch on origin/);
    assert.ok(missingBranch.stderr.includes(`Push it first: ${pushCurrentBranchCommand}`));
    assert.ok(!missingBranch.stderr.includes(hostileBranch));

    const staleBranch = runWarmup({ FAKE_REMOTE: "found", FAKE_LOCAL_SHA: "cafebabe" });
    assert.equal(staleBranch.status, 2);
    assert.match(staleBranch.stderr, /needs the current HEAD on origin/);
    assert.ok(staleBranch.stderr.includes(`Push it first: ${pushCurrentBranchCommand}`));

    const success = runWarmup({ FAKE_REMOTE: "found" });
    assert.equal(success.status, 0);
    assert.equal(success.stdout, "tbx_testbox_id\n");
    assert.doesNotMatch(success.stderr, /host Blacksmith/);
    assert.match(
      fs.readFileSync(blacksmithLog, "utf8"),
      /^testbox warmup \.github\/workflows\/e2e\.yml --ref feature\/testbox --job testbox\n$/,
    );
  } finally {
    fs.rmSync(fake.directory, { recursive: true, force: true });
  }
});

test("the Nix shell exposes local and opt-in Testbox task aliases", () => {
  const vpModule = fs.readFileSync(path.join(root, "nix/vp.nix"), "utf8");
  const devShellModule = fs.readFileSync(path.join(root, "nix/dev-shell.nix"), "utf8");
  const blacksmithModule = fs.readFileSync(path.join(root, "nix/blacksmith.nix"), "utf8");
  const defaultShell = devShellModule.match(
    /devShell = pkgs\.mkShell \{([\s\S]*?)\n\s*\};\n\s*testboxDevShell/,
  )?.[1];
  const testboxShell = devShellModule.match(
    /testboxDevShell = devShell\.overrideAttrs \(previous: \{([\s\S]*?)\n\s*\}\);/,
  )?.[1];

  for (const taskName of ["build", "test", "lint"]) {
    assert.match(vpModule, new RegExp(`(?:\\||\\s)${taskName}(?:\\s|\\||\\))`));
  }

  assert.match(vpModule, /\| \*:\*\)/);
  assert.match(vpModule, /exec "\$local_vp" run --workspace-root "\$1"/);
  assert.ok(defaultShell, "default dev shell");
  assert.match(defaultShell, /\$\{clearTestboxEnvironment\}/);
  assert.doesNotMatch(defaultShell, /activateTestboxEnvironment/);
  assert.ok(testboxShell, "Testbox dev shell");
  assert.match(testboxShell, /\$\{activateTestboxEnvironment\}/);
  assert.match(blacksmithModule, /unset VIZE_TESTBOX_SHELL VIZE_BLACKSMITH_BIN/);
  assert.match(blacksmithModule, /export VIZE_TESTBOX_SHELL=1/);
  assert.match(blacksmithModule, /export VIZE_BLACKSMITH_BIN="\$\{blacksmith\}\/bin\/blacksmith"/);
  assert.match(blacksmithModule, /testbox-environment = testboxEnvironmentCheck/);
  assert.match(testboxShell, /safe ID capture/);
  assert.match(testboxShell, /vp build:testbox \/ vp test:testbox \/ vp lint:testbox/);
});

test("contributor docs use workspace-explicit commands outside the Nix aliases", () => {
  for (const relativePath of ["CONTRIBUTING.md", "docs/content/contributing.md"]) {
    const guide = fs.readFileSync(path.join(root, relativePath), "utf8");

    for (const taskName of ["build", "test", "lint"]) {
      assert.match(guide, new RegExp(`vp run --workspace-root ${taskName}`));
    }
    assert.match(guide, /Inside the Nix development shell/);
    assert.match(guide, /nix develop \.#testbox/);
    assert.match(guide, /default `nix develop` shell\s+intentionally omits Blacksmith/);
    const lifecycle = extractTestboxLifecycle(guide);
    assert.ok(
      lifecycle.indexOf("unset BLACKSMITH_TESTBOX_ID") < lifecycle.indexOf("testbox:warmup"),
    );
    assert.match(lifecycle, /"\$VIZE_BLACKSMITH_BIN" auth login/);
    assert.match(lifecycle, /if testbox_output="\$\(vp run --workspace-root testbox:warmup\)"/);
    assert.match(lifecycle, /if vp run --workspace-root testbox:stop/);
  }
});

test("documented Testbox lifecycle rejects stale ids and always stops after task failure", () => {
  const guides = ["CONTRIBUTING.md", "docs/content/contributing.md"].map((relativePath) =>
    fs.readFileSync(path.join(root, relativePath), "utf8"),
  );
  const lifecycle = extractTestboxLifecycle(guides[0]);
  assert.equal(extractTestboxLifecycle(guides[1]), lifecycle);
  const definition = lifecycle.replace(/\nrun_testbox_checks$/, "");
  const fake = createFakeBlacksmith('[ "$1 $2" = "auth login" ]');
  const log = path.join(fake.directory, "lifecycle.log");
  const harness = [
    'git() { if [ "$1" = "branch" ]; then printf \'%s\\n\' feature/testbox; fi; }',
    "vp() {",
    '  printf \'%s|id=%s\\n\' "$*" "${BLACKSMITH_TESTBOX_ID-unset}" >> "$FAKE_LIFECYCLE_LOG"',
    '  case "$*" in',
    '    "run --workspace-root testbox:warmup")',
    "      if [ \"$FAKE_WARMUP_STATUS\" -eq 0 ]; then printf '%s\\n' fresh-box; fi",
    '      return "$FAKE_WARMUP_STATUS" ;;',
    '    "run --workspace-root build:testbox") return "${FAKE_TASK_STATUS:-0}" ;;',
    '    "run --workspace-root testbox:stop") return "${FAKE_STOP_STATUS:-0}" ;;',
    "  esac",
    "}",
    definition,
    "run_testbox_checks",
    "lifecycle_status=$?",
    'printf \'final|id=%s\\n\' "${BLACKSMITH_TESTBOX_ID-unset}" >> "$FAKE_LIFECYCLE_LOG"',
    'exit "$lifecycle_status"',
  ].join("\n");
  const runLifecycle = (extraEnv: NodeJS.ProcessEnv) =>
    spawnSync("/bin/sh", ["-c", harness], {
      encoding: "utf8",
      env: {
        PATH: process.env.PATH ?? "",
        VIZE_BLACKSMITH_BIN: fake.executable,
        FAKE_LIFECYCLE_LOG: log,
        BLACKSMITH_TESTBOX_ID: "stale-box",
        ...extraEnv,
      },
    });

  try {
    const failedWarmup = runLifecycle({ FAKE_WARMUP_STATUS: "17" });
    assert.equal(failedWarmup.status, 17);
    assert.equal(
      fs.readFileSync(log, "utf8"),
      "run --workspace-root testbox:warmup|id=unset\nfinal|id=unset\n",
    );

    fs.writeFileSync(log, "");
    const failedTask = runLifecycle({ FAKE_WARMUP_STATUS: "0", FAKE_TASK_STATUS: "23" });
    assert.equal(failedTask.status, 23);
    assert.equal(
      fs.readFileSync(log, "utf8"),
      [
        "run --workspace-root testbox:warmup|id=unset",
        "run --workspace-root build:testbox|id=fresh-box",
        "run --workspace-root testbox:stop|id=fresh-box",
        "final|id=unset",
        "",
      ].join("\n"),
    );
  } finally {
    fs.rmSync(fake.directory, { recursive: true, force: true });
  }
});
