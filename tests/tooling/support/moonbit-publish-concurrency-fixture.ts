import assert from "node:assert/strict";
import fs, { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { repoRoot } from "../_helpers/moonbit.ts";
import { writeFakeCommand } from "./fake-command.ts";

type PublishResult = { package_dir: string; exit_code: number; walltime_ms: number };
type PublishReceipt = {
  schema_version: number;
  concurrency: number;
  package_count: number;
  walltime_ms: number;
  exit_code: number;
  results: PublishResult[];
};
type Event = { phase: "start" | "end"; package: string; time: number; args: string[] };

// The parent is the actual native MoonBit command. Only its single-package
// publisher subprocess is mocked, with a deterministic registry-wait-like delay.
export function controlledPublishFixture(packageCount = 8) {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-concurrency-"));
  const baseDir = path.join(tempDir, "packages with ' spaces");
  const binDir = path.join(tempDir, "bin");
  const eventLog = path.join(tempDir, "events.jsonl");
  const receiptPath = path.join(tempDir, "receipt.json");
  fs.mkdirSync(binDir, { recursive: true });
  for (let i = 0; i < packageCount; i++) {
    const dir = path.join(baseDir, `pkg-${i}`);
    fs.mkdirSync(dir, { recursive: true });
    writeFileSync(
      path.join(dir, "package.json"),
      JSON.stringify({ name: `@test/pkg-${i}`, version: "1.0.0" }),
    );
  }
  fs.mkdirSync(path.join(baseDir, "skip-helper"));
  writeFileSync(path.join(baseDir, "skip-file"), "not a directory");
  const realMoon = process.env.MOON_BIN ?? path.join(repoRoot, ".cache", "moonbit", "bin", "moon");
  writeFakeCommand(
    binDir,
    "moon",
    [
      "const fs = require('node:fs');",
      "const path = require('node:path');",
      "const { spawnSync } = require('node:child_process');",
      "const args = process.argv.slice(2);",
      "const separator = args.indexOf('--');",
      "if (args[separator - 1]?.replaceAll('\\\\', '/').endsWith('/publish_npm_package')) {",
      "  const packageName = path.basename(args[separator + 1]);",
      "  const record = phase => fs.appendFileSync(process.env.MOCK_PUBLISH_EVENTS, JSON.stringify({ phase, package: packageName, time: Date.now(), args: args.slice(separator + 2) }) + '\\n');",
      "  record('start');",
      "  setTimeout(() => {",
      "    console.log('publisher stdout ' + packageName);",
      "    console.error('publisher stderr ' + packageName);",
      "    record('end');",
      "    process.exit(packageName === process.env.MOCK_PUBLISH_FAIL_PACKAGE ? 7 : 0);",
      "  }, Number(process.env.MOCK_PUBLISH_DELAY_MS ?? '150'));",
      "} else {",
      "  const result = spawnSync(process.env.REAL_MOON_BIN, args, { stdio: 'inherit', env: { ...process.env, MOON_BIN: process.env.MOCK_CHILD_MOON_BIN ?? process.env.MOON_BIN } });",
      "  if (result.error) console.error(result.error);",
      "  process.exit(result.status ?? 1);",
      "}",
    ].join("\n"),
  );
  const env = {
    MOON_BIN: path.join(binDir, "moon"),
    REAL_MOON_BIN: realMoon,
    MOCK_PUBLISH_EVENTS: eventLog,
    MOON_HOME: process.env.MOON_HOME ?? path.join(repoRoot, ".cache", "moonbit"),
  };
  return {
    baseDir,
    binDir,
    env,
    receiptPath,
    events: () =>
      fs
        .readFileSync(eventLog, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line) as Event),
    receipt: () => JSON.parse(fs.readFileSync(receiptPath, "utf8")) as PublishReceipt,
    cleanup: () => rmSync(tempDir, { recursive: true, force: true }),
  };
}

export function maxActivePublishers(events: Event[]): number {
  let active = 0;
  let peak = 0;
  for (const event of events) {
    active += event.phase === "start" ? 1 : -1;
    peak = Math.max(peak, active);
    assert.ok(active >= 0);
  }
  assert.equal(active, 0, "all launched publisher subprocesses must finish");
  return peak;
}
