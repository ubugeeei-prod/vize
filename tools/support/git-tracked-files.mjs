import { spawnSync } from "node:child_process";

// The full tracked filename stream exceeds Node's default 1 MiB buffer.
// A bounded overflow remains an error: never inspect a truncated inventory.
export function gitTrackedFiles(cwd, { maxBuffer = 8 * 1024 * 1024 } = {}) {
  const listed = spawnSync("git", ["ls-files", "-z"], {
    cwd,
    encoding: "utf8",
    maxBuffer,
  });
  if (listed.error || listed.status !== 0) {
    const detail = listed.error?.message ?? listed.stderr;
    throw new Error(
      `git ls-files failed (status=${listed.status}, signal=${listed.signal}): ${detail}`,
      { cause: listed.error },
    );
  }
  return listed.stdout.split("\0").filter(Boolean);
}
