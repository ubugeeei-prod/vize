import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { apiRoot, gitBlob, sha256 } from "../../tools/support/release/typed_defaults/custody.mjs";
import {
  authenticateSetupFix,
  loadSetupFixtures,
  setupDirectory,
  setupMerge,
  setupOriginals,
  setupOriginalSourcePaths,
  setupScenarios,
} from "../../tools/support/release/typed_defaults/setup_emits.mjs";
import { originalScenarios } from "../../tools/support/release/typed_defaults/contracts.mjs";
import { loadFixtures } from "../../tools/support/release/typed_defaults/fixtures.mjs";
import { returnScenarios } from "../../tools/support/release/typed_defaults/return_prefilter.mjs";
import { corpusFixture } from "./support/released-typed-defaults-corpus.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
function temporary(t) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "setup-public-custody-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  return directory;
}
function setupFixture() {
  const head = "d".repeat(40),
    treeSha = "b".repeat(40),
    rows = new Map();
  const tree = { sha: treeSha, truncated: false, tree: [] };
  rows.set(`${apiRoot}/git/trees/${treeSha}?recursive=1`, tree);
  function add(name, bytes) {
    const sha = gitBlob(bytes);
    tree.tree.push({ path: name, type: "blob", mode: "100644", sha });
    rows.set(`${apiRoot}/git/blobs/${sha}`, {
      sha,
      encoding: "base64",
      size: bytes.length,
      content: bytes.toString("base64"),
    });
  }
  for (const [name, hash] of Object.entries(setupOriginals)) {
    const sourcePath = setupOriginalSourcePaths[name] ?? name;
    const bytes = fs.readFileSync(path.join(root, sourcePath));
    assert.equal(sha256(bytes), hash);
    add(sourcePath, bytes);
  }
  const corpus = JSON.parse(fs.readFileSync(path.join(root, setupDirectory, "cases.json")));
  for (const entry of corpus.cases) {
    const name = `${setupDirectory}/${entry.filename}.fixture`;
    add(name, fs.readFileSync(path.join(root, name)));
  }
  return {
    head,
    treeSha,
    rows,
    tree,
    read: async (url) => {
      assert(rows.has(url), `unplanned inert source read: ${url}`);
      return structuredClone(rows.get(url));
    },
  };
}

await test("separate setup357 refuses whole-vector mutation and preserves original392 contracts", async (t) => {
  const source = setupFixture();
  const fixtures = await loadSetupFixtures(source.head, temporary(t), source.read, source.treeSha);
  assert.equal(fixtures.authorities.length, 55);
  assert(fixtures.authorities.every((entry) => entry.source === source.head));
  const observerRole = "tests/tooling/lint-setup-emits-returns.test.mjs";
  assert.equal(
    fixtures.authorities.find((entry) => entry.path === observerRole).sourcePath,
    setupOriginalSourcePaths[observerRole],
  );
  assert.equal(setupScenarios(fixtures).length, 357);
  const original = corpusFixture();
  const legacy = await loadFixtures(original.head, temporary(t), original.read, original.treeSha);
  assert.equal(originalScenarios(legacy).length, 280);
  assert.equal(returnScenarios(legacy).length, 112);
  for (const [name, mutate] of [
    [
      "source trivia",
      (f) => {
        f.setup.cases[0].source += "\n";
      },
    ],
    [
      "severity",
      (f) => {
        f.setup.cases[0].diagnostics[0].severity = "warning";
      },
    ],
    [
      "range",
      (f) => {
        f.setup.cases[0].diagnostics[0].end++;
      },
    ],
    [
      "compiler ownership negative",
      (f) => {
        f.setup.cases.find((entry) => !entry.diagnostics.length).diagnostics =
          f.setup.cases[0].diagnostics;
      },
    ],
    [
      "off import control",
      (f) => {
        f.setup.cases.find((entry) => entry.disabledDiagnostics.length).disabledDiagnostics = [];
      },
    ],
    [
      "off rule identity",
      (f) => {
        f.setup.rules.pop();
      },
    ],
    [
      "n8n configuration",
      (f) => {
        f.projection.linter.rules[f.setup.rules[0]] = "error";
      },
    ],
  ]) {
    const changed = {
      ...fixtures,
      setup: structuredClone(fixtures.setup),
      projection: structuredClone(fixtures.projection),
    };
    mutate(changed);
    assert.throws(
      () => setupScenarios(changed),
      /original setup whole behavior vectors changed/,
      name,
    );
  }
});

await test("installed setup fixtures require complete H tree and original regular Git blobs", async (t) => {
  for (const [name, mutate] of [
    [
      "changed archived original observer",
      (v) => {
        const sourcePath =
          setupOriginalSourcePaths["tests/tooling/lint-setup-emits-returns.test.mjs"];
        const entry = v.tree.tree.find((entry) => entry.path === sourcePath);
        const row = v.rows.get(`${apiRoot}/git/blobs/${entry.sha}`);
        const bytes = Buffer.from(Buffer.from(row.content, "base64").toString() + "\n");
        const sha = gitBlob(bytes);
        entry.sha = sha;
        v.rows.set(`${apiRoot}/git/blobs/${sha}`, {
          sha,
          encoding: "base64",
          size: bytes.length,
          content: bytes.toString("base64"),
        });
      },
    ],
    [
      "foreign tree",
      (v) => {
        v.tree.sha = "e".repeat(40);
      },
    ],
    [
      "truncated tree",
      (v) => {
        v.tree.truncated = true;
      },
    ],
    [
      "missing case",
      (v) => {
        v.tree.tree.pop();
      },
    ],
    [
      "source symlink",
      (v) => {
        v.tree.tree[0].mode = "120000";
      },
    ],
    [
      "changed blob",
      (v) => {
        const row = v.rows.get(`${apiRoot}/git/blobs/${v.tree.tree[0].sha}`);
        row.content = Buffer.from("changed authored corpus").toString("base64");
      },
    ],
  ]) {
    const source = setupFixture();
    mutate(source);
    await assert.rejects(
      () => loadSetupFixtures(source.head, temporary(t), source.read, source.treeSha),
      undefined,
      name,
    );
  }
});

await test("installed setup357 requires actual own-repository merge2284 in authenticated H", async () => {
  const head = "c".repeat(40);
  const original = {
    fix: {
      merged: true,
      merge_commit_sha: setupMerge,
      base: { repo: { full_name: "ubugeeei-prod/vize" } },
      head: { repo: { full_name: "ubugeeei-prod/vize" } },
    },
    ancestry: {
      base_commit: { sha: setupMerge },
      merge_base_commit: { sha: setupMerge },
      status: "ahead",
    },
  };
  const reader = (value) => async (url) => {
    if (url === `${apiRoot}/pulls/8647`) return value.fix;
    assert.equal(url, `${apiRoot}/compare/${setupMerge}...${head}`);
    return value.ancestry;
  };
  assert.deepEqual(await authenticateSetupFix(head, reader(original)), original);
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
      "wrong comparison",
      (v) => {
        v.ancestry.base_commit.sha = head;
      },
    ],
    [
      "missing merged source",
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
    const changed = structuredClone(original);
    mutate(changed);
    await assert.rejects(() => authenticateSetupFix(head, reader(changed)), undefined, name);
  }
});
