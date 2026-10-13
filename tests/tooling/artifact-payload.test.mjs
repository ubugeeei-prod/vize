import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdtempSync, mkdirSync, rmSync } from "node:fs";
import { devNull, tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const root = fileURLToPath(new URL("../../", import.meta.url));
const pythonArgs = (options, args) => [...options, "-B", ...args];

for (const options of [[], ["-O"]]) {
  void test(`complete artifact payload controls with Python options ${options.join(" ") || "normal"}`, () => {
    const result = spawnSync(
      "python3",
      pythonArgs(options, [
        "-m",
        "unittest",
        "discover",
        "-s",
        "tools/support/compat/github",
        "-p",
        "artifact_payload_test.py",
      ]),
      { cwd: root, encoding: "utf8", timeout: 30_000 },
    );
    assert.equal(result.error, undefined, result.error?.message);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.match(result.stderr, /Ran 8 tests/);
  });
}

void test("imported payload helpers preserve the real clean-source inventory", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-payload-source-write-"));
  try {
    const source = join(directory, "source");
    mkdirSync(source);
    for (const name of ["artifact_payload.py", "artifact_payload_files.py"]) {
      copyFileSync(join(root, "tools/support/compat/github", name), join(source, name));
    }
    const git = (args) => {
      const result = spawnSync("git", ["-c", `core.excludesFile=${devNull}`, ...args], {
        cwd: directory,
        encoding: "utf8",
      });
      assert.equal(result.status, 0, result.stderr);
      return result.stdout;
    };
    git(["init", "--quiet"]);
    git(["add", "source"]);
    git([
      "-c",
      "user.name=Payload fixture",
      "-c",
      "user.email=fixture@example.invalid",
      "-c",
      "commit.gpgsign=false",
      "-c",
      `core.hooksPath=${join(directory, "disabled-hooks")}`,
      "commit",
      "--quiet",
      "--message",
      "Track imported source fixture",
    ]);
    const status = () => git(["status", "--porcelain", "--untracked-files=all"]);
    const before = status();
    assert.equal(before, "");
    const run = (args) => {
      const result = spawnSync("python3", args, {
        cwd: source,
        encoding: "utf8",
        env: { ...process.env, PYTHONDONTWRITEBYTECODE: "" },
      });
      assert.equal(result.status, 0, result.stderr);
    };
    // Exercise the Linux runner's source-local cache policy on hosts with a global cache prefix.
    const importSource = "import sys; sys.pycache_prefix = None; import artifact_payload";
    for (const options of [[], ["-O"]]) {
      run(pythonArgs(options, ["-c", importSource]));
      assert.equal(status(), before);
      assert.equal(existsSync(join(source, "__pycache__")), false);
    }
    run(["-c", importSource]);
    assert.notEqual(status(), before);
    assert.match(status(), /\?\? source\/__pycache__\/artifact_payload/);
    assert.equal(existsSync(join(source, "__pycache__")), true);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
