import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { captureLspResourceCorpus } from "../../tools/support/compat/davinci/lib/lsp-resource-corpus.ts";

test("paired LSP inputs retain exact authored bytes and actual dependency identities", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-corpus-"));
  try {
    const backend = path.join(root, "checker");
    const vue = path.join(root, "vue");
    fs.mkdirSync(vue);
    fs.writeFileSync(backend, "original backend");
    fs.writeFileSync(path.join(vue, "package.json"), '{"version":"test-1"}');
    const dependencies = { corsaPath: backend, vuePath: vue };
    const workspaces = ["base", "head"].map((role) => path.join(root, role));
    for (const workspace of workspaces) {
      fs.mkdirSync(path.join(workspace, "src"), { recursive: true });
      fs.mkdirSync(path.join(workspace, "node_modules"));
      fs.writeFileSync(path.join(workspace, "src", "App.vue"), "<template>🌋𐐀</template>\r\n");
      fs.writeFileSync(path.join(workspace, "tsconfig.json"), "{}\n");
      fs.writeFileSync(
        path.join(workspace, "node_modules", "ignored.ts"),
        path.basename(workspace),
      );
    }
    const baseline = captureLspResourceCorpus(workspaces[0], dependencies);
    assert.deepEqual(captureLspResourceCorpus(workspaces[1], dependencies), baseline);
    assert.equal(baseline.sfcFiles, 1);
    assert.deepEqual(
      baseline.entries.map((entry) => entry.path),
      ["src/App.vue", "tsconfig.json"],
    );
    assert.equal(baseline.sourceBytes, Buffer.byteLength("<template>🌋𐐀</template>\r\n{}\n"));
    fs.writeFileSync(path.join(workspaces[1], "src", "App.vue"), "<template>🌋𐐀</template>\n");
    const changed = captureLspResourceCorpus(workspaces[1], dependencies);
    assert.notEqual(changed.sha256, baseline.sha256, "line-ending changes are authored changes");
    assert.equal(changed.backendSha256, baseline.backendSha256);
    fs.writeFileSync(backend, "different backend");
    const backendChanged = captureLspResourceCorpus(workspaces[0], dependencies);
    assert.notEqual(backendChanged.backendSha256, baseline.backendSha256);
    assert.equal(backendChanged.sha256, baseline.sha256);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
