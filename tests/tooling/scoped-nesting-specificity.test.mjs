import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/scoped_nesting_specificity.json",
      import.meta.url,
    ),
  ),
);
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const fromTests = createRequire(new URL("../package.json", import.meta.url));
const compiler = fromUi("@vue/compiler-sfc");
const { chromium } = fromTests("@playwright/test");
const { PNG } = fromTests("pngjs");
const sourcePath = process.env.VIZE_SCOPED_NESTING_CAPTURE;
const referenceOnly = process.env.VIZE_SCOPED_NESTING_REFERENCE_ONLY === "1";
const captures = sourcePath ? JSON.parse(fs.readFileSync(sourcePath)) : null;
const output = process.env.VIZE_SCOPED_NESTING_RUNTIME_CAPTURE;
const executions = [];
if (process.env.VIZE_SCOPED_NESTING_REQUIRE_SOURCE === "1")
  assert.ok(captures, "current source capture is required by the scoped CSS lane");

await test(
  "selector-list nesting preserves complete scoped CSS, cascade and Chromium pixels",
  { skip: !captures && !referenceOnly },
  async () => {
    assert.equal(pack.schema, "vize.scoped-nesting.specificity");
    assert.equal(pack.reference, "@vue/compiler-sfc@3.5.35");
    assert.equal(compiler.version, "3.5.35");
    assert.equal(pack.cases.length, 26);
    assert.equal(new Set(pack.cases.map((item) => item.id)).size, 26);
    assert.ok(captures || referenceOnly, "fresh current-source capture is mandatory");
    if (captures) assert.equal(captures.length, pack.cases.length * 3);
    const browser = await chromium.launch();
    const regressions = [];
    try {
      for (const fixture of pack.cases) {
        const reference = compiler.compileStyle({
          source: fixture.css,
          filename: "Nesting.vue",
          id: pack.scopeId,
          scoped: true,
        });
        assert.deepEqual(reference.errors, [], fixture.id);
        assert.equal(
          reference.code,
          fixture.referenceCss,
          fixture.id + ": whole independent reference CSS",
        );
        const variants = [
          { name: "reference", css: reference.code },
          { name: "public-0.566.0", css: fixture.beforeCss },
          ...(captures ?? [])
            .filter((capture) => capture.id === fixture.id)
            .map((capture) => ({ name: capture.backend, ...capture })),
        ];
        const observed = [];
        for (const variant of variants) {
          const page = await browser.newPage({
            viewport: { width: 320, height: 240 },
            deviceScaleFactor: 1,
          });
          const errors = [];
          page.on("pageerror", (error) => errors.push(error.message));
          const outside = fixture.html.replace(/ data-v-nesting(?:-s)?/g, "");
          await page.setContent(`<!doctype html><html><head><style>
          body{margin:0;background:white}.target{display:block;width:32px;height:32px;position:relative}
          #inside{position:absolute;left:0;top:0}#outside{position:absolute;left:160px;top:0}
          ${fixture.competitor}\n${variant.css}
          </style></head><body><section id="inside">${fixture.html}</section><section id="outside">${outside}</section></body></html>`);
          if (fixture.hover) await page.locator("#inside .target").hover();
          const state = await page.evaluate((pseudo) => {
            const item = (host) => {
              const element = document.querySelector(host + " .target");
              const style = getComputedStyle(element, pseudo);
              const rect = element.getBoundingClientRect();
              return {
                background: style.backgroundColor,
                color: style.color,
                content: style.content,
                rect: { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
              };
            };
            const rules = (list) =>
              Array.from(list, (rule) => ({
                text: rule.cssText,
                ...(rule.cssRules ? { children: rules(rule.cssRules) } : {}),
              }));
            return {
              inside: item("#inside"),
              outside: item("#outside"),
              rules: rules(document.styleSheets[0].cssRules),
            };
          }, fixture.pseudo);
          const screenshot = await page.screenshot({ animations: "disabled" });
          const image = PNG.sync.read(screenshot);
          const pixel = ({ rect }) => {
            const offset =
              (Math.floor(rect.y + rect.height / 2) * image.width +
                Math.floor(rect.x + rect.width / 2)) *
              4;
            return Array.from(image.data.subarray(offset, offset + 4));
          };
          const result = {
            name: variant.name,
            css: variant.css,
            state,
            pixels: { inside: pixel(state.inside), outside: pixel(state.outside) },
            screenshotSha256: createHash("sha256").update(screenshot).digest("hex"),
            screenshotBase64: screenshot.toString("base64"),
            errors,
          };
          observed.push(result);
          await page.close();
        }
        const referenceResult = observed[0];
        assert.deepEqual(referenceResult.errors, [], fixture.id);
        assert.deepEqual(
          referenceResult.pixels.inside,
          fixture.expected,
          fixture.id + ": explicit independent winner",
        );
        if (fixture.id !== "global-child")
          assert.deepEqual(
            referenceResult.pixels.outside,
            [255, 255, 255, 255],
            fixture.id + ": outside scope",
          );
        const semantic = (result) => ({
          inside: result.state.inside,
          outside: result.state.outside,
          pixels: result.pixels,
        });
        if (JSON.stringify(semantic(observed[1])) !== JSON.stringify(semantic(referenceResult)))
          regressions.push(fixture.id);
        for (const current of observed.slice(2)) {
          assert.deepEqual(current.errors, [], fixture.id + "/" + current.name);
          assert.deepEqual(
            semantic(current),
            semantic(referenceResult),
            fixture.id + "/" + current.name + ": full computed state and RGBA pixels",
          );
        }
        executions.push({
          id: fixture.id,
          source: fixture.source,
          authoredCss: fixture.css,
          observed,
        });
      }
      for (const id of [
        "reported-hover",
        "implicit-descendant",
        "nested-parent-list",
        "quoted-ampersand",
        "multiple-ampersands",
      ])
        assert.ok(
          regressions.includes(id),
          id + ": the physical public baseline must distinguish the regression",
        );
      assert.equal(captures ? captures.length : 0, referenceOnly ? 0 : 78);
    } finally {
      if (output) {
        fs.mkdirSync(path.dirname(output), { recursive: true });
        fs.writeFileSync(
          output,
          JSON.stringify(
            {
              source: process.env.HEAD_SHA ?? null,
              eventSource: process.env.GITHUB_SHA ?? null,
              browser: browser.version(),
              reference: pack.reference,
              before: pack.before,
              currentSource: !!captures,
              baselineRegressions: regressions,
              executions,
            },
            null,
            2,
          ) + "\n",
        );
      }
      await browser.close();
    }
  },
);
