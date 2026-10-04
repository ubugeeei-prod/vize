/** Buffered-pipe regression: preserve >1 MiB on both streams and nonzero status. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmodSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

test("native forwarding drains stdout/stderr pipes and preserves nonzero status", () => {
  const dir = mkdtempSync(join(os.tmpdir(), "native-phase-forwarding-"));
  try {
    const runtime = join(dir, "mock-native.mjs");
    const member = join(dir, "member.ts");
    const config = join(dir, "tsconfig.json");
    const settings = join(dir, "settings.json");
    writeFileSync(member, "export const value: number = 1;\n");
    writeFileSync(config, JSON.stringify({ compilerOptions: { noEmit: true }, files: [member] }));
    writeFileSync(settings, JSON.stringify({ runtime, directory: join(dir, "captured") }));
    const stdout = "member.ts(1,1): error TS2322: phase-雪\n".repeat(40_000);
    const stderr = "native-stderr-🌸\n".repeat(80_000);
    const footer =
      "Files: 1\nLines: 1\nIdentifiers: 1\nSymbols: 1\nTypes: 1\nInstantiations: 0\nMemory used: 1K\nMemory allocs: 1\nConfig time: 0.001s\nParse time: 0.001s\nBind time: 0.001s\nCheck time: 0.001s\nEmit time: 0.000s\nTotal time: 0.004s\n";
    assert(Buffer.byteLength(stdout) > 1024 * 1024 && Buffer.byteLength(stderr) > 1024 * 1024);
    // This mock is a transport control, not native phase/performance evidence.
    writeFileSync(
      runtime,
      `#!/usr/bin/env node
const args = process.argv.slice(2);
if (args.includes("--listFilesOnly")) {
  process.stdout.write(${JSON.stringify(`${member}\n`)});
} else {
  process.stdout.write(${JSON.stringify(stdout)});
  if (args.includes("--extendedDiagnostics")) process.stdout.write(${JSON.stringify(footer)});
  process.stderr.write(${JSON.stringify(stderr)});
  process.exitCode = 3;
}
`,
    );
    chmodSync(runtime, 0o755);
    const wrapper = fileURLToPath(
      new URL("./typechecker-native-phase-capture.mjs", import.meta.url),
    );
    const hash = (value) => createHash("sha256").update(value).digest("hex");
    for (const args of [["--project", config, "--checkers", "1"], ["--version"]]) {
      const result = spawnSync(process.execPath, [wrapper, ...args], {
        cwd: dir,
        env: { ...process.env, NATIVE_PHASE_CONFIG: settings },
        timeout: 60_000,
        maxBuffer: 32 * 1024 * 1024,
      });
      assert.equal(result.error, undefined, result.error?.message);
      assert.equal(result.signal, null);
      assert.equal(result.status, 3, "native nonzero status was lost");
      assert.equal(result.stdout.length, Buffer.byteLength(stdout));
      assert.equal(result.stderr.length, Buffer.byteLength(stderr));
      assert.equal(hash(result.stdout), hash(stdout), "stdout bytes were truncated or altered");
      assert.equal(hash(result.stderr), hash(stderr), "stderr bytes were truncated or altered");
    }
    const receipts = readdirSync(join(dir, "captured")).filter((file) => /^\d+\.json$/u.test(file));
    assert.equal(receipts.length, 1);
    const receipt = JSON.parse(readFileSync(join(dir, "captured", receipts[0]), "utf8"));
    assert.equal(receipt.runs.baseline.status, 3);
    assert.equal(receipt.nativeGraphBytesUnchanged, true);
    assert.equal(hash(receipt.phases.diagnosticText), hash(stdout));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
