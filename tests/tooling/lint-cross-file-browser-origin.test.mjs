import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { browserOriginCurrentReference } from "../differential/browser-origin-current-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtures = path.join(root, "crates/vize/tests/fixtures/issue-7907");
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");

function corpus() {
  const source = JSON.parse(fs.readFileSync(path.join(fixtures, "source.json"), "utf8"));
  assert.equal(source.schema, "vize.cross-file.browser-origin-corpus");
  assert.equal(source.version, 1);
  assert.equal(source.author.login, "ubugeeei");
  assert.equal(source.author.id, 71201308);
  const body = fs.readFileSync(path.join(fixtures, "issue.md"));
  assert.equal(digest(body), source.body_sha256);
  const heredocs = [
    ...body.toString("utf8").matchAll(/cat > (\w+\.vue) <<'VUE'\n([\s\S]*?)\nVUE/g),
  ];
  assert.equal(heredocs.length, 2);
  assert.equal(source.cases.length, 11);
  assert.equal(new Set(source.cases.map((entry) => entry.path)).size, 11);
  for (const [index, entry] of source.cases.entries()) {
    assert.match(entry.path, /^[A-Za-z]+\.vue$/);
    assert.equal(entry.file, `${entry.path}.fixture`);
    const bytes = fs.readFileSync(path.join(fixtures, entry.file));
    assert.equal(bytes.length, entry.bytes);
    assert.equal(digest(bytes), entry.sha256);
    if (index < 2) {
      assert.equal(entry.path, heredocs[index][1]);
      assert.equal(bytes.toString("utf8"), `${heredocs[index][2]}\n`);
    }
  }
  return source;
}

await test("authored browser-origin corpus preserves both complete issue heredocs", () => {
  const source = corpus();
  assert.deepEqual(
    source.cases
      .slice(0, 2)
      .map((entry) => entry.expected[0].messages.map(({ line, column }) => [line, column])),
    [[[2, 15]], [[6, 35]]],
  );
});

await test("source-built cross-file lint preserves complete browser-origin output and off controls", () => {
  const source = corpus();
  const artifact = path.join(root, "target/differential/lint-cross-file-browser-origin.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = { schema: "vize.lint.browser-origin", version: 1, runs: [] };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-browser-origin-"));
  try {
    const identity = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`), "utf8"),
    );
    evidence.build = { identity, receipt };
    evidence.corpusSha256 = digest(fs.readFileSync(path.join(fixtures, "source.json")));
    const current = browserOriginCurrentReference(root, source);
    evidence.currentReference = current;
    persist();
    validateBuildReceipt(receipt, identity);
    const cli = path.join(root, identity.binaryPath);
    const env = { ...process.env, NO_COLOR: "1", VIZE_LOG: "warn" };
    delete env.FORCE_COLOR;
    for (const entry of source.cases) {
      const input = fs.readFileSync(path.join(fixtures, entry.file));
      const target = path.join(workspace, entry.path);
      fs.writeFileSync(target, input);
      for (const enabled of [false, true]) {
        const args = [
          "lint",
          entry.path,
          "--no-config",
          "--preset",
          "incremental",
          "--format",
          "json",
          "--locale",
          "en",
          "--help-level",
          "none",
          ...(enabled ? ["--cross-file"] : []),
        ];
        const result = spawnSync(cli, args, {
          cwd: workspace,
          env,
          encoding: "utf8",
          timeout: 60_000,
          maxBuffer: 8 * 1024 * 1024,
        });
        const run = {
          command: cli,
          args,
          status: result.status,
          signal: result.signal,
          error: result.error ? { name: result.error.name, message: result.error.message } : null,
          stdout: result.stdout ?? "",
          stderr: result.stderr ?? "",
        };
        const expected = enabled
          ? entry.path === current.row.path
            ? current.row.currentExpected
            : entry.expected
          : [{ file: entry.path, messages: [], errorCount: 0, warningCount: 0 }];
        evidence.runs.push({
          input: entry,
          enabled,
          run,
          expected,
          historicalExpected: enabled ? entry.expected : expected,
          afterSha256: digest(fs.readFileSync(target)),
        });
        persist();
        assert.equal(run.error, null, JSON.stringify(run));
        assert.equal(run.signal, null, JSON.stringify(run));
        assert.equal(run.status, 0, JSON.stringify(run));
        assert.equal(run.stderr, "", JSON.stringify(run));
        assert.deepEqual(JSON.parse(run.stdout), expected, JSON.stringify(run));
        if (enabled && entry.path === current.row.path) {
          assert.notDeepEqual(JSON.parse(run.stdout), entry.expected);
        }
        assert.deepEqual(fs.readFileSync(target), input, "lint must not alter original source");
      }
      fs.unlinkSync(target);
    }
    evidence.complete = { inputs: source.cases.length, runs: evidence.runs.length };
    assert.deepEqual(evidence.complete, { inputs: 11, runs: 22 });
  } finally {
    persist();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
