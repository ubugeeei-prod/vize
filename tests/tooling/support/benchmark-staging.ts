import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

function expandPath(path: string): string[] {
  const choice = /\{([^{}]+)\}/u.exec(path);
  if (!choice) return [path];
  return choice[1]!
    .split(",")
    .flatMap((alternative) => expandPath(path.replace(choice[0], alternative)));
}

export function stagedSnapshotFiles(run: string, files: string[]): string[] {
  const commands = run
    .replace(/\\\r?\n/g, "")
    .split("\n")
    .filter((line) => /^\s*git\s+add\s/.test(line));
  assert.equal(commands.length, 1, "snapshot step must have one git add command");
  const root = mkdtempSync(join(tmpdir(), "vize-benchmark-staging-"));
  const command = (executable: string, args: string[]) => {
    const result = spawnSync(executable, args, { cwd: root, encoding: "utf8" });
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    return result.stdout;
  };
  try {
    command("git", ["init", "--quiet"]);
    for (const file of files) {
      mkdirSync(dirname(join(root, file)), { recursive: true });
      writeFileSync(join(root, file), "benchmark fixture\n");
    }
    const paths = commands[0]!.trim().split(/\s+/u).slice(2);
    for (const path of paths)
      assert.match(path, /^[\w.,/{}-]+$/u, "expected literal staging paths");
    command("git", ["add", "--", ...paths.flatMap(expandPath)]);
    return command("git", ["ls-files", "--cached", "-z"]).split("\0").filter(Boolean).sort();
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
