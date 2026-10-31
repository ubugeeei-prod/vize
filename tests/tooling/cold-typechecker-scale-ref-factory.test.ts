import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { test } from "node:test";
import ts from "typescript";
import {
  compareRefFactoryResults,
  refFactoryComparison,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-ref-factory.ts";
import {
  factoryCodeEdits,
  factoryForward,
} from "../../tools/benchmarks/scripts/ref-factory-parity.ts";

const oldFactory =
  "  type $RefInstanceFactory<_C> = 0 extends (1 & _C) ? (component: _C) => (props: any) => undefined : _C extends { __vizeResolveRef?: infer _F } ? _F extends (...args: any[]) => any ? (component: _C) => _F : (component: _C) => (props: any) => undefined : _C extends (props: any, ctx?: any, expose?: any, ...args: any[]) => any ? (component: _C) => _C : (component: _C) => (props: any) => undefined;\n";
const hash = (text: string) => createHash("sha256").update(text).digest("hex");
function fixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-factory-pair-")),
    app = path.join(root, "apps/web");
  fs.mkdirSync(path.join(app, "src"), { recursive: true });
  const source =
    "<script setup lang=\"ts\">\nimport Child from './Child.vue';\n</script>\n<template><Child /></template>\n";
  const child = "<template><div /></template>\n";
  fs.writeFileSync(path.join(app, "src/Probe.vue"), source);
  fs.writeFileSync(path.join(app, "src/Child.vue"), child);
  const prefix = "import Child from './Child.vue.ts';\n";
  const code = (factory: string) => {
    const start = Buffer.byteLength(prefix + factory) - 3;
    return prefix + factory + "const value=1;\n" + `// @vize-map: ${start}:${start + 14} -> 0:14\n`;
  };
  const result = (factory: string) => ({
    fileCount: 2,
    errorCount: 0,
    warningCount: 0,
    programs: [{ root: ".", unknown: { strict: true } }],
    files: [
      { file: "src/Child.vue", virtualTs: "export default {};\n", diagnostics: [] as string[] },
      { file: "src/Probe.vue", virtualTs: code(factory), diagnostics: [] as string[] },
    ],
  });
  const corpus = {
    root,
    app,
    entries: [
      { path: "apps/web/src/Child.vue", bytes: Buffer.byteLength(child), sha256: hash(child) },
      { path: "apps/web/src/Probe.vue", bytes: Buffer.byteLength(source), sha256: hash(source) },
    ],
  };
  const options = { corpus, oldFactory, edit: factoryCodeEdits, typescript: ts };
  return { root, source, options, before: result(oldFactory), after: result(factoryForward) };
}

test("whole packet accepts only the one source-backed factory and exact Vue import rewrite", () => {
  const data = fixture();
  try {
    const result = compareRefFactoryResults(data.before, data.after, data.options);
    assert.equal(result.completeFiles, 2);
    assert.equal(result.activeFiles, 1);
    assert.deepEqual(result.changed, [
      { file: "src/Probe.vue", sourceSha256: hash(data.source), importDelta: 3, edits: 2 },
    ]);
  } finally {
    fs.rmSync(data.root, { recursive: true, force: true });
  }
});

test("unknown fields, diagnostics, helper bytes, import ownership, foreign IDs and incomplete rosters refuse", () => {
  const data = fixture();
  try {
    for (const poison of [
      (packet: typeof data.after) => {
        packet.files[1].diagnostics.push("error:1:1 [TS2322] changed");
      },
      (packet: typeof data.after) => {
        packet.programs[0].unknown.strict = false;
      },
      (packet: typeof data.after) => {
        packet.files[1].virtualTs += "\nforeign";
      },
      (packet: typeof data.after) => {
        packet.files[1].virtualTs += factoryForward;
      },
      (packet: typeof data.after) => {
        packet.files.reverse();
      },
      (packet: typeof data.after) => {
        packet.files.pop();
      },
      (packet: typeof data.after) => {
        packet.files[0].file = "../../outside.vue";
      },
    ]) {
      const changed = structuredClone(data.after);
      poison(changed);
      assert.throws(() => compareRefFactoryResults(data.before, changed, data.options));
    }
    for (const extra of [
      "import Other from './Other.vue.ts';\n",
      "import type {X} from 'vize/virtual-ts';\n",
      "type Extra=import('./hidden.ts');\n",
    ]) {
      const before = structuredClone(data.before),
        after = structuredClone(data.after);
      before.files[1].virtualTs = extra + before.files[1].virtualTs;
      after.files[1].virtualTs = extra + after.files[1].virtualTs;
      assert.throws(() => compareRefFactoryResults(before, after, data.options));
    }
    fs.appendFileSync(path.join(data.options.corpus.app, "src/Probe.vue"), "<!-- changed -->");
    assert.throws(() => compareRefFactoryResults(data.before, data.after, data.options));
  } finally {
    fs.rmSync(data.root, { recursive: true, force: true });
  }
});

test("sources without this migration retain the existing strict comparison path", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-factory-selection-"));
  try {
    assert.equal(await refFactoryComparison(root, root, root), null);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
