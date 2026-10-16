import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import {
  baselineConfig,
  resolveBaselineRuntime,
} from "../../tools/support/compat/fixtures/lint-divergence-baseline.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const directory = path.join(root, "crates/vize_patina/tests/fixtures/this-in-template-reference");
const corpus = JSON.parse(fs.readFileSync(path.join(directory, "cases.json")));
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const requireFromBench = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));

function offset(source, line, column) {
  const lines = source.split("\n");
  return Buffer.byteLength(
    lines.slice(0, line - 1).join("\n") +
      (line > 1 ? "\n" : "") +
      lines[line - 1].slice(0, column - 1),
  );
}

test("this-in-template references replay the pinned rule on complete authored sources", async () => {
  const runtime = resolveBaselineRuntime();
  assert.equal(runtime.version, "10.9.2");
  for (const [name, identity] of Object.entries(corpus.providers)) {
    const manifest = requireFromBench.resolve(`${name}/package.json`);
    assert.equal(requireFromBench(manifest).version, identity.version);
    assert.equal(sha256(fs.readFileSync(manifest)), identity.manifestSha256);
    assert.equal(
      sha256(fs.readFileSync(requireFromBench.resolve(name))),
      identity.entrypointSha256,
    );
  }
  const plugin = path.dirname(requireFromBench.resolve("eslint-plugin-vue/package.json"));
  for (const relative of ["dist/rules/this-in-template.js", "dist/utils/js-reserved.js"])
    assert.deepEqual(
      fs.readFileSync(path.join(directory, "reference", `${relative}.fixture`)),
      fs.readFileSync(path.join(plugin, relative)),
    );
  const linter = new runtime.ESLint({
    cwd: root,
    overrideConfigFile: true,
    overrideConfig: baselineConfig(runtime, { "vue/this-in-template": "warn" }),
  });
  assert.equal(corpus.cases.length, 25);
  for (const fixture of corpus.cases) {
    assert.equal(sha256(Buffer.from(fixture.source)), fixture.sourceSha256, fixture.id);
    if (fixture.originalSource)
      assert.equal(fixture.sourceSha256, fixture.originalSource.sha256, fixture.id);
    const raw = await linter.lintText(fixture.source, { filePath: fixture.filename });
    const results = raw.map((result) => ({
      messages: result.messages,
      errorCount: result.errorCount,
      warningCount: result.warningCount,
      fatalErrorCount: result.fatalErrorCount,
      fixableErrorCount: result.fixableErrorCount,
      fixableWarningCount: result.fixableWarningCount,
    }));
    assert.deepEqual(results, fixture.reference, fixture.id);
    const ranges = results
      .flatMap((result) => result.messages)
      .map((message) => ({
        start: offset(fixture.source, message.line, message.column),
        end: offset(fixture.source, message.endLine, message.endColumn),
      }));
    assert.deepEqual(ranges, fixture.expected, fixture.id);
  }
});

test("the original Matrix findings and complete packet identities stay distinct from corrected references", () => {
  const original = JSON.parse(fs.readFileSync(path.join(directory, "original-matrix.json")));
  assert.equal(original.fullPackets.length, 6);
  assert(
    original.fullPackets.every(
      (packet) => packet.source.commitSha === "d2506ca6b5e31252973f05981829c0367cacf26b",
    ),
  );
  assert.equal(original.originalFindings.filter((row) => row.kind === "falsePositives").length, 46);
  assert.equal(original.originalFindings.filter((row) => row.kind === "falseNegatives").length, 51);
});
