import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { repoRoot } from "./_helpers/moonbit.ts";

const command = path.join(repoRoot, "tools/commands/ci/source-file-lengths.rs");

function git(cwd: string, ...args: string[]) {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

test("only canonical released contract artifacts are exempt from source length", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-artifacts-"));
  try {
    git(cwd, "init", "-q");
    fs.writeFileSync(path.join(cwd, "README.md"), "base\n");
    git(cwd, "add", "README.md");
    git(cwd, "-c", "user.name=Vize", "-c", "user.email=vize@example.com", "commit", "-qm", "base");
    const base = git(cwd, "rev-parse", "HEAD");
    const artifact = "davinci/vize_guest/versions/vize-contracts@0.1.3.json";
    const text = `${Array.from({ length: 351 }, () => "line").join("\n")}\n`;
    const write = (relative: string) => {
      const file = path.join(cwd, relative);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, text);
      git(cwd, "add", relative);
    };
    const run = () =>
      spawnSync("rust-script", [command, "--check", "--base-ref", base, "--limit", "0"], {
        cwd,
        encoding: "utf8",
      });
    write(artifact);
    assert.equal(run().status, 0);

    const controls = [
      "crates/other/versions/vize-contracts@0.1.3.json",
      "davinci/vize_guest/versions/authored.rs",
      "davinci/vize_guest/versions/other.json",
      "davinci/vize_guest/versions/vize-contracts@nested/part.json",
    ];
    controls.forEach(write);
    const result = run();
    assert.equal(result.status, 1, result.stderr);
    const failures = result.stdout.split("Files requiring action:\n").at(1)?.trim();
    assert.equal(
      failures,
      [
        "| Lines | Base | Reason | Path |",
        "| ---: | ---: | --- | --- |",
        ...controls.toSorted().map((file) => `| 351 | - | new file exceeds limit | \`${file}\` |`),
      ].join("\n"),
    );
  } finally {
    fs.rmSync(cwd, { recursive: true, force: true });
  }
});
