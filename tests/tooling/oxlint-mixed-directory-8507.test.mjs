import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  expectedRow,
  expectedStockVueRow,
} from "../../npm/oxlint/src/test-support/mixed-directory-8507-assertions.mjs";

const root = fileURLToPath(new URL("../..", import.meta.url));
const corpus = path.join(root, "tests/_fixtures/differential/lint/oxlint-mixed-directory-8507");
const read = (file) => fs.readFileSync(path.join(corpus, file), "utf8");

await test("the complete reporter files retain their original no-Git reproduction", () => {
  const issue = JSON.parse(read("issue.json"));
  assert.equal(read("issue.md"), issue.body);
  assert.equal(issue.number, 8507);
  assert.equal(issue.author.login, "AndreyYolkin");
  const shell = issue.body.match(/```sh\n([\s\S]*?)\n```/u)[1];
  for (const file of ["package.json", ".oxlintrc.json"]) {
    const escaped = file.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    const pattern = new RegExp(`cat > ${escaped} <<'JSON'\\n([\\s\\S]*?)\\nJSON`, "u");
    assert.equal(read(file), shell.match(pattern)[1] + "\n");
  }
  for (const file of ["app/index.html", "app/App.vue"]) {
    const source = shell.split("\n").find((line) => line.endsWith(` > ${file}`));
    assert.ok(source.startsWith("printf '"));
    const original = source.slice("printf '".length, source.lastIndexOf("' >"));
    assert.equal(read(file), original.replace(/\\n/gu, "\n"));
  }
  assert.ok(!shell.includes("git init"));
  assert.equal(JSON.parse(read("controls.json")).gitInit, false);
});

await test("the exact 1.81 source ledger covers all fourteen pinned original renderer files", () => {
  const evidence = JSON.parse(
    fs.readFileSync(path.join(root, "docs/acceptance/oxlint-181-capability-source.json")),
  );
  assert.equal(evidence.official181SourceCommit, "0b4e2e67f4193e7ebfcc64982275eb583ae82c83");
  assert.equal(evidence.official181Tag, "oxlint_v1.81.0");
  const vendor = path.join(root, "vendor/oxlint_html_renderer186");
  const pinned = JSON.parse(fs.readFileSync(path.join(vendor, "UPSTREAM.json")));
  assert.equal(evidence.wholeKernelSourceProof.length, 14);
  assert.deepEqual(
    Object.fromEntries(
      evidence.wholeKernelSourceProof.map((file) => {
        const relative = file.path.slice("crates/oxc_diagnostics/src/".length);
        assert.equal(file.official181GitBlobExact, true);
        assert.equal(file.existing186PinnedSourceExact, true);
        return [relative, file.sha256];
      }),
    ),
    pinned.files,
  );
});

await test("stock's extracted-script fallback preserves the distinct authored Vue location", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-stock-vue-8507-"));
  try {
    fs.mkdirSync(path.join(cwd, "app"));
    fs.copyFileSync(path.join(corpus, "controls/App-error.vue"), path.join(cwd, "app/App.vue"));
    fs.copyFileSync(path.join(corpus, "app/index.html"), path.join(cwd, "app/index.html"));
    const finding = JSON.parse(read("controls.json")).cases.find(
      (fixture) => fixture.name === "mixed-directory-keeps-vue-error",
    ).expectedDiagnostics[0];
    const original = expectedRow(finding, cwd);
    assert.deepEqual(original, {
      message: "Duplicate attribute 'id'\n    Help:\n      Remove the duplicate attribute",
      code: "vize(vue/no-duplicate-attributes)",
      severity: "error",
      filename: "app/App.vue",
      labels: [{ span: { offset: 80, length: 11, line: 6, column: 17 } }],
    });
    assert.deepEqual(expectedStockVueRow(finding, cwd), {
      ...original,
      message:
        "Duplicate attribute 'id' (at <template>:6:17)\n    Help:\n      Remove the duplicate attribute",
      labels: [{ span: { offset: 26, length: 0, line: 2, column: 2 } }],
    });
    assert.throws(() => expectedStockVueRow({ ...finding, path: "app/index.html" }, cwd));
  } finally {
    fs.rmSync(cwd, { recursive: true, force: true });
  }
});
