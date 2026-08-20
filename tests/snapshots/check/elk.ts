import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { execFileSync, execSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { elkApp, CORSA_BIN, VIZE_BIN, requireVizeAndCorsaBins } from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const app = elkApp;
let checkCwd = app.check!.cwd;

function syncCleanCheckFixture(sourceDir: string): string {
  const worktreeId = process.env.VIZE_TEST_WORKTREE_ID ?? `pid-${process.pid}`;
  const workDir = path.resolve(
    __dirname,
    "../../_fixtures/_projects/_git-worktrees",
    worktreeId,
    "check-clean",
    app.name,
  );
  const parentDir = path.dirname(workDir);
  fs.mkdirSync(parentDir, { recursive: true });

  const stagingDir = fs.mkdtempSync(path.join(parentDir, `${app.name}-staging-`));
  const archive = execFileSync("git", ["archive", "--format=tar", "HEAD"], {
    cwd: sourceDir,
    encoding: "buffer",
    maxBuffer: 200 * 1024 * 1024,
  });
  execFileSync("tar", ["-xf", "-"], {
    cwd: stagingDir,
    input: archive,
    maxBuffer: 200 * 1024 * 1024,
  });

  try {
    fs.rmSync(workDir, { recursive: true, force: true });
    fs.renameSync(stagingDir, workDir);
  } finally {
    fs.rmSync(stagingDir, { recursive: true, force: true });
  }

  return workDir;
}

describe(`${app.name} check (type checker)`, () => {
  before(() => {
    requireVizeAndCorsaBins();
    checkCwd = syncCleanCheckFixture(app.check!.cwd);
  });

  it("vize check does not crash and snapshot matches", () => {
    const checkConfig = app.check!;
    const patterns = checkConfig.patterns.map((p) => `'${p}'`).join(" ");
    const cmd = `${VIZE_BIN} check ${patterns} --format json --quiet --corsa-path '${CORSA_BIN}'`;
    console.log(`Running: ${cmd}`);

    let stdout: string;
    try {
      stdout = execSync(cmd, {
        cwd: checkCwd,
        timeout: 120_000,
        maxBuffer: 100 * 1024 * 1024,
      }).toString();
    } catch (e: any) {
      if (e.status === 1 && e.stdout) {
        stdout = e.stdout.toString();
      } else {
        throw new Error(`vize check crashed (exit code ${e.status}): ${e.stderr?.toString()}`);
      }
    }

    const parsed = JSON.parse(stdout);
    console.log(`fileCount=${parsed.fileCount}, errorCount=${parsed.errorCount}`);
    assert.ok(parsed.fileCount > 0, "fileCount should be > 0");

    const prettyOutput = JSON.stringify(parsed, null, 2).replaceAll(checkCwd, "<cwd>") + "\n";
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
  });
});
