import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { createFormatterChangeEvidence } from "../../legacy-tools/fixtures/tool-matrix-formatter.mjs";
import {
  collectFormatterWriteEvidence,
  collectProjectVueFiles,
  resolveGlyphLaunch,
  withFormattedWorkspace,
} from "../../legacy-tools/fixtures/glyph-corpus.mjs";

type CorpusProject = {
  id: string;
  fixtureDir: string;
  hydrated: boolean;
  vueGlobs: string[];
};

test("glyph corpus write evidence includes hidden Vue inputs", () => {
  const project = makeSyntheticProject(
    [
      ["docs/.vitepress/theme/Foo.vue", "<template><p>foo</p></template>\n"],
      ["packages/docs/.vuepress/theme/Layout.vue", "<template><p>layout</p></template>\n"],
    ],
    ["**/*.vue"],
  );
  try {
    const files = collectProjectVueFiles(project) as string[];
    assert.deepEqual(files, [
      "docs/.vitepress/theme/Foo.vue",
      "packages/docs/.vuepress/theme/Layout.vue",
    ]);
    const launch = resolveGlyphLaunch();
    withFormattedWorkspace(project, files, launch, ({ workspaceDir }) => {
      assert.deepEqual(
        collectFormatterWriteEvidence(project, files, workspaceDir),
        createFormatterChangeEvidence(2, files),
      );
    });
  } finally {
    fs.rmSync(project.fixtureDir, { recursive: true, force: true });
  }
});

function makeSyntheticProject(files: Array<[string, string]>, vueGlobs: string[]): CorpusProject {
  const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-corpus-"));
  for (const [file, content] of files) {
    fs.mkdirSync(path.dirname(path.join(fixtureDir, file)), { recursive: true });
    fs.writeFileSync(path.join(fixtureDir, file), content);
  }
  return {
    id: "synthetic-hidden-idempotence",
    fixtureDir,
    hydrated: true,
    vueGlobs,
  };
}
