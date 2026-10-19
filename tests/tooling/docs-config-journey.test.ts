import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { renderReferenceDocs } from "../../npm/ui/scripts/generate-reference-docs.ts";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const locales = ["", "ja", "fr", "pt-BR", "zh-CN"];
let generated: Map<string, string> | undefined;

/** Authored page source, or the generated reference page the docs build would emit. */
function sourceFor(file: string): string | undefined {
  const absolute = resolve(root, file);
  if (existsSync(absolute)) return readFileSync(absolute, "utf8");
  generated ??= renderReferenceDocs();
  return generated.get(file.replace(/^docs\/content\//u, ""));
}

function render(file: string): { html: string; source: string } {
  const source = sourceFor(file);
  assert.ok(source !== undefined, `${file}: source or generated reference exists`);
  const result = native.transform(source, {});
  assert.deepEqual(result.errors, [], file);
  return { source, html: result.html as string };
}

void test("localized configuration reading paths resolve through native rendered targets", () => {
  const rendered = new Map<string, ReturnType<typeof render>>();
  for (const locale of locales) {
    const prefix = `docs/content/${locale ? `${locale}/` : ""}`;
    for (const name of [
      "getting-started.md",
      "guide/configuration.md",
      "guide/configuration-reference.md",
      "guide/compiler-configuration-reference.md",
    ]) {
      const file = `${prefix}${name}`;
      const document = render(file);
      for (const [, href, fragment] of document.html.matchAll(/\bhref="([^"#]*\.md)(#[^"]*)?"/g)) {
        if (!href || /^(?:https?:|\/\/)/u.test(href)) continue;
        const target = href.startsWith("/")
          ? resolve(root, "docs/content", `.${href}`)
          : resolve(root, dirname(file), href);
        const relative = target.slice(`${root}/`.length);
        assert.ok(sourceFor(relative) !== undefined, `${file}: ${href}`);
        if (!rendered.has(relative)) rendered.set(relative, render(relative));
        if (fragment) {
          const id = decodeURIComponent(fragment.slice(1));
          assert.ok(
            rendered.get(relative)?.html.includes(`id="${id}"`),
            `${file}: ${href}${fragment}`,
          );
        }
      }
    }
  }
  assert.ok(rendered.size >= 30, "the complete localized reading paths were visited");
});
