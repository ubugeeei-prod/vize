import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { offsetToPosition } from "./support/lsp/assertions.ts";
import { FakeAuthoredLspSession } from "./support/fake-authored-lsp-session.ts";
import { exerciseAuthoredImporterOverlay } from "./support/real-project-lsp-importer-overlay.ts";
import type { LspAuthoredOracle } from "./support/real-project-lsp-report.ts";

test("unsaved importer retarget and repair retain both complete dependency completion banks", async () => {
  const fixture = createFixture();
  try {
    assert.equal(await fixture.run(), 4);
    fixture.assertRestored();
    assert.equal(
      fixture.session.events.filter((event) => event.startsWith("textDocument/completion:")).length,
      2,
    );
    assert.equal(
      fixture.session.events.filter((event) => event.startsWith("textDocument/definition:")).length,
      2,
    );
    assert.ok(
      fixture.session.events.some((event) => event.startsWith("workspace/didCreateFiles:")),
    );
    assert.ok(
      fixture.session.events.some((event) => event.startsWith("workspace/didDeleteFiles:")),
    );
  } finally {
    fixture.dispose();
  }
});

for (const [name, retargeted, labels, failure] of [
  [
    "stale dirty dependency after retarget",
    true,
    ["v-if", "active", "vize-oracle-probe"],
    /retarget/,
  ],
  ["stale copied dependency after repair", false, ["v-if", "active"], /repair/],
  ["extra copied completion", true, ["v-if", "active", "phantom"], /retarget/],
  ["reordered dirty completions", false, ["active", "v-if", "vize-oracle-probe"], /repair/],
] as const) {
  test(`importer overlay rejects ${name} and restores the original buffer`, async () => {
    const fixture = createFixture();
    const request = fixture.session.request.bind(fixture.session);
    fixture.session.request = async (method, params) => {
      const response = await request(method, params);
      if (
        method === "textDocument/completion" &&
        fixture.importerText().includes("$OracleFeatureChild.vue") === retargeted
      )
        return labels.map((label) => ({ label }));
      return response;
    };
    try {
      await assert.rejects(fixture.run, failure);
      fixture.assertRestored();
    } finally {
      fixture.dispose();
    }
  });
}

test("importer overlay rejects a definition that remains on the dirty original dependency", async () => {
  const fixture = createFixture();
  const request = fixture.session.request.bind(fixture.session);
  fixture.session.request = async (method, params) =>
    method === "textDocument/definition"
      ? { uri: fixture.overlay.dependency.uri, range: { start: position, end: position } }
      : request(method, params);
  try {
    await assert.rejects(fixture.run, /Expected values to be strictly equal/);
    fixture.assertRestored();
  } finally {
    fixture.dispose();
  }
});

test("restored definition ranges are validated against the dirty dependency buffer", async () => {
  const fixture = createFixture();
  const request = fixture.session.request.bind(fixture.session);
  fixture.session.request = async (method, params) => {
    if (method === "textDocument/definition" && !fixture.importerText().includes("$Oracle")) {
      const position = offsetToPosition(
        fixture.overlay.changedDependencySource,
        fixture.overlay.changedDependencySource.length - 1,
      );
      return { uri: fixture.overlay.dependency.uri, range: { start: position, end: position } };
    }
    return request(method, params);
  };
  try {
    assert.equal(await fixture.run(), 4);
    fixture.assertRestored();
  } finally {
    fixture.dispose();
  }
});

test("importer cleanup keeps the primary failure when its diagnostic barrier also fails", async () => {
  const fixture = createFixture();
  const request = fixture.session.request.bind(fixture.session);
  const wait = fixture.session.waitForNotification.bind(fixture.session);
  const primary = new Error("primary completion failure");
  let failed = false;
  fixture.session.request = async (method, params) => {
    if (method === "textDocument/completion") {
      failed = true;
      throw primary;
    }
    return request(method, params);
  };
  fixture.session.waitForNotification = async (method, predicate) => {
    if (failed) throw new Error("cleanup diagnostic failure");
    return wait(method, predicate);
  };
  try {
    await assert.rejects(fixture.run, (error) => error === primary);
    fixture.assertRestored();
    assert.ok(
      fixture.session.events.some((event) => event.startsWith("workspace/didDeleteFiles:")),
    );
  } finally {
    fixture.dispose();
  }
});

