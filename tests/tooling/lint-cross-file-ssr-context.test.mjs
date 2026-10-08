import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtures = path.join(root, "crates/vize/tests/fixtures/issue-7908");
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const originalPaths = ["HomeButton.vue", "GuardedWatch.vue", "GuardedEffect.vue"];
const originalHashes = [
  "6de8b0b5e2023d4dfa60d307e5736b88e687b31b64606589f0930c4e2307d385",
  "c59cd2c0bbe346e4b4500d5f48192960257e18311cb4457bf30f7ed52ed7e26d",
  "e90cb0c2fdde981f5acec05fcd7f75aa72de21d35d4587e6b747c93a7090aa9d",
];

function corpus() {
  const source = JSON.parse(fs.readFileSync(path.join(fixtures, "source.json"), "utf8"));
  assert.equal(source.schema, "vize.cross-file.ssr-context-corpus");
  assert.equal(source.version, 1);
  assert.equal(source.issue, "https://github.com/ubugeeei-prod/vize/issues/7908");
  assert.deepEqual(source.author, { login: "ubugeeei", id: 71201308 });
  const metadataBytes = fs.readFileSync(path.join(fixtures, "issue.json"));
  const metadata = JSON.parse(metadataBytes);
  assert.equal(digest(metadataBytes), source.metadata_sha256);
  assert.equal(metadata.number, 7908);
  assert.equal(metadata.html_url, source.issue);
  assert.equal(metadata.user.login, source.author.login);
  assert.equal(metadata.user.id, source.author.id);
  assert.equal(metadata.created_at, source.created_at);
  assert.equal(metadata.updated_at, source.updated_at);
  const body = fs.readFileSync(path.join(fixtures, "issue.md"));
  assert.equal(body.toString("utf8"), metadata.body);
  assert.equal(digest(body), source.body_sha256);
  assert.equal(
    source.body_sha256,
    "c3a02b364d06c47f6b23a03016f63f0d910c7ab183ec7b6cddbe7351ae7447cf",
  );
  const heredocs = [...metadata.body.matchAll(/cat > (\w+\.vue) <<'VUE'\n([\s\S]*?)\nVUE/g)];
  assert.equal(heredocs.length, 3);
  assert.equal(source.cases.length, 38);
  assert.equal(new Set(source.cases.map((entry) => entry.path)).size, source.cases.length);
  for (const [index, entry] of source.cases.entries()) {
    assert.match(entry.path, /^[A-Za-z]+\.vue$/);
    assert.equal(entry.file, `${entry.path}.fixture`);
    const bytes = fs.readFileSync(path.join(fixtures, entry.file));
    assert.equal(bytes.length, entry.bytes);
    assert.equal(digest(bytes), entry.sha256);
    if (index < 3) {
      assert.equal(entry.path, originalPaths[index]);
      assert.equal(entry.path, heredocs[index][1]);
      assert.equal(entry.sha256, originalHashes[index]);
      assert.equal(bytes.toString("utf8"), `${heredocs[index][2]}\n`);
    }
    assert.deepEqual(entry.offExpected, [
      { file: entry.path, messages: [], errorCount: 0, warningCount: 0 },
    ]);
    assert.equal(entry.expected.length, 1);
    assert.equal(entry.expected[0].file, entry.path);
    assert.equal(entry.expected[0].warningCount, entry.doctorLocations.length);
    assert.equal(entry.expected[0].messages.length, entry.doctorLocations.length);
    for (const [i, at] of entry.doctorLocations.entries()) {
      const prefix = bytes.subarray(0, at.start).toString("utf8");
      assert.equal(at.end, at.start);
      assert.equal(at.line, prefix.split("\n").length);
      assert.equal(at.column, prefix.split("\n").at(-1).length + 1);
      assert.match(bytes.subarray(at.start).toString("utf8"), /^(?:document|window)\b/);
      assert.deepEqual(entry.expected[0].messages[i], {
        ruleId: "cross-file",
        ruleDocsPath: "docs/content/rules/cross-file.md",
        severity: 1,
        message:
          "[vize:cross-file] vize:croquis/cf/browser-api-ssr: Browser API used in potentially SSR context",
        line: at.line,
        column: at.column,
        endLine: at.line,
        endColumn: at.column + 1,
      });
    }
  }
  assert.deepEqual(source.originalPlain.args, [
    "lint",
    "--no-config",
    "--preset",
    "opinionated",
    "--cross-file",
    "-f",
    "plain",
    ...originalPaths,
  ]);
  return source;
}

await test("authored SSR context corpus preserves all three complete issue heredocs", () => {
  const source = corpus();
  assert.deepEqual(
    source.cases.slice(0, 3).map((entry) => entry.expected[0].messages),
    [[], [], []],
  );
  assert.equal(source.cases.filter((entry) => entry.expected[0].warningCount > 0).length, 22);
});

