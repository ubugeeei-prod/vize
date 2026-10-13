import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { apiRoot } from "../../tools/support/release/typed_defaults/custody.mjs";
import { originalScenarios } from "../../tools/support/release/typed_defaults/contracts.mjs";
import { loadFixtures } from "../../tools/support/release/typed_defaults/fixtures.mjs";
import {
  authenticateReturnFix,
  returnMerge,
  returnScenarios,
} from "../../tools/support/release/typed_defaults/return_prefilter.mjs";
import { corpusFixture } from "./support/released-typed-defaults-corpus.mjs";

function temporary(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "return-public-originals-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  return root;
}

await test("112 additional return controls reject whole-vector drift while original 280 stay fixed", async (t) => {
  const fixture = corpusFixture();
  const fixtures = await loadFixtures(fixture.head, temporary(t), fixture.read, fixture.treeSha);
  const originals = originalScenarios(fixtures);
  const returns = returnScenarios(fixtures);
  assert.equal(originals.length, 280);
  assert.equal(returns.length, 112);
  for (const [name, mutate] of [
    [
      "source trivia",
      (f) => {
        f.returns.cases[0].source += "\n";
      },
    ],
    [
      "return severity",
      (f) => {
        f.returns.cases[0].diagnostics[0].severity = "warning";
      },
    ],
    [
      "return range",
      (f) => {
        f.returns.cases[0].diagnostics[0].end++;
      },
    ],
    [
      "negative control",
      (f) => {
        f.returns.cases.find((entry) => entry.diagnostics.length === 0).diagnostics =
          f.returns.cases[0].diagnostics;
      },
    ],
    [
      "explicit off rules",
      (f) => {
        f.returns.rules.pop();
      },
    ],
    [
      "n8n control",
      (f) => {
        f.projection.linter.rules[f.returns.rules[0]] = "error";
      },
    ],
  ]) {
    const changed = {
      ...fixtures,
      returns: structuredClone(fixtures.returns),
      projection: structuredClone(fixtures.projection),
    };
    mutate(changed);
    assert.throws(
      () => returnScenarios(changed),
      /original return whole behavior vectors changed/,
      name,
    );
    assert.deepEqual(originalScenarios({ ...changed, projection: fixtures.projection }), originals);
  }
});

await test("published return acceptance requires the actual protected merge in H ancestry", async () => {
  const head = "c".repeat(40);
  const original = {
    fix: {
      merged: true,
      merge_commit_sha: returnMerge,
      base: { repo: { full_name: "ubugeeei-prod/vize" } },
      head: { repo: { full_name: "ubugeeei-prod/vize" } },
    },
    ancestry: {
      base_commit: { sha: returnMerge },
      merge_base_commit: { sha: returnMerge },
      status: "ahead",
    },
  };
  function reader(value) {
    return async (url) => {
      if (url === `${apiRoot}/pulls/8643`) return value.fix;
      assert.equal(url, `${apiRoot}/compare/${returnMerge}...${head}`);
      return value.ancestry;
    };
  }
  assert.deepEqual(await authenticateReturnFix(head, reader(original)), original);
  for (const [name, mutate] of [
    [
      "unmerged",
      (v) => {
        v.fix.merged = false;
      },
    ],
    [
      "wrong merge",
      (v) => {
        v.fix.merge_commit_sha = head;
      },
    ],
    [
      "foreign base",
      (v) => {
        v.fix.base.repo.full_name = "other/vize";
      },
    ],
    [
      "foreign head",
      (v) => {
        v.fix.head.repo.full_name = "other/vize";
      },
    ],
    [
      "wrong comparison base",
      (v) => {
        v.ancestry.base_commit.sha = head;
      },
    ],
    [
      "missing merged fix",
      (v) => {
        v.ancestry.merge_base_commit.sha = head;
      },
    ],
    [
      "diverged source",
      (v) => {
        v.ancestry.status = "diverged";
      },
    ],
  ]) {
    const value = structuredClone(original);
    mutate(value);
    await assert.rejects(() => authenticateReturnFix(head, reader(value)), undefined, name);
  }
});
