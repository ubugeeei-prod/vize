import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import type { Browser } from "@playwright/test";

type JsonObject = Record<string, unknown>;
function object(value: unknown): JsonObject {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as JsonObject;
}
function string(value: unknown): string {
  assert.equal(typeof value, "string");
  return value as string;
}
function array(value: unknown): unknown[] {
  assert(Array.isArray(value));
  return value;
}
function read(url: URL): unknown {
  return JSON.parse(fs.readFileSync(url, "utf8")) as unknown;
}
type ScopeCase = {
  id: string;
  source: string;
  referenceCss: string;
  expected: Record<string, string>;
};
function scopeCase(value: unknown): ScopeCase {
  const row = object(value);
  const expected = Object.fromEntries(
    Object.entries(object(row.expected)).map(([id, color]) => [id, string(color)]),
  );
  return {
    id: string(row.id),
    source: string(row.source),
    referenceCss: string(row.referenceCss),
    expected,
  };
}
type Capture = { id: string; minify: boolean; code: string; source: string };
function capture(value: unknown): Capture {
  const row = object(value);
  assert.equal(typeof row.minify, "boolean");
  assert.deepEqual(row.errors, []);
  assert.deepEqual(row.warnings, []);
  return {
    id: string(row.id),
    minify: row.minify as boolean,
    code: string(row.code),
    source: string(row.source),
  };
}
const root = new URL("../../crates/vize_atelier_sfc/tests/fixtures/", import.meta.url);
const pack = object(read(new URL("scope-body-10810.json.txt", root)));
const cases = array(pack.cases).map(scopeCase);
const before = object(read(new URL("scope-body-before-public-0.600.0.json.txt", root)));
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const compiler = fromUi("@vue/compiler-sfc") as {
  version: string;
  compileStyle(options: { source: string; filename: string; id: string; scoped: boolean }): {
    code: string;
    errors: unknown[];
  };
};

async function observe(browser: Browser, css: string, ids: string[]) {
  const page = await browser.newPage();
  const errors: string[] = [];
  page.on("pageerror", (error: Error) => errors.push(error.message));
  try {
    await page.setContent(
      "<!doctype html><style>*{color:rgb(0,0,0)}" + css + "</style>" + string(pack.html),
    );
    const colors = await page.evaluate((names) => {
      return Object.fromEntries(
        names.map((id) => {
          const element = document.getElementById(id);
          if (!element) throw new Error("Missing scope witness " + id);
          return [id, getComputedStyle(element).color];
        }),
      );
    }, ids);
    const rules = await page.evaluate(() =>
      Array.from(document.styleSheets[0]?.cssRules ?? [], (rule) => rule.cssText),
    );
    assert.deepEqual(errors, []);
    return { css, colors, rules, errors };
  } finally {
    await page.close();
  }
}

test(
  "scope source preserves whole Vue CSS, relative selectors and Chromium specificity",
  { skip: !process.env.VIZE_SCOPE_BODY_CAPTURE },
  async () => {
    assert.equal(pack.schema, "vize.scope-body.10810");
    assert.equal(pack.reference, "@vue/compiler-sfc@3.5.35");
    assert.equal(compiler.version, "3.5.35");
    assert.equal(cases.length, 11);
    assert.equal(new Set(cases.map((row) => row.id)).size, 11);
    assert.equal(before.version, "0.600.0");
    const capturePath = process.env.VIZE_SCOPE_BODY_CAPTURE;
    assert(capturePath, "fresh current-source CSS capture is required");
    const current = array(JSON.parse(fs.readFileSync(capturePath, "utf8")) as unknown).map(capture);
    assert.equal(current.length, 22);
    const oldRows = array(before.rows).map((value) => object(value));
    assert.equal(oldRows.length, 22);
    const fromTests = createRequire(new URL("../package.json", import.meta.url));
    const { chromium } = fromTests("@playwright/test") as typeof import("@playwright/test");
    const browser = await chromium.launch();
    const observations: unknown[] = [];
    const historicalFailures = new Set<string>();
    try {
      for (const fixture of cases) {
        const reference = compiler.compileStyle({
          source: fixture.source,
          filename: "Scope.vue",
          id: string(pack.scopeId),
          scoped: true,
        });
        assert.deepEqual(reference.errors, []);
        assert.equal(reference.code, fixture.referenceCss, fixture.id + ": whole Vue output");
        const expected = await observe(browser, reference.code, Object.keys(fixture.expected));
        assert.deepEqual(
          expected.colors,
          fixture.expected,
          fixture.id + ": explicit browser oracle",
        );
        const actual = current.filter((row) => row.id === fixture.id);
        assert.deepEqual(
          actual.map((row) => row.minify),
          [false, true],
        );
        const variants: unknown[] = [];
        for (const row of actual) {
          assert.equal(row.source, fixture.source, fixture.id + ": full authored source");
          const observed = await observe(browser, row.code, Object.keys(fixture.expected));
          assert.deepEqual(observed.colors, expected.colors, fixture.id + "/" + row.minify);
          variants.push({ minify: row.minify, ...observed });
        }
        const historical: unknown[] = [];
        for (const row of oldRows.filter((item) => item.id === fixture.id)) {
          const observed = await observe(browser, string(row.code), Object.keys(fixture.expected));
          if (JSON.stringify(observed.colors) !== JSON.stringify(expected.colors))
            historicalFailures.add(fixture.id);
          historical.push({ originalPacket: row, observed });
        }
        observations.push({ fixture, expected, variants, historical });
      }
      for (const id of ["reported-scope", "mixed-selector-list", "explicit-scope-specificity"])
        assert(
          historicalFailures.has(id),
          id + ": original public bytes must distinguish regression",
        );
    } finally {
      const output = process.env.VIZE_SCOPE_BODY_RUNTIME_CAPTURE;
      if (output)
        fs.writeFileSync(
          output,
          JSON.stringify(
            {
              schema: "vize.scope-body.browser.10810",
              source: process.env.GITHUB_SHA ?? null,
              observations,
              historicalFailures: [...historicalFailures],
            },
            null,
            2,
          ),
        );
      await browser.close();
    }
  },
);
