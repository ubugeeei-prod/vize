import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../..", import.meta.url));
const corpus = path.join(root, "tests/_fixtures/differential/lint/oxlint-mixed-directory-8507");
const read = (file) => fs.readFileSync(path.join(corpus, file), "utf8");

test("the complete reporter files retain their original no-Git reproduction", () => {
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

test("the exact 1.81 source ledger covers all fourteen pinned original renderer files", () => {
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
