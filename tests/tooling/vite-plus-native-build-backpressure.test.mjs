import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

const repoRoot = fileURLToPath(new URL("../../", import.meta.url));
const require = createRequire(import.meta.url);
const packageFile = require.resolve("vite-plus/package.json");
const packageRoot = path.dirname(packageFile);
const vp = path.join(packageRoot, "bin/vp");
const metadata = JSON.parse(fs.readFileSync(packageFile, "utf8"));
const catalog = parse(fs.readFileSync(path.join(repoRoot, "pnpm-workspace.yaml"), "utf8"));
const stdout = Buffer.from("native-build-output:0123456789abcdef\n".repeat(32_768));
const stderr = Buffer.from("native-build-diagnostic:fedcba9876543210\n".repeat(16_384));

async function consumeUnderBackpressure(directory, exitStatus) {
  const child = spawn(process.execPath, [vp, "run", "--no-cache", "--workspace-root", "probe"], {
    cwd: directory,
    env: { ...process.env, NO_COLOR: "1", FORCE_COLOR: "0" },
    stdio: ["ignore", "pipe", "pipe"],
  });
  const complete = new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (status, signal) => resolve({ status, signal }));
  });
  const chunks = [[], []];
  // Fill the consumer pipes first, then drain slowly enough to cross many pipe
  // buffers. This exercises the installed VP, not a mocked spawn or writer.
  const timers = new Set();
  const later = (callback, delay) => {
    const timer = setTimeout(() => {
      timers.delete(timer);
      callback();
    }, delay);
    timers.add(timer);
  };
  for (const [index, stream] of [child.stdout, child.stderr].entries()) {
    stream.pause();
    stream.on("data", (chunk) => {
      chunks[index].push(chunk);
      stream.pause();
      later(() => stream.resume(), 5);
    });
    later(() => stream.resume(), 200);
  }
  const deadline = setTimeout(() => child.kill("SIGKILL"), 30_000);
  try {
    const result = await complete;
    assert.equal(result.signal, null, "backpressure command did not finish within its deadline");
    assert.equal(result.status, exitStatus);
    return chunks.map((parts) => Buffer.concat(parts));
  } finally {
    clearTimeout(deadline);
    for (const timer of timers) clearTimeout(timer);
  }
}

void test("uncached native build execution preserves complete output and nonzero status under backpressure", async (context) => {
  assert.equal(metadata.version, catalog.catalogs["vite-stack"]["vite-plus"]);
  context.diagnostic(
    `actual installed VP ${metadata.version}; stdout ${stdout.length} bytes; stderr ${stderr.length} bytes; CLI SHA256 ${createHash(
      "sha256",
    )
      .update(fs.readFileSync(path.join(packageRoot, "dist/bin.js")))
      .digest("hex")}`,
  );
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vp-backpressure-"));
  try {
    fs.writeFileSync(
      path.join(directory, "package.json"),
      JSON.stringify({
        name: "vize-native-build-output-law",
        type: "module",
        scripts: { probe: "node probe.mjs" },
      }),
    );
    fs.writeFileSync(
      path.join(directory, "vite.config.mjs"),
      "export default { run: { cache: { scripts: true } } };\n",
    );
    for (const exitStatus of [0, 1]) {
      fs.writeFileSync(
        path.join(directory, "probe.mjs"),
        `
import fs from "node:fs";
fs.appendFileSync("invocations.txt", "${exitStatus}\\n");
await new Promise((resolve) => process.stdout.write(Buffer.from(${JSON.stringify(stdout.toString())}), resolve));
await new Promise((resolve) => process.stderr.write(Buffer.from(${JSON.stringify(stderr.toString())}), resolve));
process.exitCode = ${exitStatus};
`,
      );
      const [actualOut, actualErr] = await consumeUnderBackpressure(directory, exitStatus);
      // The pinned interleaved reporter adds only this genuine task-start line
      // and trailing LF. Compare the entire output, including that framing.
      assert.deepEqual(
        actualOut,
        Buffer.concat([
          Buffer.from("$ node probe.mjs ⊘ cache disabled\n"),
          stdout,
          Buffer.from("\n"),
        ]),
        "complete stdout was truncated or substituted",
      );
      assert.deepEqual(actualErr, stderr, "complete stderr was truncated or substituted");
      assert.doesNotMatch(
        actualErr.toString(),
        /Failed to spawn process|Resource temporarily unavailable/,
      );
    }
    assert.equal(fs.readFileSync(path.join(directory, "invocations.txt"), "utf8"), "0\n1\n");
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
