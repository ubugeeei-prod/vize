import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { once } from "node:events";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { installSourceRestores } from "../app/dev/source-restore.ts";

const cleanupModule = pathToFileURL(path.resolve("tests/app/dev/source-restore.ts")).href;

async function verifyRestoreOnTermination(
  mode: "exit" | NodeJS.Signals,
): Promise<{ code: number | null; signal: NodeJS.Signals | null }> {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-hmr-source-restore-"));
  const sourcePaths = [path.join(root, "Button.vue"), path.join(root, "Child.vue")];
  const createdPath = path.join(root, "ExternalTemplate.html");
  for (const sourcePath of sourcePaths) fs.writeFileSync(sourcePath, "updated");
  fs.writeFileSync(createdPath, "updated");
  const restoreEntries = [
    ...sourcePaths.map((sourcePath) => ({ sourcePath, originalSource: "original" })),
    { sourcePath: createdPath, originalSource: null },
  ];
  const script = `
    import { installSourceRestores } from ${JSON.stringify(cleanupModule)};
    installSourceRestores(${JSON.stringify(restoreEntries)});
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
    for (const sourcePath of sourcePaths) {
      assert.equal(fs.readFileSync(sourcePath, "utf8"), "original");
    }
    assert.equal(fs.existsSync(createdPath), false);
    return { code, signal };
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
}

test("HMR source cleanup rejects empty and duplicate restore plans", () => {
  assert.throws(() => installSourceRestores([]), /at least one file/);
  assert.throws(
    () =>
      installSourceRestores([
        { sourcePath: "/tmp/App.vue", originalSource: "first" },
        { sourcePath: "/tmp/App.vue", originalSource: "second" },
      ]),
    /paths must be unique/,
  );
});

test("HMR source cleanup restores on normal process exit", async () => {
  assert.deepEqual(await verifyRestoreOnTermination("exit"), { code: 0, signal: null });
});

for (const signal of ["SIGINT", "SIGTERM"] as const) {
  test(`HMR source cleanup restores and re-raises ${signal}`, async () => {
    assert.deepEqual(await verifyRestoreOnTermination(signal), { code: null, signal });
  });
}
