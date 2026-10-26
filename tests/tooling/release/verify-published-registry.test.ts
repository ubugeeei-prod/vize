import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { missingFromRegistry } from "../../../tools/support/release/verify_published/main.ts";
import { writeFakeCommand } from "../support/fake-command.ts";
import { fakeNpmArchiveSource } from "../support/fake-npm-registry.ts";

// Serves the exact version with its archive; `latest` comes from TEST_LATEST.
function fakeRegistry(bin: string): void {
  writeFakeCommand(
    bin,
    "curl",
    [
      fakeNpmArchiveSource,
      "const url = new URL(process.argv.at(-1));",
      "const [, encoded, version] = url.pathname.split('/');",
      "const name = decodeURIComponent(encoded);",
      "if (version !== undefined && version !== process.env.TEST_PUBLISHED) {",
      "  process.stdout.write('{}VIZE_HTTP_STATUS:404');",
      "  process.exit(0);",
      "}",
      "const body = version === undefined",
      "  ? { name, 'dist-tags': { latest: process.env.TEST_LATEST } }",
      "  : { name, version, dist: fakeDist(name, version) };",
      "process.stdout.write(`${JSON.stringify(body)}VIZE_HTTP_STATUS:200`);",
    ].join("\n"),
  );
}

async function missing(published: string, latest: string): Promise<string[]> {
  const bin = mkdtempSync(path.join(tmpdir(), "vize-verify-published-"));
  const saved = { ...process.env };
  try {
    fakeRegistry(bin);
    process.env.PATH = `${bin}${path.delimiter}${process.env.PATH ?? ""}`;
    process.env.TEST_PUBLISHED = published;
    process.env.TEST_LATEST = latest;
    return await missingFromRegistry("0.580.0", ["vize", "@vizejs/native"], 1, 0);
  } finally {
    process.env = saved;
    rmSync(bin, { recursive: true, force: true });
  }
}

test("verification accepts a later release's latest tag", async () => {
  assert.deepEqual(await missing("0.580.0", "0.580.0"), []);
  // The next release can publish while this one still waits for propagation,
  // and an npm resume of an older version publishes under its backfill tag
  // while `latest` stays on the newer release, exactly as publish-npm checks.
  assert.deepEqual(await missing("0.580.0", "0.581.0"), []);
  assert.deepEqual(await missing("0.580.0", "0.582.0"), []);
});

test("verification still requires the exact version and a current latest tag", async () => {
  assert.deepEqual(await missing("0.579.0", "0.581.0"), ["vize", "@vizejs/native"]);
  assert.deepEqual(await missing("0.580.0", "0.579.0"), ["vize", "@vizejs/native"]);
  assert.deepEqual(await missing("0.580.0", "0.581.0-beta.1"), ["vize", "@vizejs/native"]);
});
