import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const command = path.join(repoRoot, "tools", "commands", "levels", "incremental-equivalence.ts");

test("incremental equivalence accepts complete normal and seeded scope evidence", (context) => {
  const fakeBin = mkdtempSync(path.join(tmpdir(), "vize-incremental-command-"));
  context.after(() => rmSync(fakeBin, { recursive: true, force: true }));
  const cargo = path.join(fakeBin, "cargo");
  writeFileSync(
    cargo,
    `#!/usr/bin/env node
const seeded = process.argv.includes("seeded-stale-cache");
process.stdout.write([
  "files=3",
  "steps_applied=5",
  "comparisons=7",
  "blocks_compared=11",
  \`mismatches=\${seeded ? 2 : 0}\`,
  "",
].join("\\n"));
process.exitCode = seeded ? 1 : 0;
`,
  );
  chmodSync(cargo, 0o755);
  const env = { ...process.env, PATH: `${fakeBin}:${process.env.PATH ?? ""}` };

  const normal = spawnSync(process.execPath, [command, "--fixtures", "tests/_fixtures/_projects"], {
    cwd: repoRoot,
    env,
    encoding: "utf8",
  });
  assert.equal(normal.status, 0, normal.stderr);
  assert.match(
    normal.stdout,
    /TS-42 PASS: 3 files, 5 steps, 7 comparisons, 11 blocks, 0 mismatches/,
  );

  const seeded = spawnSync(
    process.execPath,
    [command, "--fixtures", "tests/_fixtures/_projects", "--seeded"],
    { cwd: repoRoot, env, encoding: "utf8" },
  );
  assert.equal(seeded.status, 0, seeded.stderr);
  assert.match(seeded.stdout, /TS-42 seeded stale cache: caught \(2 stale states\)/);
});

test("incremental equivalence fails closed when the harness reports no scope", (context) => {
  const fakeBin = mkdtempSync(path.join(tmpdir(), "vize-incremental-empty-"));
  context.after(() => rmSync(fakeBin, { recursive: true, force: true }));
  const cargo = path.join(fakeBin, "cargo");
  writeFileSync(cargo, "#!/bin/sh\nprintf 'mismatches=0\\n'\n");
  chmodSync(cargo, 0o755);

  const result = spawnSync(process.execPath, [command, "--fixtures", "tests/_fixtures/_projects"], {
    cwd: repoRoot,
    env: { ...process.env, PATH: `${fakeBin}:${process.env.PATH ?? ""}` },
    encoding: "utf8",
  });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /TS-42 FAIL: the run compared nothing/);
});
