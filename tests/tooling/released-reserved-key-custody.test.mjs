import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { test } from "node:test";
import { apiRoot, gitBlob, sha256 } from "../../tools/support/release/typed_defaults/custody.mjs";
import {
  loadReservedFixtures,
  reservedContract,
  reservedManifest,
} from "../../tools/support/release/typed_defaults/reserved_fixtures.mjs";
import { reservedNormalized } from "../../tools/support/release/typed_defaults/reserved_scenarios.mjs";

const fixture = fileURLToPath(
  new URL("../_fixtures/differential/lint/released-reserved-8090/", import.meta.url),
);
const read = (name) => fs.readFileSync(path.join(fixture, name));
const json = (name) => JSON.parse(read(name));
const packed = (name) => JSON.parse(gunzipSync(read(name)));
function provider(t) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "reserved-inert-provider-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const source = packed("inert-provider.json.gz"),
    head = "d".repeat(40),
    treeSha = "b".repeat(40);
  const rows = new Map(),
    tree = { sha: treeSha, truncated: false, tree: [] };
  for (const entry of source.bodies) {
    const bytes = Buffer.from(entry.base64, "base64"),
      blob = gitBlob(bytes);
    tree.tree.push({ path: entry.path, sha: blob, mode: "100644", type: "blob" });
    rows.set(`${apiRoot}/git/blobs/${blob}`, {
      sha: blob,
      size: bytes.length,
      encoding: "base64",
      content: entry.base64,
    });
  }
  rows.set(`${apiRoot}/git/trees/${treeSha}?recursive=1`, tree);
  return {
    directory,
    head,
    treeSha,
    rows,
    tree,
    read: async (url) => {
      assert(rows.has(url), "inert provider only permits declared own-repository requests");
      return structuredClone(rows.get(url));
    },
  };
}

await test("separate443 retains whole literal vectors and immutable old-public RED packets", () => {
  assert.equal(
    sha256(read("old-public-authority.json")),
    "36af55405810dd2d529c356653f9fc714c86374ac56cd8d094f3392e1d57d60e",
  );
  const authority = json("old-public-authority.json");
  assert.equal(authority.acceptance, false);
  for (const [name, hash] of Object.entries(authority.files))
    assert.equal(sha256(read(name)), hash, name);
  const before = packed("original-public-red.json.gz"),
    initial = packed("initial-old-public-red.json.gz");
  assert.equal(before.version, 1);
  assert.equal(before.acceptance, false);
  assert.equal(before.versionObservation.stdout, "vize 0.443.0\n");
  assert.equal(initial.version.stdout, before.versionObservation.stdout);
  assert.equal(before.runs.length, 443);
  assert.equal(before.refused, 163);
  assert.equal(before.runs.filter((r) => r.matchesCorrectedSuccessor).length, 280);
  assert.equal(before.contract.sha256, reservedContract.sha256);
  assert.deepEqual(before.options, authority.publicOptions);
  assert.equal(before.bareAncestry.status, "diverged");
  assert.notEqual(
    before.bareAncestry.merge_base_commit.sha,
    "19e8fa700d3c79631166bcaed4e3cfe65dda1903",
  );
  before.runs.forEach((run, index) => {
    for (const key of [
      "suite",
      "id",
      "argv",
      "config",
      "input",
      "status",
      "error",
      "signal",
      "stdout",
      "stderr",
      "expected",
      "expectedStatus",
      "actual",
      "matchesCorrectedSuccessor",
      "raw",
    ])
      assert.deepEqual(run[key], initial.runs[index][key]);
    assert.equal(run.error, null);
    assert.equal(run.signal, null);
    assert.equal(run.stderr, "");
    assert.equal(sha256(run.input.text), run.input.sha256);
    assert.deepEqual(JSON.parse(run.stdout), run.actual);
    assert.equal(Buffer.from(run.raw.stdoutBase64, "base64").toString(), run.stdout);
    assert.equal(sha256(run.stdout), run.raw.stdoutSha256);
    assert.equal(
      run.matchesCorrectedSuccessor,
      JSON.stringify(run.actual) === JSON.stringify(run.expected) &&
        run.status === run.expectedStatus,
    );
  });
  assert.deepEqual(
    json("source-equivalence.json").qualified.map(({ processes }) => processes),
    [113, 161, 169],
  );
});

