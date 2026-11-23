import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
const dir = process.argv[2];
const name =
  "content_mapper_tsgo_cli::recovered_attribute_boundary_11731::legal_quoted_boundaries_keep_neighbor_and_unused_diagnostics";
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const stdout = fs.readFileSync(path.join(dir, "test.stdout"));
const stderr = fs.readFileSync(path.join(dir, "test.stderr"));
const status = fs.readFileSync(path.join(dir, "test.status"), "utf8");
const wanted = new Set(
  ["reported", "valid", "same-file", "unused"].flatMap((variant) =>
    ["\n", "\r\n"].flatMap((newline) =>
      [false, true].map((unused) => JSON.stringify([variant, newline, unused])),
    ),
  ),
);
const seen = new Set();
let failure: string | null = null;
let provider = null;
try {
  assert.equal(status, "0\n");
  assert.equal(
    stdout
      .toString()
      .split("\n")
      .filter((line) => line === `test ${name} ... ok`).length,
    1,
  );
  assert.equal(
    [...stdout.toString().matchAll(/^test result: ok\. 1 passed; 0 failed; 0 ignored; /gm)].length,
    1,
  );
  const selected = process.env.VIZE_TEST_RECOVERED_ATTRIBUTE_CONTENT_MAPPER_TSGO;
  assert.ok(selected, "selected native provider is required");
  const program = fs.realpathSync(selected);
  const manifest = JSON.parse(
    fs.readFileSync(path.join(path.dirname(path.dirname(program)), "package.json"), "utf8"),
  );
  assert.equal(manifest.name, "@typescript/typescript-linux-x64");
  assert.equal(manifest.version, "7.1.0-beta");
  const sha256 = digest(fs.readFileSync(program));
  assert.equal(sha256, "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70");
  provider = { program, sha256, package: manifest.name, version: manifest.version };
  for (const line of stderr
    .toString()
    .split("\n")
    .filter((line) => line.startsWith("11731 cell "))) {
    const row =
      /^11731 cell variant=(\S+) newline=("(?:\\.|[^"])*") noUnusedLocals=(false|true) provider=(.+) status=exit status: (\d+) stdout=/.exec(
        line,
      );
    assert.ok(row, line);
    const key = JSON.stringify([row[1], JSON.parse(row[2]), row[3] === "true"]);
    assert.ok(wanted.has(key) && !seen.has(key), key);
    assert.equal(row[4], process.env.VIZE_TEST_RECOVERED_ATTRIBUTE_CONTENT_MAPPER_TSGO);
    assert.equal(Number(row[5]), row[1] === "valid" ? 0 : 2);
    seen.add(key);
  }
  assert.deepEqual(seen, wanted);
} catch (error) {
  failure = String(error);
}
fs.writeFileSync(
  path.join(dir, "test-run.json"),
  JSON.stringify(
    {
      source: execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
      githubSha: process.env.GITHUB_SHA,
      run: process.env.GITHUB_RUN_ID,
      attempt: process.env.GITHUB_RUN_ATTEMPT,
      name,
      status,
      cellCount: seen.size,
      inventory: [...seen],
      provider,
      stdoutSha256: digest(stdout),
      stderrSha256: digest(stderr),
      qualified: failure === null,
      failure,
    },
    null,
    2,
  ) + "\n",
);
if (failure !== null) throw new Error(failure);
