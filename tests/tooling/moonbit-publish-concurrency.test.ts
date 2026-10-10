import assert from "node:assert/strict";
import fs from "node:fs";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";
import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

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

test("platform publisher limits concurrency and records every result in package order", () => {
  const fixture = controlledPublishFixture();
  try {
    const result = runMoonScript(
      "publish_npm_package_dirs",
      [
        fixture.baseDir,
        "--concurrency",
        "4",
        "--receipt",
        fixture.receiptPath,
        "--provenance",
        "--retries",
        "2",
        "--delay",
        "1",
      ],
      { env: fixture.env },
    );
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
    const events = fixture.events();
    assert.equal(maxActivePublishers(events), 4);
    assert.equal(events.length, 16);
    for (const event of events)
      assert.deepEqual(event.args, ["--provenance", "--retries", "2", "--delay", "1"]);
    const receipt = fixture.receipt();
    assert.equal(receipt.schema_version, 1);
    assert.equal(receipt.concurrency, 4);
    assert.equal(receipt.package_count, 8);
    assert.equal(receipt.exit_code, 0);
    assert.ok(receipt.walltime_ms > 0);
    assert.deepEqual(
      receipt.results.map((item) => path.basename(item.package_dir)),
      Array.from({ length: 8 }, (_, i) => `pkg-${i}`),
    );
    assert.ok(receipt.results.every((item) => item.exit_code === 0 && item.walltime_ms >= 150));
  } finally {
    fixture.cleanup();
  }
});

test("platform publisher keeps the default serial mode and supports explicit concurrency 1", () => {
  for (const args of [[], ["--concurrency", "1"]]) {
    const fixture = controlledPublishFixture(3);
    try {
      const result = runMoonScript(
        "publish_npm_package_dirs",
        [fixture.baseDir, ...args, "--receipt", fixture.receiptPath],
        { env: fixture.env },
      );
      assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
      assert.equal(maxActivePublishers(fixture.events()), 1);
      assert.equal(fixture.receipt().concurrency, 1);
    } finally {
      fixture.cleanup();
    }
  }
});

test("a failed publisher preserves child diagnostics and waits for all launched publishers", () => {
  const fixture = controlledPublishFixture();
  try {
    const result = runMoonScript(
      "publish_npm_package_dirs",
      [fixture.baseDir, "--concurrency", "4", "--receipt", fixture.receiptPath],
      { env: { ...fixture.env, MOCK_PUBLISH_FAIL_PACKAGE: "pkg-1" } },
    );
    assert.equal(result.status, 7, `${result.stderr}\n${result.stdout}`);
    const events = fixture.events();
    assert.equal(maxActivePublishers(events), 4);
    assert.equal(events.filter((event) => event.phase === "end").length, 8);
    for (let i = 0; i < 8; i++) {
      assert.match(result.stdout, new RegExp(`publisher stdout pkg-${i}`));
      assert.match(result.stderr, new RegExp(`publisher stderr pkg-${i}`));
    }
    assert.equal(fixture.receipt().exit_code, 7);
    assert.deepEqual(
      fixture.receipt().results.map((item) => item.exit_code),
      [0, 7, 0, 0, 0, 0, 0, 0],
    );
  } finally {
    fixture.cleanup();
  }
});

test("platform concurrency rejects invalid or missing values before starting a publisher", () => {
  for (const value of ["0", "-1", "1.5", "abc", "2oops", "5", "9", "", undefined]) {
    const fixture = controlledPublishFixture(1);
    try {
      const args = [fixture.baseDir, "--concurrency", ...(value === undefined ? [] : [value])];
      const result = runMoonScript("publish_npm_package_dirs", args, { env: fixture.env });
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /--concurrency requires an integer from 1 to 4/);
      assert.equal(
        fs.existsSync(path.join(path.dirname(fixture.receiptPath), "events.jsonl")),
        false,
      );
    } finally {
      fixture.cleanup();
    }
  }
});

test("publisher spawn errors become failed results without aborting the result collection", () => {
  const fixture = controlledPublishFixture(3);
  try {
    const result = runMoonScript(
      "publish_npm_package_dirs",
      [fixture.baseDir, "--concurrency", "3", "--receipt", fixture.receiptPath],
      { env: { ...fixture.env, MOCK_CHILD_MOON_BIN: path.join(fixture.binDir, "missing-moon") } },
    );
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Failed to run publisher/);
    assert.equal(fixture.receipt().results.length, 3);
    assert.ok(fixture.receipt().results.every((item) => item.exit_code === 1));
  } finally {
    fixture.cleanup();
  }
});

test("receipt write failures occur after all launched publishers finish", () => {
  const fixture = controlledPublishFixture(3);
  try {
    const missingParent = path.join(path.dirname(fixture.receiptPath), "missing", "receipt.json");
    const result = runMoonScript(
      "publish_npm_package_dirs",
      [fixture.baseDir, "--concurrency", "3", "--receipt", missingParent],
      { env: fixture.env },
    );
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Failed to write publish receipt/);
    assert.equal(maxActivePublishers(fixture.events()), 3);
    assert.equal(fixture.events().filter((event) => event.phase === "end").length, 3);
  } finally {
    fixture.cleanup();
  }
});

