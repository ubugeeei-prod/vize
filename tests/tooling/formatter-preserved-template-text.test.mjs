import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";
import { observeSemantics } from "./support/formatter-template-semantics.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtureRoot = path.join(
  root,
  "tests/_fixtures/differential/formatter/sfc-preserved-template-text-7871",
);
const original = fs.readFileSync(path.join(fixtureRoot, "App.vue.txt"), "utf8");
const originalReference = fs.readFileSync(path.join(fixtureRoot, "reference.expected.txt"), "utf8");

void test("source CLI preserves runtime text while formatting attributes", async (t) => {
  const identity = expectedBuildIdentity(root);
  validateBuildReceipt(
    JSON.parse(fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`))),
    identity,
  );
  assert.equal(original, originalReference);
  const project = fs.mkdtempSync(path.join(os.tmpdir(), "vize-preserved-template-text-"));
  t.after(() => fs.rmSync(project, { recursive: true, force: true }));
  fs.writeFileSync(
    path.join(project, "vize.config.json"),
    fs.readFileSync(path.join(fixtureRoot, "vize.config.json")),
  );
  const cases = [
    {
      id: "original",
      source: original,
      expected: originalReference,
      states: [{}],
      html: ['<button type="button">  two  spaces  </button>'],
    },
    {
      id: "formatted-bind",
      source: '<template>\n  <button  v-bind:title="label">  two  spaces  </button>\n</template>\n',
      expected: '<template>\n  <button :title="label">  two  spaces  </button>\n</template>\n',
      states: [{ label: "Label" }, { label: "日本語 & text" }],
      html: [
        '<button title="Label">  two  spaces  </button>',
        '<button title="日本語 &amp; text">  two  spaces  </button>',
      ],
    },
    {
      id: "inline-boundaries",
      source: "<template><p>  first\t<i>  nested  </i>\t last  </p></template>\n",
      expected: "<template><p>  first\t<i>  nested  </i>\t last  </p></template>\n",
      states: [{}],
      html: ["<p>  first\t<i>  nested  </i>\t last  </p>"],
    },
  ];
  const report = { identity, observations: [] };
  const reportPath = path.join(root, "target/differential/formatter-preserved-template-text.json");
  fs.mkdirSync(path.dirname(reportPath), { recursive: true });
  const persist = () => fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
  for (const fixture of cases) {
    fixture.file = "App.vue";
    const before = await observeSemantics(fixture.source, fixture, "preserve");
    const row = { id: fixture.id, before, reference: null, cli: [], after: [] };
    report.observations.push(row);
    persist();
    row.reference = await observeSemantics(fixture.expected, fixture, "preserve");
    assert.deepEqual(row.reference.states, before.states);
    assert.deepEqual(
      row.reference.states.map((state) => state.ssrHtml),
      fixture.html,
    );
    fs.writeFileSync(path.join(project, fixture.file), fixture.source);
    for (const mode of ["--write", "--write", "--write", "--check"]) {
      const output = spawnSync(path.join(root, identity.binaryPath), ["fmt", mode, fixture.file], {
        cwd: project,
        encoding: "utf8",
      });
      row.cli.push({ mode, status: output.status, stdout: output.stdout, stderr: output.stderr });
      persist();
      assert.equal(output.status, 0, output.stderr);
      assert.equal(output.stdout, "");
      const actual = fs.readFileSync(path.join(project, fixture.file), "utf8");
      assert.equal(actual, fixture.expected);
      const after = await observeSemantics(actual, fixture, "preserve");
      row.after.push(after);
      persist();
      assert.deepEqual(after.states, before.states);
    }
  }
});
