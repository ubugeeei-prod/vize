import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { writeFakeCommand } from "./support/fake-command.ts";
import { writeFakeNpmRegistry } from "./support/fake-npm-registry.ts";
import { runReleaseCommand } from "./support/release-command.ts";
import {
  controlledPublishFixture,
  maxActivePublishers,
} from "./support/publish-npm-package-dirs-fixture.ts";

const publish = (args: string[], env: Record<string, string>) =>
  runReleaseCommand("publish_npm_package_dirs", args, { env });

test("every package publishes at once up to the concurrency bound", () => {
  const fixture = controlledPublishFixture(8);
  try {
    const result = publish(
      [fixture.baseDir, "--concurrency", "3", "--provenance", "--no-wait", "--retries", "2"],
      fixture.env,
    );
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
    const events = fixture.events();
    assert.equal(maxActivePublishers(events), 3);
    assert.equal(events.length, 16);
    for (const event of events)
      assert.deepEqual(event.args, ["--provenance", "--no-wait", "--retries", "2", "--prepared"]);
    assert.match(result.stdout, /Published 8\/8 packages/);
  } finally {
    fixture.cleanup();
  }
});

test("package directories and parents of package directories mix freely", () => {
  const fixture = controlledPublishFixture(2);
  try {
    const single = path.join(fixture.baseDir, "pkg-0");
    const result = publish([single, fixture.baseDir], fixture.env);
    assert.equal(result.status, 0, result.stderr);
    const started = fixture.events().filter((event) => event.phase === "start");
    assert.deepEqual(started.map((event) => event.package).sort(), ["pkg-0", "pkg-0", "pkg-1"]);
    const names = publish(["--names", single, fixture.baseDir], fixture.env);
    assert.equal(names.stdout.trim(), "@test/pkg-0 @test/pkg-0 @test/pkg-1");
  } finally {
    fixture.cleanup();
  }
});

test("a failed package fails the run after every package was attempted", () => {
  const fixture = controlledPublishFixture(4);
  try {
    const result = publish([fixture.baseDir, "--concurrency", "2"], {
      ...fixture.env,
      MOCK_PUBLISH_FAIL_PACKAGE: "pkg-1",
    });
    assert.equal(result.status, 1);
    assert.equal(fixture.events().filter((event) => event.phase === "end").length, 4);
    assert.match(result.stderr, /Failed: .*pkg-1/);
    assert.match(result.stderr, /publisher stderr pkg-1/);
  } finally {
    fixture.cleanup();
  }
});

test("invalid arguments are rejected before any publisher starts", () => {
  const fixture = controlledPublishFixture(1);
  try {
    for (const args of [
      [fixture.baseDir, "--concurrency", "0"],
      [fixture.baseDir, "--concurrency", "17"],
      ["--provenance"],
      [path.join(fixture.baseDir, "skip-helper")],
    ]) {
      assert.equal(publish(args, fixture.env).status, 1, args.join(" "));
    }
  } finally {
    fixture.cleanup();
  }
});

test("--no-wait returns after a successful publish without polling the registry", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "release-publish-no-wait-"));
  const bin = path.join(dir, "bin");
  const urls = path.join(dir, "urls.log");
  try {
    fs.mkdirSync(path.join(dir, "pkg"), { recursive: true });
    fs.mkdirSync(bin);
    fs.writeFileSync(
      path.join(dir, "pkg", "package.json"),
      JSON.stringify({ name: "@vizejs/example", version: "1.0.0" }),
    );
    writeFakeNpmRegistry(bin);
    writeFakeCommand(bin, "vp", "process.exit(0);");
    const env = {
      PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
      NPM_FAKE_URL_LOG: urls,
      NPM_FAKE_VERSION: "1.0.0",
      PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
      PUBLISH_RESOLUTION_RETRY_DELAY: "1",
    };
    const pkg = path.join(dir, "pkg");
    const waited = runReleaseCommand("publish_npm_package", [pkg], { env });
    assert.equal(waited.status, 1, "the default waits and times out on an invisible version");
    fs.rmSync(urls);
    const fast = runReleaseCommand("publish_npm_package", [pkg, "--no-wait"], { env });
    assert.equal(fast.status, 0, fast.stderr);
    // Only the pre-publish "already published?" probe touches the registry.
    assert.equal(fs.readFileSync(urls, "utf8").trim().split("\n").length, 1);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test("manifests are resolved before any publish rewrites a sibling package.json", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "release-publish-prepare-"));
  const bin = path.join(root, "bin");
  try {
    fs.mkdirSync(bin);
    fs.writeFileSync(path.join(root, "pnpm-workspace.yaml"), 'packages:\n  - "npm/*"\n');
    const manifest = (name: string, dependencies = {}) =>
      JSON.stringify({ name, version: "1.0.0", dependencies });
    for (const [dir, json] of [
      ["b", manifest("@vizejs/b")],
      ["a", manifest("@vizejs/a", { "@vizejs/b": "workspace:*" })],
    ]) {
      fs.mkdirSync(path.join(root, "npm", dir), { recursive: true });
      fs.writeFileSync(path.join(root, "npm", dir, "package.json"), json);
    }
    writeFakeNpmRegistry(bin);
    // Publishing b rewrites its manifest non-atomically, like napi's prepublishOnly.
    writeFakeCommand(
      bin,
      "vp",
      "const fs = require('node:fs'); if (process.cwd().endsWith('b')) fs.writeFileSync('package.json', '{'); process.exit(0);",
    );
    const result = runReleaseCommand(
      "publish_npm_package_dirs",
      [path.join(root, "npm/b"), path.join(root, "npm/a"), "--concurrency", "1", "--no-wait"],
      {
        cwd: root,
        env: {
          PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
          VIZE_REPO_ROOT: root,
          NPM_FAKE_VERSION: "1.0.0",
        },
      },
    );
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
    const published = JSON.parse(fs.readFileSync(path.join(root, "npm/a/package.json"), "utf8"));
    assert.deepEqual(published.dependencies, { "@vizejs/b": "1.0.0" });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
