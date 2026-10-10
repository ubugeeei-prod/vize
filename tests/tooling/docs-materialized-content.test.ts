import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { CATALOGUE_SOURCES, materializeContent } from "../../docs/scripts/materialize-content.ts";

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
    assert.equal(
      CATALOGUE_SOURCES.length,
      66,
      "five complete catalogue locales and eleven retained subgroups",
    );
    const catalogues = CATALOGUE_SOURCES.map((source) => {
      const match = source.match(/^generated\/rules\/([^/]+)\/(.+)$/);
      assert(match, "derivative follows its existing route convention");
      const [, locale, file] = match;
      const target = `${locale === "en" ? "" : `${locale}/`}rules/${file}`;
      const text = "# Complete " + source + "\n\n```vue\n<div>  </div>\n```\n";
      put(`content/${target}`, `Source index ${target}\n`);
      put(`content/${source}`, text);
      return { source, target, text };
    });
    put(".generated/content/stale.md", "Previous build must not survive\n");
    const staged = materializeContent(root);
    assert.equal(staged, join(root, ".generated/content"));
    for (const [path, bytes] of Object.entries(originals))
      assert.deepEqual(
        readFileSync(join(staged, path)),
        typeof bytes === "string" ? Buffer.from(bytes) : Buffer.from(bytes),
        path,
      );
    for (const { source, target, text } of catalogues) {
      assert.equal(readFileSync(join(staged, target), "utf8"), text);
      assert.throws(() => readFileSync(join(staged, source)), { code: "ENOENT" });
      assert.equal(
        readFileSync(join(root, `content/${target}`), "utf8"),
        `Source index ${target}\n`,
      );
    }
    assert.throws(() => readFileSync(join(staged, "stale.md")), { code: "ENOENT" });
    rmSync(join(root, "content/generated/rules/ja/all.md"));
    assert.throws(() => materializeContent(root), { code: "ENOENT" });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