test("importer overlay refuses an existing copy without deleting its authored bytes", async () => {
  const fixture = createFixture();
  fs.writeFileSync(fixture.copiedPath, "owned existing copy\n");
  try {
    await assert.rejects(fixture.run, /already exists/);
    assert.equal(fs.readFileSync(fixture.copiedPath, "utf8"), "owned existing copy\n");
    assert.deepEqual(fixture.session.events, []);
  } finally {
    fixture.dispose();
  }
});

const position = { line: 0, character: 0 };

function createFixture() {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-importer-overlay-"));
  const importerSource = `<script setup lang="ts">
import FeatureChild from './FeatureChild.vue'
</script>
<template><FeatureChild /></template>
`;
  const childSource = `<script setup lang="ts">
defineProps<{ active?: boolean }>()
</script>
`;
  const importerPath = path.join(workspace, "FeatureParent.vue");
  const childPath = path.join(workspace, "FeatureChild.vue");
  fs.writeFileSync(importerPath, importerSource);
  fs.writeFileSync(childPath, childSource);
  const oracle: LspAuthoredOracle = {
    componentBoundary: {
      componentFile: "FeatureChild.vue",
      completionItemCount: 2,
      completionItems: [
        { label: "v-if", rank: 0 },
        { label: "active", rank: 1 },
      ],
      dependencyEdit: {
        anchor: "active?: boolean",
        replacement: "active?: boolean;\n  vizeOracleProbe?: boolean\n",
        completionLabel: "vize-oracle-probe",
      },
      importerFile: "FeatureParent.vue",
      tagAnchor: "<FeatureChild ",
      tagName: "FeatureChild",
    },
    fileLifecycle: {
      copiedFile: "$OracleFeatureChild.vue",
      copiedImportSpecifier: "./$OracleFeatureChild.vue",
      originalImportSpecifier: "./FeatureChild.vue",
      renamedFile: "$OracleRenamedFeatureChild.vue",
      renamedImportSpecifier: "./$OracleRenamedFeatureChild.vue",
      markerInsertionAnchor: '<script setup lang="ts">\n',
      markerSymbol: "__vizeFileLifecycleMarker__",
    },
    templateBinding: {
      file: "Binding.vue",
      declarationAnchor: "title",
      usageAnchor: "{{ title }}",
      symbol: "title",
      hoverContains: ["string"],
      renameTo: "renamedTitle",
    },
  };
  const session = new FakeAuthoredLspSession(workspace);
  const dependency = { source: childSource, uri: pathToFileURL(childPath).href };
  const importer = { source: importerSource, uri: pathToFileURL(importerPath).href };
  const changedDependencySource = childSource.replace(
    "active?: boolean",
    oracle.componentBoundary.dependencyEdit.replacement,
  );
  session.seedDocument("FeatureChild.vue", changedDependencySource, 3);
  session.seedDocument("FeatureParent.vue", importerSource, 2);
  let importerText = importerSource;
  const notify = session.notify.bind(session);
  session.notify = (method, params) => {
    const payload = params as {
      textDocument?: { uri: string };
      contentChanges?: { text: string }[];
    };
    if (method === "textDocument/didChange" && payload.textDocument?.uri === importer.uri)
      importerText = payload.contentChanges![0]!.text;
    notify(method, params);
  };
  const tagOffset = importerSource.indexOf("FeatureChild />");
  const overlay = {
    importer,
    dependency,
    changedDependencySource,
    tagRange: {
      start: offsetToPosition(importerSource, tagOffset),
      end: offsetToPosition(importerSource, tagOffset + "FeatureChild".length),
    },
    completionPosition: offsetToPosition(importerSource, tagOffset + "FeatureChild ".length),
    baselineLabels: ["v-if", "active"],
    changedLabels: ["v-if", "active", "vize-oracle-probe"],
    initialVersion: 2,
  };
  const copiedPath = path.join(workspace, oracle.fileLifecycle.copiedFile);
  return {
    overlay,
    copiedPath,
    session,
    importerText: () => importerText,
    run: () => exerciseAuthoredImporterOverlay(session, workspace, oracle, overlay, () => 1_000),
    assertRestored: () => {
      assert.equal(importerText, importerSource);
      assert.equal(fs.readFileSync(importerPath, "utf8"), importerSource);
      assert.equal(fs.readFileSync(childPath, "utf8"), childSource);
      assert.equal(fs.existsSync(copiedPath), false);
    },
    dispose: () => fs.rmSync(workspace, { recursive: true, force: true }),
  };
}
