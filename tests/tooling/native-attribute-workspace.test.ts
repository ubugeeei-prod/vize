import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { projectViteConfigSource } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";
import { sha256 } from "../differential/harness.mjs";
import { nativeAttributeWorkspace } from "./support/lsp/native-attribute-hover-capture.ts";
import { loadHoverCorpus } from "./support/lsp/native-attribute-hover-contract.ts";

const fixtureRoot = fileURLToPath(
  new URL("../_fixtures/differential/lsp/native-attribute-hover-range-original", import.meta.url),
);

test("all four native hover projects preserve the twelve original requests and policy through real current carriers", async () => {
  const original = fs.readFileSync(`${fixtureRoot}/manifest.json`);
  const { corpus } = loadHoverCorpus(fixtureRoot);
  assert.equal(corpus.sessions.length, 4);
  assert.equal(
    corpus.sessions.reduce((count, session) => count + session.requests.length, 0),
    12,
  );
  for (const session of corpus.sessions) {
    const workspace = nativeAttributeWorkspace(
      fixtureRoot,
      corpus,
      session,
      "/original/provider/tsc",
    );
    assert.deepEqual(Object.keys(workspace.files), [
      "Field.vue",
      "tsconfig.json",
      "package.json",
      "vize.config.json.fixture",
      "vite.config.mjs",
    ]);
    assert.deepEqual(
      workspace.files["Field.vue"],
      fs.readFileSync(`${fixtureRoot}/${session.file.path}`),
    );
    const native = workspace.files["vize.config.json.fixture"];
    assert.deepEqual(JSON.parse(native.toString()), {
      lsp: corpus.initialization.options,
      typeChecker: { corsaPath: "/original/provider/tsc" },
    });
    assert.equal(workspace.configuration.original.bytesBase64, native.toString("base64"));
    assert.equal(workspace.configuration.original.sha256, sha256(native));
    const current = workspace.files["vite.config.mjs"];
    assert.equal(workspace.configuration.current.bytesBase64, current.toString("base64"));
    assert.equal(workspace.configuration.current.sha256, sha256(current));
    const { default: exported } = await import(
      `data:text/javascript;base64,${current.toString("base64")}`
    );
    assert.deepEqual(await projectViteConfigSource(exported), {
      languageServer: corpus.initialization.options,
      typeChecker: { jsxTypecheck: false, corsaPath: "/original/provider/tsc" },
    });
  }
  assert.deepEqual(fs.readFileSync(`${fixtureRoot}/manifest.json`), original);
});
