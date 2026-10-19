import assert from "node:assert/strict";
import fs, { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { writeFakeCommand } from "./fake-command.ts";

type Event = { phase: "start" | "end"; package: string; time: number; args: string[] };

// The parent is the actual multi-package publisher command. Only its single-package
// publisher subprocess is mocked, with a deterministic registry-wait-like delay.
export function controlledPublishFixture(packageCount = 8) {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-concurrency-"));
  const baseDir = path.join(tempDir, "packages with ' spaces");
  const binDir = path.join(tempDir, "bin");
  const eventLog = path.join(tempDir, "events.jsonl");
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
  // The fake runtime is not named `node`: fake commands start through
  // `/usr/bin/env node`, and the bin directory can lead PATH.
  writeFakeCommand(
    binDir,
    "fake-node",
    [
      "const fs = require('node:fs');",
      "const path = require('node:path');",
      "const { spawnSync } = require('node:child_process');",
      "const args = process.argv.slice(2);",
      "if (args[0]?.replaceAll('\\\\', '/').endsWith('/publish_npm_package/main.ts')) {",
      "  const packageName = path.basename(args[1]);",
      "  const record = phase => fs.appendFileSync(process.env.MOCK_PUBLISH_EVENTS, JSON.stringify({ phase, package: packageName, time: Date.now(), args: args.slice(2) }) + '\\n');",
      "  record('start');",
      "  setTimeout(() => {",
      "    console.log('publisher stdout ' + packageName);",
      "    console.error('publisher stderr ' + packageName);",
      "    record('end');",
      "    process.exit(packageName === process.env.MOCK_PUBLISH_FAIL_PACKAGE ? 7 : 0);",
      "  }, Number(process.env.MOCK_PUBLISH_DELAY_MS ?? '150'));",
      "} else {",
      "  const result = spawnSync(process.env.REAL_NODE_BIN, args, { stdio: 'inherit' });",
      "  if (result.error) console.error(result.error);",
      "  process.exit(result.status ?? 1);",
      "}",
    ].join("\n"),
  );
  const env = {
    NODE_BIN: path.join(binDir, "fake-node"),
    REAL_NODE_BIN: process.execPath,
    MOCK_PUBLISH_EVENTS: eventLog,
  };
  return {
    baseDir,
    binDir,
    env,
    events: () =>
      fs
        .readFileSync(eventLog, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line) as Event),
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
