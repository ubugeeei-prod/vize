import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { once } from "node:events";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const cleanupModule = pathToFileURL(path.resolve("tests/app/dev/nuxt-ui-hmr-cleanup.ts")).href;

async function verifyRestoreOnTermination(
  mode: "exit" | NodeJS.Signals,
): Promise<{ code: number | null; signal: NodeJS.Signals | null }> {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-hmr-cleanup-"));
  const sourcePath = path.join(root, "Button.vue");
  fs.writeFileSync(sourcePath, "updated");
  const script = `
    import { installSourceRestore } from ${JSON.stringify(cleanupModule)};
    installSourceRestore(${JSON.stringify(sourcePath)}, "original");
    process.stdout.write("ready\\n");
    ${mode === "exit" ? "setTimeout(() => process.exit(0), 50);" : "setInterval(() => {}, 1000);"}
  `;
  const child = spawn(process.execPath, ["--input-type=module", "--eval", script], {
    stdio: ["ignore", "pipe", "inherit"],
  });

  try {
    await once(child.stdout!, "data");
    if (mode !== "exit") child.kill(mode);
    const [code, signal] = (await once(child, "exit")) as [number | null, NodeJS.Signals | null];
    assert.equal(fs.readFileSync(sourcePath, "utf8"), "original");
    return { code, signal };
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
}

test("Nuxt HMR source cleanup restores on normal process exit", async () => {
  assert.deepEqual(await verifyRestoreOnTermination("exit"), { code: 0, signal: null });
});

for (const signal of ["SIGINT", "SIGTERM"] as const) {
  test(`Nuxt HMR source cleanup restores and re-raises ${signal}`, async () => {
    assert.deepEqual(await verifyRestoreOnTermination(signal), { code: null, signal });
  });
}
