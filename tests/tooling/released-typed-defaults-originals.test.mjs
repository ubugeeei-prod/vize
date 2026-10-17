import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { apiRoot, gitBlob } from "../../tools/support/release/typed_defaults/custody.mjs";
import { originalScenarios } from "../../tools/support/release/typed_defaults/contracts.mjs";
import {
  returnDirectory,
  returnScenarios,
} from "../../tools/support/release/typed_defaults/return_prefilter.mjs";
import {
  loadFixtures,
  originalSourcePaths,
  typedDirectory,
} from "../../tools/support/release/typed_defaults/fixtures.mjs";
import {
  inputObservation,
  verifyPacket,
} from "../../tools/support/release/typed_defaults/packet.mjs";
import { corpusFixture } from "./support/released-typed-defaults-corpus.mjs";

function temporary(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "typed-public-originals-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  return root;
}

function substitute(fixture, name, change) {
  const entry = fixture.tree.tree.find((entry) => entry.path === name);
  const original = fixture.rows.get(`${apiRoot}/git/blobs/${entry.sha}`);
  const bytes = Buffer.from(change(Buffer.from(original.content, "base64").toString("utf8")));
  const sha = gitBlob(bytes);
  entry.sha = sha;
  fixture.rows.set(`${apiRoot}/git/blobs/${sha}`, {
    sha,
    encoding: "base64",
    size: bytes.length,
    content: bytes.toString("base64"),
  });
}

await test("immutable H source must retain every authored fixture and all 51 n8n rules", async (t) => {
  const root = temporary(t),
    good = corpusFixture();
  const fixtures = await loadFixtures(good.head, path.join(root, "good"), good.read, good.treeSha);
  assert.equal(
    fixtures.authorities.filter(
      (entry) =>
        !entry.path.startsWith(returnDirectory) &&
        entry.path !== "tests/tooling/lint-return-prefilter.test.mjs",
    ).length,
    40,
  );
  assert.equal(
    fixtures.authorities.filter((entry) => entry.path.startsWith(returnDirectory)).length,
    17,
  );
  assert.equal(
    fixtures.authorities.filter(
      (entry) => entry.path === "tests/tooling/lint-return-prefilter.test.mjs",
    ).length,
    1,
  );
  const observerRole = "tests/tooling/lint-return-prefilter.test.mjs";
  assert.equal(
    fixtures.authorities.find((entry) => entry.path === observerRole).sourcePath,
    originalSourcePaths[observerRole],
  );
  const vectors = originalScenarios(fixtures);
  assert.equal(vectors.length, 280);
  assert.equal(returnScenarios(fixtures).length, 112);
  const mutations = [
    [
      "changed archived original observer",
      (f) => substitute(f, originalSourcePaths[observerRole], (text) => text + "\n"),
    ],
    [
      "wrong source tree",
      (f) => {
        f.tree.sha = "e".repeat(40);
      },
    ],
    [
      "incomplete source",
      (f) => {
        f.tree.truncated = true;
      },
    ],
    [
      "fixture symlink",
      (f) => {
        f.tree.tree.find((entry) => entry.path.endsWith("DrawerContent.vue.fixture")).mode =
          "120000";
      },
    ],
    [
      "changed authored default",
      (f) =>
        substitute(f, `${typedDirectory}/DrawerContent.vue.fixture`, (text) =>
          text.replace("direction: 'bottom'", "direction: 'top'"),
        ),
    ],
    [
      "changed 51-rule projection",
      (f) =>
        substitute(f, "tests/_fixtures/n8n-cli-adoption.json", (text) => {
          const projection = JSON.parse(text),
            first = Object.keys(projection.linter.rules)[0];
          projection.linter.rules[first] =
            projection.linter.rules[first] === "off" ? "error" : "off";
          assert.equal(Object.keys(projection.linter.rules).length, 51);
          return `${JSON.stringify(projection, null, 2)}\n`;
        }),
    ],
    [
      "changed original expectation",
      (f) =>
        substitute(f, `${typedDirectory}/cases.json`, (text) =>
          text.replace('"severity": "error"', '"severity": "warning"'),
        ),
    ],
  ];
  for (const [index, [name, mutate]] of mutations.entries()) {
    const fixture = corpusFixture();
    mutate(fixture);
    await assert.rejects(
      () =>
        loadFixtures(
          fixture.head,
          path.join(root, `rejected-${index}`),
          fixture.read,
          fixture.treeSha,
        ),
      undefined,
      name,
    );
  }
});

