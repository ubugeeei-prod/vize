import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

await test("patched production dependencies retain genuine Nuxt Git RPC and safe Vue SSR", () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-dependency-security-"));
  const env = { ...process.env, NODE_ENV: "development", NUXT_TELEMETRY_DISABLED: "1" };
  delete env.TEST;
  delete env.VITEST;
  try {
    const child = spawnSync(
      process.execPath,
      [
        fileURLToPath(new URL("./support/dependency-security-compat.mjs", import.meta.url)),
        fixture,
      ],
      {
        env,
        encoding: "utf8",
        timeout: 180_000,
        maxBuffer: 8 * 1024 * 1024,
      },
    );
    console.log(
      JSON.stringify({
        status: child.status,
        signal: child.signal,
        stdout: child.stdout,
        stderr: child.stderr,
      }),
    );
    assert.ifError(child.error);
    assert.equal(child.signal, null);
    assert.equal(child.status, 0, `${child.stdout}\n${child.stderr}`);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});
