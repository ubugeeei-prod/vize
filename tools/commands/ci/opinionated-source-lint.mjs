import { spawnSync } from "node:child_process";
import {
  isOpinionatedSourceInput,
  opinionatedSourceRoots,
} from "../../support/github/opinionated-source-inputs.ts";

const tracked = spawnSync("git", ["ls-files", "-z", "--", ...opinionatedSourceRoots], {
  encoding: "utf8",
  maxBuffer: 16 * 1024 * 1024,
});
if (tracked.error || tracked.status !== 0) {
  console.error("Could not list tracked source files", tracked.error ?? tracked.stderr);
  process.exit(1);
}

const files = tracked.stdout.split("\0").filter(isOpinionatedSourceInput);

if (process.argv.includes("--list-files")) {
  console.log(files.join("\n"));
} else if (files.length === 0) {
  console.error("No authored source files matched the opinionated lint scope");
  process.exitCode = 1;
} else {
  console.log(`Checking ${files.length} authored source files with the current vize CLI`);
  const lint = spawnSync(
    process.env.VIZE_BIN ?? "vize",
    ["lint", "--no-config", "--preset", "opinionated", "--max-warnings", "0", ...files],
    { stdio: "inherit" },
  );
  if (lint.error) {
    console.error("Could not run vize lint", lint.error);
  }
  process.exitCode = lint.status ?? 1;
}