test("concurrent real package publishers retain provenance, retry, tag visibility, and rerun safety", () => {
  const fixture = controlledPublishFixture(3);
  const stateDir = path.join(path.dirname(fixture.receiptPath), "registry");
  fs.mkdirSync(stateDir);
  try {
    writeFakeCommand(
      fixture.binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "const path = require('node:path');",
        "const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));",
        "const state = path.join(process.env.MOCK_REGISTRY_DIR, pkg.name.split('/')[1]);",
        "fs.mkdirSync(state, { recursive: true });",
        "const attempts = path.join(state, 'attempts.jsonl');",
        "fs.appendFileSync(attempts, JSON.stringify(process.argv.slice(2)) + '\\n');",
        "const count = fs.readFileSync(attempts, 'utf8').trim().split('\\n').length;",
        "if (count === 1) { console.error('temporary publish failure ' + pkg.name); process.exit(1); }",
        "fs.writeFileSync(path.join(state, 'published'), 'yes');",
      ].join("\n"),
    );
    writeFakeCommand(
      fixture.binDir,
      "curl",
      [
        "const fs = require('node:fs');",
        "const path = require('node:path');",
        "const url = new URL(process.argv.at(-1));",
        "const name = decodeURIComponent(url.pathname.split('/')[1]);",
        "const state = path.join(process.env.MOCK_REGISTRY_DIR, name.split('/')[1]);",
        "if (!fs.existsSync(path.join(state, 'published'))) { process.stdout.write('{}VIZE_HTTP_STATUS:404'); process.exit(0); }",
        "let body;",
        "if (url.pathname.split('/').length === 3) body = { name, version: '1.0.0' };",
        "else {",
        "  const tagMarker = path.join(state, 'tag-probed');",
        "  const visible = fs.existsSync(tagMarker);",
        "  fs.writeFileSync(tagMarker, 'yes');",
        "  body = { name, 'dist-tags': visible ? { latest: '1.0.0' } : {} };",
        "}",
        "process.stdout.write(JSON.stringify(body) + 'VIZE_HTTP_STATUS:200');",
      ].join("\n"),
    );
    const env = {
      ...fixture.env,
      MOON_BIN: fixture.env.REAL_MOON_BIN,
      PATH: `${fixture.binDir}${path.delimiter}${process.env.PATH ?? ""}`,
      VP_BIN: path.join(fixture.binDir, "vp"),
      MOCK_REGISTRY_DIR: stateDir,
      PUBLISH_RESOLUTION_RETRY_LIMIT: "3",
      PUBLISH_RESOLUTION_RETRY_DELAY: "1",
    };
    const args = [
      fixture.baseDir,
      "--concurrency",
      "3",
      "--receipt",
      fixture.receiptPath,
      "--provenance",
      "--retries",
      "2",
      "--delay",
      "1",
    ];
    const first = runMoonScript("publish_npm_package_dirs", args, { env });
    assert.equal(first.status, 0, `${first.stderr}\n${first.stdout}`);
    assert.match(first.stdout, /Waiting for .*dist-tag latest/);
    assert.match(first.stderr, /temporary publish failure/);
    for (let i = 0; i < 3; i++) {
      const attempts = fs
        .readFileSync(path.join(stateDir, `pkg-${i}`, "attempts.jsonl"), "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line));
      assert.deepEqual(
        attempts,
        Array(2).fill([
          "pm",
          "publish",
          "--access",
          "public",
          "--no-git-checks",
          "--tag",
          "latest",
          "--",
          "--provenance",
        ]),
      );
    }
    assert.ok(
      fixture.receipt().results.every((item) => item.exit_code === 0 && item.walltime_ms >= 2000),
    );
    const second = runMoonScript("publish_npm_package_dirs", args, { env });
    assert.equal(second.status, 0, `${second.stderr}\n${second.stdout}`);
    assert.equal(second.stdout.match(/already published/g)?.length, 3);
    for (let i = 0; i < 3; i++) {
      assert.equal(
        fs
          .readFileSync(path.join(stateDir, `pkg-${i}`, "attempts.jsonl"), "utf8")
          .trim()
          .split("\n").length,
        2,
      );
    }
  } finally {
    fixture.cleanup();
  }
});

test("release platform packages use bounded concurrency before the native wrapper", () => {
  const source = workflowJobBody(
    readRepoFile(".github", "workflows", "release.yml"),
    "release-npm-native",
  );
  assert.match(source, /publish_npm_package_dirs -- npm\/native\/npm --concurrency 4 --provenance/);
  assert.match(source, /--receipt "\$RUNNER_TEMP\/native-platform-publish\.json"/);
  assert.match(
    source,
    /name: Upload native platform publish receipt\s+if: \$\{\{ always\(\) \}\}\s+uses: actions\/upload-artifact@[0-9a-f]{40}/,
  );
  assert.match(
    source,
    /name: native-platform-publish-receipt\s+path: \$\{\{ runner\.temp \}\}\/native-platform-publish\.json/,
  );
  const platforms = source.indexOf("publish_npm_package_dirs --");
  const wrapper = source.indexOf("publish_npm_package -- npm/native --provenance");
  assert.ok(platforms >= 0 && wrapper > platforms);
  assert.doesNotMatch(source, /continue-on-error|--ignore-failure/);
});
