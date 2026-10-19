// The one post-publish check: wait until every released npm package is
// visible at the release version, then install the CLI from npm and run it.
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { collectOutput, println, run, runMain, writeStderr } from "../command_io.ts";
import { cliArgs } from "../command_runtime.ts";
import { waitForRegistryVisibility } from "../publish_npm_package/registry.ts";

const SFC = "<script setup>const n = 1</script><template><p>{{ n }}</p></template>";

/** Installs `vize@version` into a fresh project and exercises the binary. */
async function smokeInstall(version: string): Promise<boolean> {
  const dir = mkdtempSync(join(tmpdir(), "vize-published-"));
  try {
    const npm = process.platform === "win32" ? "npm.cmd" : "npm";
    if ((await collectOutput(npm, ["init", "-y"], { cwd: dir })).status !== 0) return false;
    const install = ["install", "--no-audit", "--no-fund", `vize@${version}`];
    if ((await run(npm, install, { cwd: dir })) !== 0) return false;
    const cli = await collectOutput(process.execPath, [
      join(dir, "node_modules/vize/bin/vize"),
      "--version",
    ]);
    println(cli.stdout.trim());
    if (cli.status !== 0 || !cli.stdout.includes(version)) {
      writeStderr(`vize --version did not report ${version}\n${cli.stderr}`);
      return false;
    }
    const compile = `const { compileSfc } = require(require.resolve("@vizejs/native", { paths: [${JSON.stringify(dir)}] }));
      const out = compileSfc(${JSON.stringify(SFC)}, { filename: "Smoke.vue" });
      if (!JSON.stringify(out).includes("createElementBlock")) throw new Error("unexpected compile output");`;
    return (await run(process.execPath, ["-e", compile])) === 0;
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
    names.map((name) => waitForRegistryVisibility({ name, version, tag: "latest" }, 90, 10)),
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
