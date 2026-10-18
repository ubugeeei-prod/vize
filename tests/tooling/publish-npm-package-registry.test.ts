import assert from "node:assert/strict";
import fs from "node:fs";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { fakeNpmArchiveSource } from "./support/fake-npm-registry.ts";

import { runReleaseCommand } from "./support/release-command.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

const versionUrl = "https://registry.npmjs.org/%40vizejs%2Fexample/1.2.3-beta.1";
const metadataUrl = "https://registry.npmjs.org/%40vizejs%2Fexample";
const probeQuery = "?vize-publish-check=registry-phase-test-1-";
const preflight = `${versionUrl}${probeQuery}0`;
const visibleVersion = `${versionUrl}${probeQuery}1`;
const visibleTag = `${metadataUrl}${probeQuery}1`;
const published = "publish succeeded";

function assertPublicationTrace(events: readonly string[]): void {
  assert.equal(
    events.length,
    5,
    "preflight, publish, two metadata probes and one whole archive GET",
  );
  assert.equal(events[0], preflight, "the absent version must be checked before publishing");
  assert.equal(events[1], published, "visibility probes must follow successful publishing");
  // waitForRegistryVisibility starts these independent reads with Promise.all.
  // Preserve both exact requests without imposing an order on their child processes.
  assert.equal(events[4], "https://registry.npmjs.org/@vizejs/example/-/fixture-1.2.3-beta.1.tgz");
  const visibility = events.slice(2, 4);
  assert.equal(visibility.filter((event) => event === visibleVersion).length, 1);
  assert.equal(visibility.filter((event) => event === visibleTag).length, 1);
}

test("registry publication trace preserves phases and both exact parallel probes", () => {
  const archive = "https://registry.npmjs.org/@vizejs/example/-/fixture-1.2.3-beta.1.tgz";
  assertPublicationTrace([preflight, published, visibleVersion, visibleTag, archive]);
  assertPublicationTrace([preflight, published, visibleTag, visibleVersion, archive]);
  const invalidTraces = {
    "premature visibility": [preflight, visibleTag, published, visibleVersion],
    "late preflight": [published, preflight, visibleVersion, visibleTag],
    "duplicate version": [preflight, published, visibleVersion, visibleVersion],
    "duplicate metadata": [preflight, published, visibleTag, visibleTag],
    "missing probe": [preflight, published, visibleVersion],
    "extra probe": [preflight, published, visibleVersion, visibleTag, visibleTag],
    "missing publish": [preflight, visibleVersion, visibleTag],
    "wrong version": [preflight, published, visibleVersion.replace("beta.1", "beta.2"), visibleTag],
    "wrong package": [preflight, published, visibleVersion, visibleTag.replace("example", "other")],
    "wrong registry": [
      preflight,
      published,
      visibleVersion.replace("npmjs.org", "example.com"),
      visibleTag,
    ],
    "reused probe token": [preflight, published, preflight, visibleTag],
    "foreign run token": [
      preflight,
      published,
      visibleVersion,
      visibleTag.replace("phase-test", "other"),
    ],
  };
  for (const [name, events] of Object.entries(invalidTraces)) {
    assert.throws(() => assertPublicationTrace([...events, archive]), assert.AssertionError, name);
  }
});

test("publish_npm_package computes the tag and forwards provenance to vp", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-npm-"));
  const packageDir = path.join(tempDir, "pkg");
  const binDir = path.join(tempDir, "bin");
  const argsLogPath = path.join(tempDir, "vp-args.log");
  const cwdLogPath = path.join(tempDir, "vp-cwd.log");
  const registryUrlLogPath = path.join(tempDir, "registry-urls.log");
  const statePath = path.join(tempDir, "vp-state.json");

  try {
    fs.mkdirSync(packageDir, { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    // Only vp publishes state. Concurrent registry reads must not rewrite the
    // same JSON file merely to count requests; the append-only trace counts them.
    writeFakeCommand(
      binDir,
      "curl",
      [
        fakeNpmArchiveSource,
        "const fs = require('node:fs');",
        "const url = process.argv.at(-1);",
        "const name = decodeURIComponent(new URL(url).pathname.split('/')[1]);",
        "const version = process.env.NPM_FAKE_VERSION;",
        "const statePath = process.env.NPM_FAKE_STATE_PATH;",
        "const state = fs.existsSync(statePath)",
        "  ? JSON.parse(fs.readFileSync(statePath, 'utf8')) : {};",
        "fs.appendFileSync(process.env.NPM_FAKE_URL_LOG, `${url}\\n`);",
        "if (state.published !== true) {",
        "  process.stdout.write('{}VIZE_HTTP_STATUS:404');",
        "  process.exit(0);",
        "}",
        "const body = url.includes(`/${version}?`)",
        "  ? { name, version, dist: fakeDist(name, version) }",
        "  : { name, 'dist-tags': { latest: version, beta: version, rc: version } };",
        "process.stdout.write(`${JSON.stringify(body)}VIZE_HTTP_STATUS:200`);",
      ].join("\n"),
    );
    writeFileSync(
      path.join(packageDir, "package.json"),
      `${JSON.stringify({ name: "@vizejs/example", version: "1.2.3-beta.1" }, null, 2)}\n`,
    );
    writeFakeCommand(
      binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "const state = fs.existsSync(process.env.VP_STATE_PATH)",
        "  ? JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, 'utf8'))",
        "  : { published: false };",
        "if (args[0] === 'pm' && args[1] === 'publish') {",
        "  fs.writeFileSync(process.env.VP_ARGS_LOG, args.join('\\n'));",
        "  fs.writeFileSync(process.env.VP_CWD_LOG, process.cwd());",
        "  state.published = true;",
        "  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));",
        `  fs.appendFileSync(process.env.NPM_FAKE_URL_LOG, ${JSON.stringify(`${published}\n`)});`,
        "  process.exit(0);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'version') {",
        "  if (state.published) {",
        "    process.stdout.write(JSON.stringify('1.2.3-beta.1'));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'dist-tags') {",
        "  if (state.published) {",
        "    process.stdout.write(JSON.stringify({ beta: '1.2.3-beta.1' }));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "process.exit(1);",
      ].join("\n"),
    );

    const result = runReleaseCommand("publish_npm_package", [packageDir, "--provenance"], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        VP_ARGS_LOG: argsLogPath,
        VP_CWD_LOG: cwdLogPath,
        VP_STATE_PATH: statePath,
        NPM_FAKE_STATE_PATH: statePath,
        NPM_FAKE_VERSION: "1.2.3-beta.1",
        NPM_FAKE_URL_LOG: registryUrlLogPath,
        GITHUB_RUN_ID: "registry-phase-test",
        GITHUB_RUN_ATTEMPT: "1",
        PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
        PUBLISH_RESOLUTION_RETRY_DELAY: "1",
      },
    });
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.deepEqual(fs.readFileSync(argsLogPath, "utf8").trim().split("\n"), [
      "pm",
      "publish",
      "--access",
      "public",
      "--no-git-checks",
      "--tag",
      "beta",
      "--",
      "--provenance",
    ]);
    assert.equal(fs.realpathSync(fs.readFileSync(cwdLogPath, "utf8")), fs.realpathSync(packageDir));
    assertPublicationTrace(fs.readFileSync(registryUrlLogPath, "utf8").trim().split("\n"));
    assert.equal(fs.readFileSync(statePath, "utf8"), JSON.stringify({ published: true }));
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
