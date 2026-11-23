import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { test, type TestContext } from "node:test";

import { runReleaseCommand } from "./support/release-command.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

test("publish_open_vsx_extension publishes a packaged VSIX with ovsx", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-open-vsx-publish-"));
  const binDir = path.join(tempDir, "bin");
  const manifestPath = path.join(tempDir, "package.json");
  const vsixPath = path.join(tempDir, "vize.vsix");
  const statePath = path.join(tempDir, "state.json");
  const callsPath = path.join(tempDir, "calls.json");
  const curlCallsPath = path.join(tempDir, "curl-calls.json");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(
      manifestPath,
      `${JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" }, null, 2)}\n`,
    );
    writeFileSync(vsixPath, "vsix");
    writeFakeCommand(
      binDir,
      "vp",
      readToolingProgram("commands/publish-open-vsx-extension/vp-publishes-packaged-vsix-ovsx.js"),
    );
    writeFakeCommand(
      binDir,
      "curl",
      readToolingProgram(
        "commands/publish-open-vsx-extension/curl-publishes-packaged-vsix-ovsx.js",
      ),
    );

    const runPublish = (namespaceError?: string) =>
      runReleaseCommand("publish_open_vsx_extension", [vsixPath, manifestPath], {
        env: {
          PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
          CALLS_PATH: callsPath,
          CURL_CALLS_PATH: curlCallsPath,
          STATE_PATH: statePath,
          NAMESPACE_ERROR: namespaceError ?? "",
        },
      });

    const rejected = runPublish("auth");
    assert.equal(rejected.status, 1);
    assert.match(rejected.stderr, /token is not authorized/);
    assert.doesNotMatch(rejected.stdout, /continuing to publish/);
    const rejectedCalls = JSON.parse(fs.readFileSync(callsPath, "utf8")) as string[][];
    assert.deepEqual(
      rejectedCalls.map((call) => call[4]),
      ["create-namespace"],
    );
    const expectedLookup = [
      "--fail",
      "--silent",
      "--show-error",
      "https://open-vsx.org/api/ubugeeei/vize/universal/0.57.0",
    ];
    assert.deepEqual(JSON.parse(fs.readFileSync(curlCallsPath, "utf8")), [expectedLookup]);

    writeFileSync(callsPath, "[]");
    writeFileSync(curlCallsPath, "[]");
    const result = runPublish();
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /already exists; continuing to publish/);

    const calls = JSON.parse(fs.readFileSync(callsPath, "utf8")) as string[][];
    assert.deepEqual(calls.at(-2), [
      "dlx",
      "-p",
      "ovsx@^1.0.0",
      "ovsx",
      "create-namespace",
      "ubugeeei",
    ]);
    assert.deepEqual(calls.at(-1), ["dlx", "-p", "ovsx@^1.0.0", "ovsx", "publish", vsixPath]);
    assert.deepEqual(JSON.parse(fs.readFileSync(curlCallsPath, "utf8")), [
      expectedLookup,
      expectedLookup,
    ]);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_open_vsx_extension selects the published universal version even when the generic endpoint selects another platform", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-open-vsx-skip-"));
  const binDir = path.join(tempDir, "bin");
  const manifestPath = path.join(tempDir, "package.json");
  const vsixPath = path.join(tempDir, "vize.vsix");
  const callsPath = path.join(tempDir, "calls.json");
  const curlCallsPath = path.join(tempDir, "curl-calls.json");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(
      manifestPath,
      `${JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" }, null, 2)}\n`,
    );
    writeFileSync(vsixPath, "vsix");
    writeFileSync(callsPath, "[]");
    writeFakeCommand(
      binDir,
      "vp",
      readToolingProgram(
        "commands/publish-open-vsx-extension/vp-selects-published-universal-version-even-generic-endpoint-selects.js",
      ),
    );
    writeFakeCommand(
      binDir,
      "curl",
      readToolingProgram(
        "commands/publish-open-vsx-extension/curl-selects-published-universal-version-even-generic-endpoint-selects.js",
      ),
    );

    const result = runReleaseCommand("publish_open_vsx_extension", [vsixPath, manifestPath], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        CALLS_PATH: callsPath,
        CURL_CALLS_PATH: curlCallsPath,
      },
    });
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /already published/);

    const calls = JSON.parse(fs.readFileSync(callsPath, "utf8")) as string[][];
    assert.deepEqual(calls, []);
    assert.deepEqual(JSON.parse(fs.readFileSync(curlCallsPath, "utf8")), [
      [
        "--fail",
        "--silent",
        "--show-error",
        "https://open-vsx.org/api/ubugeeei/vize/universal/0.57.0",
      ],
    ]);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

// ovsx prints this and exits 1 for every write during registry maintenance.
function readOnlyRun(t: TestContext, namespaceRefusals: number, publishRefusals: number) {
  const root = mkdtempSync(path.join(tmpdir(), "release-open-vsx-read-only-")),
    bin = path.join(root, "bin"),
    state = path.join(root, "state.json"),
    manifest = path.join(root, "package.json");
  t.after(() => rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(bin);
  writeFileSync(state, JSON.stringify({ calls: [], published: false }));
  writeFileSync(
    manifest,
    JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" }),
  );
  const step = readToolingProgram("commands/publish-open-vsx-extension/read-only-registry.cjs");
  writeFakeCommand(bin, "vp", step);
  writeFakeCommand(bin, "curl", step);
  const result = runReleaseCommand(
    "publish_open_vsx_extension",
    [path.join(root, "x.vsix"), manifest],
    {
      env: {
        PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
        STATE_PATH: state,
        NAMESPACE_REFUSALS: String(namespaceRefusals),
        PUBLISH_REFUSALS: String(publishRefusals),
        OPEN_VSX_READ_ONLY_RETRY_LIMIT: "3",
        OPEN_VSX_READ_ONLY_RETRY_DELAY: "0",
        PUBLISH_RESOLUTION_RETRY_LIMIT: "2",
        PUBLISH_RESOLUTION_RETRY_DELAY: "0",
      },
    },
  );
  return {
    result,
    calls: (JSON.parse(fs.readFileSync(state, "utf8")) as { calls: string[] }).calls,
  };
}

test("publish_open_vsx_extension retries writes while Open VSX is read-only", (t) => {
  const { result, calls } = readOnlyRun(t, 2, 1);
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(calls, [
    "curl",
    "create-namespace",
    "create-namespace",
    "create-namespace",
    "publish",
    "publish",
    "curl",
  ]);
  assert.equal(
    result.stdout.split("\n").filter((line) => line.startsWith("Open VSX is in read-only mode"))
      .length,
    3,
  );
  assert.match(
    result.stdout,
    /^Open VSX is in read-only mode for maintenance; retrying ovsx create-namespace in 0 s \(1\/3\)\.$/m,
  );
});

for (const [step, namespaceRefusals, publishRefusals] of [
  ["create-namespace", 3, 0],
  ["publish", 0, 3],
] as const) {
  test(`publish_open_vsx_extension stops after a read-only ${step} without polling`, (t) => {
    const { result, calls } = readOnlyRun(t, namespaceRefusals, publishRefusals);
    assert.equal(result.status, 1);
    assert.equal(calls.filter((call) => call === step).length, 3);
    assert.deepEqual(
      calls.filter((call) => call === "curl"),
      ["curl"],
    );
    assert.match(
      result.stderr,
      /^Open VSX stayed in read-only mode \(maintenance\) through 3 attempts; ubugeeei\.vize 0\.57\.0 was not uploaded\. Re-run this job once open-vsx\.org accepts writes\.$/m,
    );
  });
}