await test("all97 original H authorities build443 independently bound whole routes", async (t) => {
  const f = provider(t),
    loaded = await loadReservedFixtures(f.head, f.directory, f.read, f.treeSha);
  assert.equal(loaded.authorities.length, 97);
  assert.equal(new Set(loaded.authorities.map((a) => a.path)).size, 97);
  assert(loaded.authorities.every((a) => a.source === f.head));
  const normalized = reservedNormalized(loaded.vectors, loaded.wholeJson);
  const before = packed("original-public-red.json.gz");
  normalized.forEach((entry, index) => {
    const packet = before.runs[index];
    assert.equal(entry.id, packet.id);
    assert.equal(entry.suite, packet.suite);
    assert.equal(entry.source, packet.input.text);
    assert.equal(entry.config, packet.config);
    assert.deepEqual(entry.expected, packet.expected);
    assert.equal(entry.expectedStatus, packet.expectedStatus);
  });
  const messages = (vector) => vector.expected[0].messages;
  const mutations = [
    [
      "source",
      (n) => {
        n[0].source += "\n";
      },
    ],
    [
      "config",
      (n) => {
        n[6].config = "{}\n";
      },
    ],
    [
      "positive severity",
      (n) => {
        messages(n.find((v) => messages(v).some((d) => d.severity === 2))).find(
          (d) => d.severity === 2,
        ).severity = 1;
      },
    ],
    [
      "positive span",
      (n) => {
        messages(n.find((v) => messages(v).length))[0].endColumn++;
      },
    ],
    [
      "negative finding",
      (n) => {
        const positive = messages(n.find((v) => messages(v).length))[0];
        messages(n.find((v) => !messages(v).length)).push(positive);
      },
    ],
    [
      "target-off companion",
      (n) => {
        messages(n.find((v) => v.config?.includes('"off"') && messages(v).length)).pop();
      },
    ],
    [
      "configured n8n order",
      (n) => {
        messages(
          n.find(
            (v) =>
              v.config &&
              messages(v).some((d) => d.ruleId === "vue/sfc-element-order" && d.severity === 2),
          ),
        ).find((d) => d.ruleId === "vue/sfc-element-order").severity = 1;
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const changed = structuredClone(normalized);
    mutate(changed);
    assert.notEqual(sha256(JSON.stringify(changed)), reservedContract.sha256, name);
  }
});

await test("reserved443 refuses foreign, incomplete, duplicate or changed H blob transport", async (t) => {
  const mutations = [
    [
      "foreign tree",
      (f) => {
        f.tree.sha = "f".repeat(40);
      },
    ],
    [
      "truncated tree",
      (f) => {
        f.tree.truncated = true;
      },
    ],
    [
      "missing authority",
      (f) => {
        f.tree.tree.shift();
      },
    ],
    [
      "duplicate authority",
      (f) => {
        f.tree.tree.push(structuredClone(f.tree.tree[0]));
      },
    ],
    [
      "symlink authority",
      (f) => {
        f.tree.tree[0].mode = "120000";
      },
    ],
    [
      "wrong blob size",
      (f) => {
        f.rows.get(`${apiRoot}/git/blobs/${f.tree.tree[0].sha}`).size++;
      },
    ],
    [
      "substituted blob",
      (f) => {
        f.rows.get(`${apiRoot}/git/blobs/${f.tree.tree[0].sha}`).content =
          Buffer.from("{}").toString("base64");
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const f = provider(t);
    mutate(f);
    await assert.rejects(
      () => loadReservedFixtures(f.head, f.directory, f.read, f.treeSha),
      undefined,
      name,
    );
  }
  assert.equal(reservedManifest.authorities.length, 97);
});
