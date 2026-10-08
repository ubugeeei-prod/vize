import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { materializeContent } from "../../docs/scripts/materialize-content.ts";

await test("native renderer input preserves route/source bytes and replaces only generated catalogues", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-docs-content-"));
  const put = (path: string, bytes: string | Buffer) => {
    mkdirSync(dirname(join(root, path)), { recursive: true });
    writeFileSync(join(root, path), bytes);
  };
  try {
    const originals = {
      "index.md": "---\ntitle: Vize\n---\n\n# Vize\n",
      "ja/guide/setup.md": "# 設定\r\n\r\nそのまま保つ。\r\n",
      "rules/reference/vue-probe.md": "# Probe\n\n```vue\n<div>  </div>\n```\n",
      "media/probe.png": Buffer.from([0, 255, 13, 10]),
      "generated/other.md": "An unrelated generated page stays at its route\n",
    };
    for (const [path, bytes] of Object.entries(originals)) put(`content/${path}`, bytes);
    put("content/rules/all.md", "English source index\n");
    put("content/ja/rules/all.md", "日本語の索引\n");
    const catalogues = {
      en: "# All rules\n\nFull English examples\n",
      ja: "# 全ルール\n\n完全な例\n",
    };
    for (const [locale, text] of Object.entries(catalogues))
      put(`content/generated/rules/${locale}/all.md`, text);
    put(".generated/content/stale.md", "Previous build must not survive\n");
    const staged = materializeContent(root);
    assert.equal(staged, join(root, ".generated/content"));
    for (const [path, bytes] of Object.entries(originals))
      assert.deepEqual(
        readFileSync(join(staged, path)),
        typeof bytes === "string" ? Buffer.from(bytes) : Buffer.from(bytes),
        path,
      );
    assert.equal(readFileSync(join(staged, "rules/all.md"), "utf8"), catalogues.en);
    assert.equal(readFileSync(join(staged, "ja/rules/all.md"), "utf8"), catalogues.ja);
    for (const locale of ["en", "ja"])
      assert.throws(() => readFileSync(join(staged, `generated/rules/${locale}/all.md`)), {
        code: "ENOENT",
      });
    assert.throws(() => readFileSync(join(staged, "stale.md")), { code: "ENOENT" });
    assert.equal(
      readFileSync(join(root, "content/rules/all.md"), "utf8"),
      "English source index\n",
    );
    assert.equal(readFileSync(join(root, "content/ja/rules/all.md"), "utf8"), "日本語の索引\n");
    rmSync(join(root, "content/generated/rules/ja/all.md"));
    assert.throws(() => materializeContent(root), { code: "ENOENT" });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
