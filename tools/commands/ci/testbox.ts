import { spawnSync } from "node:child_process";
import { accessSync, constants } from "node:fs";

const pushCommand = 'git push --set-upstream origin "$(git branch --show-current)"';
const pushHint = "Push it first: " + pushCommand;

function diagnostic(message: string, code = 2): number {
  console.error(message);
  return code;
}

function warmup(binary: string): number {
  const git = (...args: string[]) =>
    spawnSync("git", args, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
  const branch = git("branch", "--show-current").stdout?.trim();
  if (!branch)
    return diagnostic("Testbox warmup needs a named branch. Check one out, push it, and retry.");
  const remote = git("ls-remote", "--heads", "origin", "refs/heads/" + branch);
  if (remote.status !== 0)
    return diagnostic(
      "Testbox warmup could not query origin. Check the Git remote, network access, and Git credentials, then retry.",
    );
  if (!remote.stdout.trim())
    return diagnostic("Testbox warmup cannot find the current branch on origin.\n" + pushHint);
  if (git("rev-parse", "HEAD").stdout?.trim() !== remote.stdout.split(/\s/u)[0])
    return diagnostic("Testbox warmup needs the current HEAD on origin.\n" + pushHint);
  const result = spawnSync(
    binary,
    ["testbox", "warmup", ".github/workflows/e2e.yml", "--ref", branch, "--job", "testbox"],
    { stdio: "inherit" },
  );
  return result.status === 0
    ? 0
    : diagnostic(
        'Blacksmith warmup failed. Authenticate with "$VIZE_BLACKSMITH_BIN" auth login, verify repository access, and retry.',
        1,
      );
}

function main([operation, taskName]: string[]): number {
  if (process.env.VIZE_TESTBOX_SHELL !== "1")
    return diagnostic(
      "Testbox tasks require the pinned Blacksmith environment.\nEnter the dedicated shell first: nix develop .#testbox",
    );
  const binary = process.env.VIZE_BLACKSMITH_BIN ?? "";
  try {
    accessSync(binary, constants.X_OK);
  } catch {
    return diagnostic(
      "Pinned Blacksmith CLI is unavailable inside the Testbox shell. Exit and re-enter: nix develop .#testbox",
    );
  }
  if (operation === "warmup") return warmup(binary);
  const id = process.env.BLACKSMITH_TESTBOX_ID;
  if (!id)
    return diagnostic(
      [
        "BLACKSMITH_TESTBOX_ID is unset.",
        "Use the guarded lifecycle in .github/CONTRIBUTING.md. It clears stale ids before warmup:",
        "  unset BLACKSMITH_TESTBOX_ID",
        '  "$VIZE_BLACKSMITH_BIN" auth login',
        "  " + pushCommand,
        "  vp run --workspace-root testbox:warmup",
      ].join("\n"),
    );
  if (
    operation !== "stop" &&
    (operation !== "run" || !taskName || !["build", "lint", "test"].includes(taskName))
  )
    return diagnostic("Usage: testbox warmup | stop | run <build|lint|test>");
  const args = ["testbox", operation, "--id", id];
  if (operation === "run") args.push("vp run --workspace-root " + taskName);
  return spawnSync(binary, args, { stdio: "inherit" }).status ?? 1;
}
process.exitCode = main(process.argv.slice(2));
