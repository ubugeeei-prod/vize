import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const driver = fileURLToPath(
  new URL("../../tools/benchmarks/scripts/string-storage.ts", import.meta.url),
);

test("benchmark preparation validates both checkouts before overlaying the candidate harness", (t) => {
  const cwd = mkdtempSync(path.join(tmpdir(), "vize-storage-harness-"));
  t.after(() => rmSync(cwd, { recursive: true, force: true }));
  const git = (dir: string, ...args: string[]) =>
    execFileSync(
      "git",
      [
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "-c",
        "commit.gpgsign=false",
        ...args,
      ],
      { cwd: dir, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    ).trim();
  const head = path.join(cwd, "head");
  const base = path.join(cwd, "base");
  const crate = "crates/vize_croquis_cf";
  const benches = ["cross_file.rs", "string_storage.rs"];
  mkdirSync(path.join(head, crate, "benches"), { recursive: true });
  git(head, "init", "-q");
  for (const name of benches) writeFileSync(path.join(head, crate, "benches", name), "before");
  writeFileSync(path.join(head, crate, "Cargo.toml"), "[dev-dependencies]\n");
  git(head, "add", ".");
  git(head, "commit", "-qm", "base");
  const baseSha = git(head, "rev-parse", "HEAD");
  git(cwd, "clone", "-q", "--no-hardlinks", head, base);
  for (const name of benches) writeFileSync(path.join(head, crate, "benches", name), "after");
  git(head, "commit", "-qam", "head");
  const headSha = git(head, "rev-parse", "HEAD");
  const prepare = (baseRef: string, dispatch: string) =>
    spawnSync(process.execPath, [driver, "prepare"], {
      cwd,
      env: { ...process.env, BASE_SHA: baseRef, HEAD_SHA: headSha, DISPATCH_SHA: dispatch },
    });
  assert.notEqual(prepare(baseSha, baseSha).status, 0);
  assert.notEqual(prepare(headSha, headSha).status, 0);
  assert.ok(!existsSync(path.join(cwd, "reports")));
  const result = prepare(baseSha, headSha);
  assert.equal(result.status, 0, result.stderr.toString());
  for (const name of benches)
    assert.equal(readFileSync(path.join(base, crate, "benches", name), "utf8"), "after");
  assert.equal(
    readFileSync(path.join(base, crate, "Cargo.toml"), "utf8"),
    "[dev-dependencies]\ndavinci_harness.workspace = true\n",
  );
  assert.equal(readFileSync(path.join(cwd, "reports", "base-sha.txt"), "utf8"), baseSha + "\n");
  assert.equal(readFileSync(path.join(cwd, "reports", "head-sha.txt"), "utf8"), headSha + "\n");
});
