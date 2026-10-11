import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const root = fileURLToPath(new URL("../../", import.meta.url));

for (const options of [[], ["-O"]]) {
  void test(`complete artifact payload controls with Python options ${options.join(" ") || "normal"}`, () => {
    const result = spawnSync(
      "python3",
      [
        ...options,
        "-m",
        "unittest",
        "discover",
        "-s",
        "tools/support/compat/github",
        "-p",
        "artifact_payload_test.py",
      ],
      { cwd: root, encoding: "utf8", timeout: 30_000 },
    );
    assert.equal(result.error, undefined, result.error?.message);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.match(result.stderr, /Ran 8 tests/);
  });
}
