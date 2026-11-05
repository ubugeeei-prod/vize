import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import type { ArtFileInfo } from "../types/art.js";
import { loadInlineArtCustomBlock } from "./inline-art-query.js";
import { createLoad, type VirtualModuleState } from "./virtual.js";

const moduleCode = "export default {};\n";

function inlineArt(file: string): ArtFileInfo {
  return {
    path: file,
    metadata: { title: "Registered", tags: [], status: "ready" },
    variants: [{ name: "Default", template: "<Self />", isDefault: true, skipVrt: false }],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
    isInline: true,
    componentPath: file,
    inlineArtBlockIndex: 1,
  };
}

function queryState(root: string, file: string): VirtualModuleState {
  return {
    basePath: "/__musea__",
    inlineArt: true,
    artFiles: new Map([[file, inlineArt(file)]]),
    resolvedPreviewCss: [],
    resolvedPreviewSetup: null,
    getConfigRoot: () => root,
    getScanRoots: () => [path.join(root, "src")],
    getVueVersion: () => 3,
    getServer: () => null,
    processArtFile: async () => {},
  };
}

void test("registered inline Art consumes only its exact native custom-block query", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-inline-query-"));
  try {
    const file = path.join(root, "src/Card.vue");
    const state = queryState(root, file);
    const queries = [
      "vue&type=art&index=1",
      "index=1&type=art&vue=",
      "vue&type=art&index=1&title=Registered",
      "vue&type=art&index=0",
      "vue&type=art&index=2",
      "vue&type=docs&index=1",
      "vue&type=template&index=1",
      "type=art&index=1",
      "vue=true&type=art&index=1",
      "vue&type=art",
      "vue&type=art&index=",
      "vue&type=art&index=-1",
      "vue&type=art&index=1.0",
      "vue&type=art&index=1suffix",
      "vue&type=art&index=01",
      "vue&type=art&index=9007199254740993",
      "vue&type=art&index=1&index=0",
      "vue&type=art&index=1&index=1",
      "vue&type=art&type=docs&index=1",
      "vue&type=art&type=art&index=1",
      "vue&vue&type=art&index=1",
    ];
    assert.deepEqual(
      queries.map((query) => loadInlineArtCustomBlock(state, `${file}?${query}`)),
      [moduleCode, moduleCode, moduleCode, ...Array(18).fill(null)],
    );
    assert.equal(loadInlineArtCustomBlock(state, file), null);
    assert.equal(loadInlineArtCustomBlock(state, `/@fs${file}?${queries[0]}`), moduleCode);
    assert.equal(createLoad(state)(`${file}?${queries[0]}`), moduleCode);
    assert.deepEqual(state.artFiles.get(file), inlineArt(file));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("disabled, unregistered, foreign and mismatched inline Art remain unclaimed", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "musea-inline-boundary-"));
  try {
    const root = path.join(directory, "project");
    const file = path.join(root, "src/Card.vue");
    const query = (source: string) => `${source}?vue&type=art&index=1`;
    const state = queryState(root, file);
    const foreign = path.join(directory, "foreign/Card.vue");
    state.artFiles.set(foreign, inlineArt(foreign));
    const separate = path.join(root, "src/Card.art.vue");
    state.artFiles.set(separate, inlineArt(separate));
    assert.deepEqual(
      [
        path.join(root, "src/Unknown.vue"),
        path.join(root, "excluded/Card.vue"),
        foreign,
        separate,
      ].map((source) => loadInlineArtCustomBlock(state, query(source))),
      [null, null, null, null],
    );
    state.inlineArt = false;
    assert.equal(loadInlineArtCustomBlock(state, query(file)), null);
    state.inlineArt = true;
    const mismatches: Partial<ArtFileInfo>[] = [
      { isInline: false },
      { path: foreign },
      { componentPath: foreign },
      { inlineArtBlockIndex: undefined },
      { inlineArtBlockIndex: 0 },
    ];
    assert.deepEqual(
      mismatches.map((mismatch) => {
        state.artFiles.set(file, { ...inlineArt(file), ...mismatch });
        return loadInlineArtCustomBlock(state, query(file));
      }),
      [null, null, null, null, null],
    );
    state.artFiles.set(file, inlineArt(file));
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.mkdirSync(path.dirname(foreign), { recursive: true });
    fs.writeFileSync(foreign, "<template><div /></template>");
    fs.symlinkSync(foreign, file);
    assert.equal(loadInlineArtCustomBlock(state, query(file)), null);
    state.getScanRoots = () => [path.dirname(foreign)];
    assert.equal(loadInlineArtCustomBlock(state, query(foreign)), moduleCode);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
