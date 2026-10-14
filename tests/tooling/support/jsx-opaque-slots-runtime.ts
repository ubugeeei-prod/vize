// Mount compiled opaque-slot modules (#9075) beside the real
// `@vue/babel-plugin-jsx` output and require identical HTML.
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, symlinkSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import { transformSync } from "@babel/core";
import tsSyntax from "@babel/plugin-syntax-typescript";
import jsx from "@vue/babel-plugin-jsx";
import { Window } from "happy-dom";

type Fixture = { source: string; lang: "jsx" | "tsx"; code: string };

let payload = "";
for await (const chunk of process.stdin) payload += chunk;
const fixtures = JSON.parse(payload) as Record<string, Fixture>;
const window = new Window();
for (const key of [
  "window",
  "document",
  "Document",
  "Node",
  "Text",
  "Comment",
  "Element",
  "HTMLElement",
  "SVGElement",
  "MathMLElement",
])
  (globalThis as Record<string, unknown>)[key] =
    key === "window" ? window : (window as unknown as Record<string, unknown>)[key];
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const Vue = fromUi("vue");
const workspace = mkdtempSync(join(tmpdir(), "vize-jsx-opaque-slots-"));

// Babel keeps TypeScript syntax; erase `as` for Node, as the spread-child harness does.
function reference(source: string, lang: string): string {
  const plugins: unknown[] = [jsx];
  if (lang === "tsx") plugins.push([tsSyntax, { isTSX: true }]);
  const code = transformSync(source, {
    plugins,
    babelrc: false,
    configFile: false,
    filename: `fixture.${lang}`,
  })!.code!;
  if (lang !== "tsx") return code;
  return transformSync(code, {
    plugins: [
      [tsSyntax, { isTSX: true }],
      () => ({
        visitor: {
          TSAsExpression(path: {
            replaceWith(node: unknown): void;
            node: { expression: unknown };
          }) {
            path.replaceWith(path.node.expression);
          },
        },
      }),
    ],
    babelrc: false,
    configFile: false,
    filename: "fixture.tsx",
  })!.code!;
}

async function render(id: string, kind: string, code: string): Promise<string> {
  const root = join(workspace, `${id}-${kind}`);
  mkdirSync(join(root, "node_modules"), { recursive: true });
  symlinkSync(dirname(fromUi.resolve("vue/package.json")), join(root, "node_modules/vue"));
  writeFileSync(join(root, "compiled.mjs"), code);
  const module = await import(pathToFileURL(join(root, "compiled.mjs")).href);
  const host = window.document.createElement("div");
  const warnings: string[] = [];
  const app = Vue.createApp(module.App);
  app.config.warnHandler = (message: string) => warnings.push(message);
  app.mount(host);
  assert.deepEqual(warnings, [], `${id} ${kind} warnings`);
  return host.innerHTML;
}

const rendered: Record<string, string> = {};
for (const [id, fixture] of Object.entries(fixtures)) {
  const vize = await render(id, "vize", fixture.code);
  const babel = await render(id, "babel", reference(fixture.source, fixture.lang));
  assert.equal(vize, babel, id);
  assert.match(vize, /<i>icon<\/i>/u, id);
  rendered[id] = vize;
}
process.stdout.write(JSON.stringify({ rendered }));