await test("original 56 + 80 + 144 whole vectors refuse source, configuration, warning and provider expectation drift", async (t) => {
  const root = temporary(t),
    fixture = corpusFixture();
  const originals = await loadFixtures(
    fixture.head,
    path.join(root, "originals"),
    fixture.read,
    fixture.treeSha,
  );
  originalScenarios(originals);
  const changes = [
    [
      "typed default severity",
      (f) => {
        f.typed.cases[0].diagnostics[0].severity = "warning";
      },
    ],
    [
      "Unicode original source",
      (f) => {
        f.typed.cases.find((entry) => entry.id === "unicode-crlf").source += "\n";
      },
    ],
    [
      "Vue version configuration",
      (f) => {
        f.first.configuredVueVersions[1] = "2.6";
      },
    ],
    [
      "original warning severity",
      (f) => {
        f.first.cases
          .find((entry) => entry.id === "full-modifiers")
          .diagnostics.find((finding) => finding.severity === "warning").severity = "error";
      },
    ],
    [
      "migration preset expectation",
      (f) => {
        f.second.cases[0].essentialDiagnostics = [];
      },
    ],
    [
      "n8n rule control",
      (f) => {
        const first = Object.keys(f.projection.linter.rules)[0];
        f.projection.linter.rules[first] =
          f.projection.linter.rules[first] === "error" ? "off" : "error";
      },
    ],
    [
      "whole diagnostic provider range",
      (f) => {
        const original = f.wholeJson;
        f.wholeJson = (entry, findings) => {
          const result = original(entry, findings);
          if (result[0].messages[0]) result[0].messages[0].endColumn++;
          return result;
        };
      },
    ],
  ];
  for (const [name, change] of changes) {
    const mutated = {
      ...originals,
      ...Object.fromEntries(
        ["typed", "first", "second", "projection"].map((key) => [
          key,
          structuredClone(originals[key]),
        ]),
      ),
    };
    change(mutated);
    assert.throws(() => originalScenarios(mutated), /whole behavior vectors changed/, name);
  }
});

await test("whole process verification refuses count-preserving JSON drift and retains removed source/config reads", (t) => {
  const root = temporary(t),
    source = path.join(root, "source.vue"),
    config = path.join(root, "vize.config.json");
  fs.writeFileSync(source, "<template/>\n");
  fs.writeFileSync(config, "{}\n");
  const expected = [
    {
      file: "source.vue",
      messages: [
        {
          severity: 1,
          line: 1,
          column: 2,
          endLine: 1,
          endColumn: 8,
          message: "original",
          ruleId: "vue/control",
          help: "original help",
        },
      ],
      errorCount: 0,
      warningCount: 1,
    },
  ];
  const packet = {
    error: null,
    signal: null,
    status: 0,
    expectedStatus: 0,
    expected,
    input: inputObservation(source, "<template/>\n"),
    configuration: inputObservation(config, "{}\n"),
  };
  const stdout = Buffer.from(JSON.stringify(expected));
  verifyPacket(structuredClone(packet), stdout, Buffer.alloc(0));
  for (const key of ["severity", "column", "endColumn", "message", "ruleId", "help"]) {
    const changed = structuredClone(expected);
    changed[0].messages[0][key] =
      typeof changed[0].messages[0][key] === "number" ? 99 : "different";
    assert.throws(
      () =>
        verifyPacket(
          structuredClone(packet),
          Buffer.from(JSON.stringify(changed)),
          Buffer.alloc(0),
        ),
      undefined,
      key,
    );
  }
  assert.throws(() =>
    verifyPacket({ ...structuredClone(packet), status: 1 }, stdout, Buffer.alloc(0)),
  );
  assert.throws(() =>
    verifyPacket(structuredClone(packet), stdout, Buffer.from("unexpected stderr")),
  );
  assert.throws(() => verifyPacket(structuredClone(packet), Buffer.from([0xff]), Buffer.alloc(0)));
  fs.unlinkSync(source);
  fs.unlinkSync(config);
  const removed = {
    ...packet,
    input: inputObservation(source, packet.input.text),
    configuration: inputObservation(config, packet.configuration.text),
  };
  assert.equal(removed.input.readError.name, "Error");
  assert.equal(removed.configuration.afterText, null);
  assert.throws(() => verifyPacket(removed, stdout, Buffer.alloc(0)), /disappeared/);
});