await test("source-built cross-file SSR context preserves complete JSON and authored positive controls", () => {
  const source = corpus();
  const artifact = path.join(root, "target/differential/lint-cross-file-ssr-context.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = { schema: "vize.lint.ssr-context", version: 1, runs: [] };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ssr-context-"));
  try {
    const identity = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`), "utf8"),
    );
    evidence.build = { identity, receipt };
    evidence.corpusSha256 = digest(fs.readFileSync(path.join(fixtures, "source.json")));
    evidence.issueBodySha256 = source.body_sha256;
    persist();
    validateBuildReceipt(receipt, identity);
    const cli = path.join(root, identity.binaryPath);
    const env = { ...process.env, NO_COLOR: "1", VIZE_LOG: "warn" };
    delete env.FORCE_COLOR;
    function invoke(args) {
      const result = spawnSync(cli, args, {
        cwd: workspace,
        env,
        encoding: "utf8",
        timeout: 60_000,
        maxBuffer: 8 * 1024 * 1024,
      });
      return {
        command: cli,
        args,
        status: result.status,
        signal: result.signal,
        error: result.error ? { name: result.error.name, message: result.error.message } : null,
        stdout: result.stdout ?? "",
        stderr: result.stderr ?? "",
      };
    }
    function processChecks(run, statuses = [0]) {
      assert.equal(run.error, null, JSON.stringify(run));
      assert.equal(run.signal, null, JSON.stringify(run));
      assert.ok(statuses.includes(run.status), JSON.stringify(run));
      assert.equal(run.stderr, "", JSON.stringify(run));
    }
    const jsonArgs = (file, enabled, defaultHelp = false) => [
      "lint",
      file,
      "--no-config",
      "--preset",
      "incremental",
      "--format",
      "json",
      "--locale",
      "en",
      ...(defaultHelp ? [] : ["--help-level", "none"]),
      ...(enabled ? ["--cross-file"] : []),
    ];
    for (const entry of source.cases) {
      const input = fs.readFileSync(path.join(fixtures, entry.file));
      const target = path.join(workspace, entry.path);
      fs.writeFileSync(target, input);
      for (const enabled of [false, true]) {
        const run = invoke(jsonArgs(entry.path, enabled));
        const expected = enabled ? entry.expected : entry.offExpected;
        evidence.runs.push({
          input: entry,
          enabled,
          run,
          expected,
          afterSha256: digest(fs.readFileSync(target)),
        });
        persist();
        processChecks(run);
        assert.deepEqual(JSON.parse(run.stdout), expected, JSON.stringify(run));
        assert.deepEqual(fs.readFileSync(target), input, "lint must not alter authored source");
      }
      fs.unlinkSync(target);
    }

    // The original opinionated/plain command can emit unrelated preset findings.
    // Preserve its complete process/output, and assert only the reported cross-file acceptance.
    const originals = source.cases.slice(0, 3);
    for (const entry of originals) {
      fs.writeFileSync(
        path.join(workspace, entry.path),
        fs.readFileSync(path.join(fixtures, entry.file)),
      );
    }
    const plain = invoke(source.originalPlain.args);
    evidence.originalPlain = {
      oracleScope: "cross-file diagnostic absence only; no whole-plain output oracle",
      processScope: "0 or 1 are native lint outcomes; unrelated opinionated findings may fail lint",
      input: originals,
      run: plain,
      afterSha256: originals.map((entry) =>
        digest(fs.readFileSync(path.join(workspace, entry.path))),
      ),
    };
    persist();
    processChecks(plain, [0, 1]);
    for (const line of plain.stdout.split("\n")) {
      assert.doesNotMatch(line, /\bcross-file\b|vize:croquis\/cf\//, JSON.stringify(plain));
    }
    for (const entry of originals) {
      const target = path.join(workspace, entry.path);
      assert.deepEqual(fs.readFileSync(target), fs.readFileSync(path.join(fixtures, entry.file)));
      fs.unlinkSync(target);
    }

    const helpEntry = source.cases.find((entry) => entry.path === source.defaultHelp.path);
    assert.ok(helpEntry);
    const input = fs.readFileSync(path.join(fixtures, helpEntry.file));
    const target = path.join(workspace, helpEntry.path);
    fs.writeFileSync(target, input);
    const help = invoke(jsonArgs(helpEntry.path, true, true));
    evidence.defaultHelp = {
      input: helpEntry,
      run: help,
      expected: source.defaultHelp.expected,
      afterSha256: digest(fs.readFileSync(target)),
    };
    persist();
    processChecks(help);
    assert.deepEqual(JSON.parse(help.stdout), source.defaultHelp.expected, JSON.stringify(help));
    assert.match(source.defaultHelp.expected[0].messages[0].help, /!import\.meta\.env\.SSR/);
    assert.match(source.defaultHelp.expected[0].messages[0].help, /import\.meta\.client/);
    assert.deepEqual(fs.readFileSync(target), input);
    evidence.complete = {
      inputs: source.cases.length,
      jsonRuns: evidence.runs.length,
      plainRuns: 1,
      defaultHelpRuns: 1,
    };
    assert.deepEqual(evidence.complete, {
      inputs: 38,
      jsonRuns: 76,
      plainRuns: 1,
      defaultHelpRuns: 1,
    });
  } finally {
    persist();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
