import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { once } from "node:events";
import fs from "node:fs";
import { createServer } from "node:http";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  npmTarballIsVisible,
  type RegistryProbeObservation,
} from "../../tools/support/release/publish_npm_package/registry.ts";
import { parseJson } from "../../tools/support/release/ordered_json.ts";
import { writeFakeCommand } from "./support/fake-command.ts";
import { releaseCommandPath, repoRoot } from "./support/release-command.ts";
import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";
import { planSourceChecks } from "../../tools/support/compat/github/plan-source-checks.ts";

const payload = Buffer.alloc(2 * 1024 * 1024 + 17);
for (let index = 0; index < payload.length; index++) payload[index] = index % 251;
const integrity = `sha512-${createHash("sha512").update(payload).digest("base64")}`;
const metadata = (tarball: string, sri = integrity) => {
  const result = parseJson(JSON.stringify({ dist: { tarball, integrity: sri } }));
  assert.ok(result);
  return result;
};

async function fixture() {
  const requests: { path: string; method?: string; range?: string }[] = [];
  const server = createServer((request, response) => {
    requests.push({
      path: request.url ?? "",
      method: request.method,
      range: request.headers.range,
    });
    if (request.url === "/missing.tgz") {
      response.writeHead(404).end("not propagated");
      return;
    }
    if (request.url === "/headers-stall.tgz") return;
    if (request.url === "/redirect.tgz") {
      response.writeHead(302, { Location: "/correct.tgz" }).end();
      return;
    }
    response.writeHead(200, {
      "Content-Type": "application/octet-stream",
      ...(request.url === "/truncated.tgz" ? { "Content-Length": payload.length } : {}),
    });
    if (request.url === "/empty.tgz") {
      response.end();
      return;
    }
    if (request.url === "/body-stall.tgz") {
      response.write(payload.subarray(0, 100));
      return;
    }
    if (request.url === "/truncated.tgz") {
      response.end(payload.subarray(0, 100));
      return;
    }
    const bytes = request.url === "/corrupt-tail.tgz" ? Buffer.from(payload) : payload;
    if (bytes !== payload) bytes[bytes.length - 1] ^= 1;
    // Real separate binary writes; a correct prefix cannot authenticate a corrupt tail.
    let offset = 0;
    const send = () => {
      if (response.destroyed) return;
      if (offset === bytes.length) {
        response.end();
        return;
      }
      const end = Math.min(offset + 65536, bytes.length);
      const ready = response.write(bytes.subarray(offset, end));
      offset = end;
      if (ready) setImmediate(send);
      else response.once("drain", send);
    };
    send();
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  return {
    requests,
    url: `http://127.0.0.1:${address.port}`,
    close: async () => {
      server.closeAllConnections();
      await new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      );
    },
  };
}

await test("ordinary archive GET authenticates every binary byte and refuses HTTP or SRI failures", async () => {
  const http = await fixture();
  try {
    for (const [route, expected, sri] of [
      ["correct", true, integrity],
      ["corrupt-tail", false, integrity],
      ["correct", false, `sha512-${Buffer.alloc(64).toString("base64")}`],
      ["missing", false, integrity],
      ["truncated", false, integrity],
      ["empty", false, integrity],
      ["redirect", false, integrity],
    ] as const) {
      assert.equal(
        await npmTarballIsVisible(metadata(`${http.url}/${route}.tgz`, sri)),
        expected,
        route,
      );
    }
    assert.deepEqual(
      http.requests.map(({ path: route }) => route),
      [
        "/correct.tgz",
        "/corrupt-tail.tgz",
        "/correct.tgz",
        "/missing.tgz",
        "/truncated.tgz",
        "/empty.tgz",
        "/redirect.tgz",
      ],
    );
    assert.ok(
      http.requests.every((request) => request.method === "GET" && request.range === undefined),
    );
    for (const value of [
      {},
      { dist: {} },
      { dist: { tarball: `${http.url}/correct.tgz`, integrity: "sha1-weak" } },
      { dist: { tarball: "file:///tmp/archive.tgz", integrity } },
      { dist: { tarball: "https://user:password@example.invalid/a.tgz", integrity } },
    ]) {
      const input = parseJson(JSON.stringify(value));
      assert.ok(input);
      assert.equal(await npmTarballIsVisible(input), false);
    }
    assert.equal(
      http.requests.length,
      7,
      "malformed manifests must not initiate any archive request",
    );
  } finally {
    await http.close();
  }
});

await test("both missing response headers and a stalled successful body retain the six-second bound", async () => {
  const http = await fixture();
  const started = performance.now();
  try {
    const results = await Promise.all(
      ["headers-stall", "body-stall"].map((route) =>
        npmTarballIsVisible(metadata(`${http.url}/${route}.tgz`)),
      ),
    );
    assert.deepEqual(results, [false, false]);
    assert.ok(
      performance.now() - started < 9000,
      "whole GET deadline includes response body consumption",
    );
    assert.equal(http.requests.length, 2);
  } finally {
    await http.close();
  }
});

await test("already-created versions cannot release through metadata plus latest while their archive is absent", async () => {
  const http = await fixture();
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-npm-archive-gate-"));
  const bin = path.join(temp, "bin"),
    pkg = path.join(temp, "pkg"),
    trace = path.join(temp, "publish.log");
  fs.mkdirSync(bin);
  fs.mkdirSync(pkg);
  fs.writeFileSync(
    path.join(pkg, "package.json"),
    JSON.stringify({ name: "@vizejs/example", version: "1.2.3" }),
  );
  // Resolve the real executable before installing the child-only mock PATH.
  const resolver = process.platform === "win32" ? "where.exe" : "which";
  const curl = execFileSync(resolver, ["curl"], { encoding: "utf8" }).trim().split(/\r?\n/)[0];
  assert.ok(curl, "the real curl executable is required for HTTP controls");
  writeFakeCommand(
    bin,
    "vp",
    `require('node:fs').appendFileSync(${JSON.stringify(trace)}, 'UNEXPECTED PUBLISH\\n'); process.exit(1);`,
  );
  writeFakeCommand(
    bin,
    "curl",
    [
      "const url = new URL(process.argv.at(-1));",
      "if (url.pathname.endsWith('.tgz')) {",
      "  if (process.env.TEST_PROBE_MODE === 'frame') process.stderr.write('unexpected diagnostic frame\\n');",
      `  const child = require('node:child_process').spawnSync(${JSON.stringify(curl)}, process.argv.slice(2), { stdio: 'inherit' });`,
      "  process.exit(child.status ?? 1);",
      "}",
      "if (process.env.TEST_METADATA_DELAY_MS && url.searchParams.get('vize-publish-check')?.endsWith('-1')) await new Promise((resolve) => setTimeout(resolve, Number(process.env.TEST_METADATA_DELAY_MS)));",
      "const probe = url.searchParams.get('vize-publish-check')?.endsWith('-1');",
      "const mode = process.env.TEST_PROBE_MODE;",
      "const body = { name: '@vizejs/example', version: probe && mode === 'version' ? '9.9.9' : '1.2.3', 'dist-tags': { latest: probe && mode === 'tag' ? '9.9.9' : '1.2.3' },",
      `  dist: { tarball: process.env.TEST_TARBALL_URL, integrity: ${JSON.stringify(integrity)} } };`,
      "process.stdout.write(JSON.stringify(body) + 'VIZE_HTTP_STATUS:200');",
    ].join("\n"),
  );
  const run = (route: string, metadataDelayMs = 0, mode = "") =>
    new Promise<{ code: number | null; stdout: string; stderr: string }>((resolve, reject) => {
      const child = spawn(process.execPath, [releaseCommandPath("publish_npm_package"), pkg], {
        cwd: temp,
        env: {
          ...process.env,
          PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
          VIZE_REPO_ROOT: temp,
          NPM_TAG: "latest",
          TEST_TARBALL_URL: `${http.url}/${route}.tgz`,
          TEST_METADATA_DELAY_MS: String(metadataDelayMs),
          TEST_PROBE_MODE: mode,
          PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
          PUBLISH_RESOLUTION_RETRY_DELAY: "1",
        },
        stdio: ["ignore", "pipe", "pipe"],
      });
      let stdout = "",
        stderr = "";
      child.stdout.on("data", (value) => {
        stdout += value.toString();
      });
      child.stderr.on("data", (value) => {
        stderr += value.toString();
      });
      child.once("error", reject);
      child.once("close", (code) => resolve({ code, stdout, stderr }));
    });
  const probe = (stdout: string) => {
    const line = stdout.split("\n").find((line) => line.startsWith("Registry visibility probe: "));
    assert.ok(line, stdout);
    return JSON.parse(line.slice("Registry visibility probe: ".length)) as {
      versionMatches: boolean;
      tagVisible: boolean;
      archiveVisible: boolean;
      requests: RegistryProbeObservation[];
    };
  };
  try {
    for (const mode of ["version", "tag"]) {
      const result = await run("correct", 0, mode);
      assert.equal(result.code, 1, result.stdout + result.stderr);
      const observed = probe(result.stdout);
      assert.equal(observed.versionMatches, mode !== "version");
      assert.equal(observed.tagVisible, mode !== "tag");
      assert.equal(observed.archiveVisible, false);
      assert.equal(observed.requests.length, 2, "wrong metadata/tag must not request an archive");
      assert.ok(
        observed.requests.every((entry) => entry.httpStatus === "200" && entry.reason === "json"),
      );
      assert.doesNotMatch(result.stdout + result.stderr, /9\.9\.9|TEST_PROBE_MODE|dist-tags/);
      assert.equal(http.requests.length, 0);
    }
    for (const route of ["missing", "corrupt-tail"]) {
      const result = await run(route);
      assert.equal(result.code, 1, result.stdout + result.stderr);
      assert.match(result.stdout, /already published.*Skipping publish/);
      assert.match(result.stderr, /Timed out waiting/);
      assert.doesNotMatch(result.stdout, /is visible in npm/);
      const observed = probe(result.stdout);
      assert.equal(observed.versionMatches, true);
      assert.equal(observed.tagVisible, true);
      assert.equal(observed.archiveVisible, false);
      const archive = observed.requests.find((entry) => entry.phase === "archive");
      assert.ok(archive);
      assert.equal(archive.httpStatus, route === "missing" ? "404" : "200");
      assert.equal(archive.reason, route === "missing" ? "http-status" : "sri-mismatch");
    }
    const started = performance.now();
    const timedOut = await run("body-stall", 5000);
    assert.equal(timedOut.code, 1, timedOut.stdout + timedOut.stderr);
    const timeoutArchive = probe(timedOut.stdout).requests.find(
      (entry) => entry.phase === "archive",
    );
    assert.ok(timeoutArchive);
    assert.ok(["terminated", "deadline"].includes(timeoutArchive.reason));
    assert.ok(
      performance.now() - started < 8500,
      "metadata and archive share the original six-second attempt budget",
    );
    const framed = await run("correct", 0, "frame");
    assert.equal(framed.code, 1, framed.stdout + framed.stderr);
    const framedArchive = probe(framed.stdout).requests.find((entry) => entry.phase === "archive");
    assert.ok(framedArchive);
    assert.equal(framedArchive.curlExit, 0);
    assert.equal(framedArchive.httpStatus, "200");
    assert.equal(framedArchive.bytes, payload.length);
    assert.equal(framedArchive.reason, "status-frame");
    assert.equal(probe(framed.stdout).archiveVisible, false);
    assert.doesNotMatch(framed.stdout + framed.stderr, /unexpected diagnostic frame/);
    const result = await run("correct");
    assert.equal(result.code, 0, result.stdout + result.stderr);
    assert.match(result.stdout, /is visible in npm with dist-tag latest/);
    const success = probe(result.stdout);
    assert.equal(success.versionMatches, true);
    assert.equal(success.tagVisible, true);
    assert.equal(success.archiveVisible, true);
    const archive = success.requests.find((entry) => entry.phase === "archive");
    assert.ok(archive);
    assert.equal(archive.reason, "verified");
    assert.equal(archive.bytes, payload.length);
    assert.doesNotMatch(JSON.stringify(success), /headers|set-cookie|VIZE_HTTP_STATUS|dist-tags/);
    assert.equal(fs.existsSync(trace), false, "an existing version must never be published again");
    assert.deepEqual(
      http.requests.map((request) => request.path),
      ["/missing.tgz", "/corrupt-tail.tgz", "/body-stall.tgz", "/correct.tgz", "/correct.tgz"],
    );
  } finally {
    await http.close();
    fs.rmSync(temp, { recursive: true, force: true });
  }
});

await test("ordinary PR and merge-group source plans execute the complete publisher regression family", () => {
  const workflow = readRepoFile(".github", "workflows", "pr-source-checks.yml");
  const plan = workflowJobBody(workflow, "pr-source-plan");
  assert.match(plan, /github.event_name == 'pull_request' \|\| github.event_name == 'merge_group'/);
  const job = workflowJobBody(workflow, "ci-selection-tests");
  assert.match(job, /needs: pr-source-plan/);
  const step = job.slice(job.indexOf("      - name: Test CI selection when its inputs change"));
  assert.match(step, /if: \$\{\{ needs.pr-source-plan.outputs.tooling == 'true' \}\}/);
  const run = step.split("run: >-\n")[1];
  assert.ok(run?.trim().startsWith("node --test "));
  const patterns = run.trim().split(/\s+/).slice(2);
  const suites = fs
    .readdirSync(path.join(repoRoot, "tests/tooling"))
    .filter((name) => name.startsWith("publish-npm-package") && name.endsWith(".test.ts"))
    .map((name) => `tests/tooling/${name}`);
  assert.ok(suites.includes("tests/tooling/publish-npm-package-tarball.test.ts"));
  for (const suite of suites) {
    assert.equal(patterns.filter((pattern) => path.matchesGlob(suite, pattern)).length, 1, suite);
  }
  for (const event of ["pull_request", "merge_group"]) {
    for (const input of [
      "tools/support/release/publish_npm_package/registry.ts",
      "tests/tooling/support/fake-npm-registry.ts",
      ...suites,
    ]) {
      assert.equal(planSourceChecks([input], event).tooling, true, `${event}: ${input}`);
    }
  }
});
