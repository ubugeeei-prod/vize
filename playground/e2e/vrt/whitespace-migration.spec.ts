import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { expect, test } from "@playwright/test";
import native from "../../../npm/native/index.js";

const require = createRequire(import.meta.url);
const vueRuntime = readFileSync(require.resolve("vue/dist/vue.global.prod.js"), "utf8");
const source = `<p style="display: flex; letter-spacing: 0.25px">
    {{ label }}
    <i class="icon" />
  </p>`;

test("Vue 2 line-break migration keeps the text before a flex icon", async ({ page }) => {
  const defaultResult = native.compile(source, { mode: "module", whitespace: "condense" });
  const migrationResult = native.compile(source, {
    mode: "module",
    whitespace: "vue2-line-breaks",
  });

  await page.setContent(`
    <style>
      p { margin: 0; font: 12px Arial, sans-serif; }
      i { display: block; width: 10px; height: 10px; background: red; }
    </style>
    <div id="default"></div>
    <div id="migration"></div>
  `);
  await page.addScriptTag({ content: vueRuntime });

  const rendered = await page.evaluate(
    ({ defaultCode, migrationCode }) => {
      const Vue = (globalThis as typeof globalThis & { Vue: typeof import("vue") }).Vue;
      const helperNames = [
        "_toDisplayString",
        "_createElementVNode",
        "_openBlock",
        "_createElementBlock",
        "_createTextVNode",
      ];
      const helpers = [
        Vue.toDisplayString,
        Vue.createElementVNode,
        Vue.openBlock,
        Vue.createElementBlock,
        Vue.createTextVNode,
      ];

      function mount(id: string, code: string) {
        // Execute only the compiler output generated from the fixed fixture above.
        // oxlint-disable-next-line typescript/no-implied-eval
        const render = new Function(
          ...helperNames,
          `${code.replace("export function render", "function render")}\nreturn render;`,
        )(...helpers) as () => ReturnType<typeof Vue.h>;
        const container = document.getElementById(id)!;
        Vue.createApp({ data: () => ({ label: "Label" }), render }).mount(container);
        const paragraph = container.querySelector("p")!;
        const icon = paragraph.querySelector("i")!;
        return {
          text: paragraph.childNodes[0]?.textContent,
          iconX: icon.getBoundingClientRect().x - paragraph.getBoundingClientRect().x,
        };
      }

      return {
        default: mount("default", defaultCode),
        migration: mount("migration", migrationCode),
      };
    },
    { defaultCode: defaultResult.code, migrationCode: migrationResult.code },
  );

  expect(rendered.default.text).toBe("Label ");
  expect(rendered.migration.text).toBe("Label\n");
  // Chromium collapses the trailing whitespace for flex layout even though
  // the migration mode preserves the DOM text needed by Vue 2 consumers.
  expect(rendered.migration.iconX).toBeCloseTo(rendered.default.iconX, 4);
});
