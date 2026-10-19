// The one post-publish check: wait until every released npm package is
// visible at the release version, then install the CLI from npm and exercise
// it the way a user does (compile, lint, fmt, LSP).
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { collectOutput, println, run, runMain, writeStderr } from "../command_io.ts";
import { cliArgs } from "../command_runtime.ts";
import { waitForRegistryVisibility } from "../publish_npm_package/registry.ts";
import { userChecks, writeProject } from "./user_checks.ts";

/** Installs `vize@version` into a fresh project and runs what a user does first. */
async function smokeInstall(version: string): Promise<boolean> {
  const dir = mkdtempSync(join(tmpdir(), "vize-published-"));
  try {
    const npm = process.platform === "win32" ? "npm.cmd" : "npm";
    if ((await collectOutput(npm, ["init", "-y"], { cwd: dir })).status !== 0) return false;
    const install = ["install", "--no-audit", "--no-fund", `vize@${version}`];
    if ((await run(npm, install, { cwd: dir })) !== 0) return false;
    writeProject(dir);
    let ok = true;
    for (const check of userChecks(dir, version)) {
      const failure = await check.run();
      println(
        `${failure === null ? "ok  " : "FAIL"} ${check.name}${failure === null ? "" : `: ${failure}`}`,
      );
      ok &&= failure === null;
    }
    return ok;
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

export async function verifyPublished(args: readonly string[]): Promise<number> {
  const [version, ...names] = args;
  if (version === undefined || names.length === 0) {
    writeStderr("Usage: verify_published/main.ts <version> <package-name>...\n");
    return 1;
  }
  const visible = await Promise.all(
    names.map((name) => waitForRegistryVisibility({ name, version, tag: "latest" }, 180, 10)),
  );
  const missing = names.filter((_, index) => visible[index] !== 0);
  if (missing.length > 0) {
    writeStderr(`Not visible on npm at ${version}: ${missing.join(", ")}\n`);
    return 1;
  }
  println(`All ${names.length} packages are visible at ${version}.`);
  return (await smokeInstall(version)) ? 0 : 1;
}

if (import.meta.main) {
  runMain(() => verifyPublished(cliArgs()));
}
