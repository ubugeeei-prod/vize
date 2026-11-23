import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { automationShellPath, readAutomationPrograms } from "./support/automation-source.ts";
import { installToolingProgram, toolingProgram } from "./support/program-path.ts";

const workflow = parse(
  readAutomationPrograms(".github", "workflows", "nuxt3-module-build.yml"),
) as {
  jobs: Record<string, { steps: Array<{ name?: string; run?: string }> }>;
};
const setup = workflow.jobs["nuxt3-module-build"].steps.find(
  (step) => step.name === "Download Chromium dependencies while building the current CLI",
)?.run;
assert.ok(setup);
const setupFile = automationShellPath(setup, "nuxt3-module-build");

function probe(downloadStatus: number, cliStatus: number, installStatus = 0, cancel = false) {
  const dir = mkdtempSync(path.join(tmpdir(), "vize-nuxt-cli-browser-"));
  try {
    for (const name of ["bin", "tests", "config", "cache/archives"]) {
      mkdirSync(path.join(dir, name), { recursive: true });
    }
    const original = path.join(dir, "config", "99-vize-nuxt-archives-original");
    const target = path.join(dir, "unowned-target");
    writeFileSync(original, "original configuration\n");
    writeFileSync(target, "unowned symlink target\n");
    symlinkSync(target, path.join(dir, "config", "98-vize-nuxt-download-only-original"));
    writeFileSync(path.join(dir, "cache", "archives", "default.deb"), "default archive\n");
    assert.equal(spawnSync("mkfifo", [path.join(dir, "ready"), path.join(dir, "gate")]).status, 0);

    installToolingProgram(path.join(dir, "bin", "sudo"), "workflow/translate-apt-paths.cjs");
    installToolingProgram(path.join(dir, "bin", "cargo"), "workflow/observe-apt-build.cjs");
    installToolingProgram(path.join(dir, "bin", "vp"), "workflow/run-apt-install.cjs");
    installToolingProgram(
      path.join(dir, "bin", "vize-ci-apt-retry"),
      "workflow/forward-apt-command.cjs",
    );
    const result = spawnSync(
      "bash",
      ["-e", "-o", "pipefail", toolingProgram("workflow/observe-apt-setup.sh"), setupFile],
      {
        cwd: dir,
        encoding: "utf8",
        timeout: 8000,
        env: {
          ...process.env,
          PATH: `${path.join(dir, "bin")}:${process.env.PATH}`,
          PROBE_DIR: dir,
          VIZE_CI_FILES: path.dirname(setupFile),
          HELD_PROCESS: toolingProgram("process/hold-process-open.js"),
          DOWNLOAD_STATUS: String(downloadStatus),
          CLI_STATUS: String(cliStatus),
          INSTALL_STATUS: String(installStatus),
          CANCEL: cancel ? "1" : "0",
        },
      },
    );
    assert.ifError(result.error);
    assert.equal(result.signal, null, result.stderr);
    assert.equal(readFileSync(original, "utf8"), "original configuration\n");
    assert.equal(readFileSync(target, "utf8"), "unowned symlink target\n");
    assert.deepEqual(
      spawnSync("find", [path.join(dir, "cache"), "-maxdepth", "1", "-type", "d"], {
        encoding: "utf8",
      })
        .stdout.trim()
        .split("\n")
        .sort(),
      [path.join(dir, "cache"), path.join(dir, "cache", "archives")].sort(),
    );
    assert.equal(
      spawnSync("find", [path.join(dir, "config"), "-type", "f"], {
        encoding: "utf8",
      }).stdout.trim(),
      original,
    );
    const events = readFileSync(path.join(dir, "events"), "utf8").trim().split("\n");
    assert.ok(events.includes("child-closed"));
    assert.ok(events.includes("cli-done"));
    return { status: result.status, events, output: result.stdout };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

const linux = { skip: process.platform !== "linux" };

test(
  "Nuxt setup joins both commands, retains private archives through update hooks, and preserves unrelated configuration",
  linux,
  () => {
    const result = probe(0, 0);
    assert.equal(result.status, 0);
    assert.ok(result.events.indexOf("download-start") < result.events.indexOf("cli-done"));
    assert.ok(result.events.indexOf("install-start") > result.events.indexOf("cli-done"));
    assert.ok(result.events.includes("archive-reused"));
    assert.equal(
      result.output.match(/Nuxt3 archive inventory after package-index update/g)?.length,
      2,
    );
  },
);

test(
  "Nuxt setup preserves either command failure, reaps the CLI, and refuses installation",
  linux,
  () => {
    for (const [download, cli, expected] of [
      [23, 0, 23],
      [0, 11, 11],
      [23, 11, 23],
    ]) {
      const result = probe(download, cli);
      assert.equal(result.status, expected);
      assert.ok(result.output.includes(`Nuxt3 setup statuses: download=${download} CLI=${cli}`));
      assert.ok(!result.events.includes("install-start"));
    }
  },
);

test("Nuxt setup preserves the original install failure after both commands finish", linux, () => {
  const result = probe(0, 0, 29);
  assert.equal(result.status, 29);
  assert.ok(result.events.includes("archive-reused"));
});

test(
  "Nuxt setup interruption terminates and reaps the owned CLI process group and cleans only owned paths",
  linux,
  () => {
    const result = probe(0, 0, 0, true);
    assert.equal(result.status, 143);
    assert.ok(!result.events.includes("install-start"));
  },
);
