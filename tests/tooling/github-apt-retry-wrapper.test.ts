import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { readRepoFile } from "./support/github-workflows.ts";

function retryWrapperSource(): string {
  const action = readRepoFile(".github", "actions", "setup-ubuntu-archive", "action.yml");
  const start = action.indexOf("sudo tee /usr/local/bin/vize-ci-apt-retry >/dev/null <<'BASH'\n");
  assert.notEqual(start, -1);
  const bodyStart = action.indexOf("\n", start) + 1;
  const bodyEnd = action.indexOf("\n        BASH", bodyStart);
  assert.notEqual(bodyEnd, -1);
  return action
    .slice(bodyStart, bodyEnd)
    .split("\n")
    .map((line) => line.replace(/^        /, ""))
    .join("\n");
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
    writeFileSync(sudo, "#!/usr/bin/env bash\nexit 0\n", { mode: 0o755 });
    writeFileSync(
      probe,
      [
        "#!/usr/bin/env bash",
        'count="$(cat "$1" 2>/dev/null || true)"',
        'count="$((10#${count:-0} + 1))"',
        'printf "%s" "$count" > "$1"',
        "if (( count >= 10#$2 )); then exit 0; fi",
        'exit "$3"',
      ].join("\n"),
      { mode: 0o755 },
    );

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
