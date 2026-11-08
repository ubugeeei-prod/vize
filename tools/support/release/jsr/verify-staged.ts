import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { jsrPackageDirectory, jsrPackages } from "./packages.ts";

export function verifyStagedPackages(stagingRoot: string): void {
  for (const descriptor of jsrPackages) {
    const directory = resolve(stagingRoot, jsrPackageDirectory(descriptor.jsrName));
    console.log(`Checking ${descriptor.jsrName}`);
    const result = spawnSync("deno", ["publish", "--dry-run", "--config", "jsr.json"], {
      cwd: directory,
      stdio: "inherit",
    });
    if (result.error) throw result.error;
    if (result.signal) throw new Error(`${descriptor.jsrName} dry-run received ${result.signal}`);
    if (result.status !== 0) {
      throw new Error(`${descriptor.jsrName} dry-run exited with ${result.status}`);
    }
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [command, directory] = process.argv.slice(2);
  if (command !== "dry-run" || !directory) {
    throw new Error("Usage: verify-staged.ts dry-run <staging-root>");
  }
  verifyStagedPackages(directory);
}
