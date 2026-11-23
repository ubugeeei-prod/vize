import { readToolingProgram } from "./support/program-path.ts";
import { readAutomationPrograms } from "./support/automation-source.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { readRepoFile } from "./support/github-workflows.ts";

function retryWrapperSource(): string {
  const action = readAutomationPrograms(".github", "actions", "setup-ubuntu-archive", "action.yml");
  assert.match(
    action,
    /sudo tee \/usr\/local\/bin\/vize-ci-apt-retry[^\n]*scripts\/retry-apt\.sh/u,
  );
  return readRepoFile(".github/actions/setup-ubuntu-archive/scripts/retry-apt.sh");
}

test("apt retry wrapper preserves observable retry behavior", () => {
  const dir = mkdtempSync(path.join(tmpdir(), "vize-apt-retry-"));
  try {
    const bin = path.join(dir, "bin");
    const wrapper = path.join(bin, "vize-ci-apt-retry");
    const sudo = path.join(bin, "sudo");
    const probe = path.join(dir, "probe.sh");
    mkdirSync(bin);
    writeFileSync(wrapper, retryWrapperSource(), { mode: 0o755 });
    writeFileSync(sudo, readToolingProgram("commands/github-apt-retry-wrapper/exit-0.sh"), {
      mode: 0o755,
    });
    writeFileSync(probe, readToolingProgram("commands/github-apt-retry-wrapper/probe.sh"), {
      mode: 0o755,
    });

    const run = (retries: string, passAfter: string, status: string) => {
      const countFile = path.join(dir, `count-${retries}-${passAfter}-${status}`);
      const result = spawnSync(wrapper, [probe, countFile, passAfter, status], {
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${bin}:${process.env.PATH}`,
          VIZE_CI_APT_RETRIES: retries,
          VIZE_CI_APT_RETRY_DELAY_SECONDS: "0",
        },
      });
      return { count: Number(readFileSync(countFile, "utf8")), status: result.status };
    };

    assert.deepEqual(run("08", "8", "42"), { count: 8, status: 0 });
    assert.deepEqual(run("0", "99", "7"), { count: 1, status: 7 });
    assert.deepEqual(run("2", "99", "23"), { count: 2, status: 23 });
  } finally {
    rmSync(dir, { force: true, recursive: true });
  }
});
