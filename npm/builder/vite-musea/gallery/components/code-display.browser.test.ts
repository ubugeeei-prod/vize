import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium, type Locator, type Page } from "playwright";
import { createServer } from "vite";
import vue from "@vitejs/plugin-vue";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
const gallery = fileURLToPath(new URL("../", import.meta.url));
const output = process.env.MUSEA_CODE_PROOF_DIR || path.join(repository, "artifacts/musea-code");
interface Fixture {
  template: string;
  label: string;
}

async function metrics(pre: Locator) {
  return pre.evaluate((element) => {
    const code = element.querySelector("code");
    if (!code) throw new Error("Missing highlighted code");
    const style = getComputedStyle(code);
    return {
      text: code.textContent,
      width: element.clientWidth,
      scrollWidth: element.scrollWidth,
      height: element.clientHeight,
      scrollHeight: element.scrollHeight,
      left: element.getBoundingClientRect().left,
      right: element.getBoundingClientRect().right,
      padding: style.padding,
      whiteSpace: style.whiteSpace,
      tabSize: style.tabSize,
      overflow: style.overflow,
      color: style.color,
      background: getComputedStyle(element).backgroundColor,
    };
  });
}

async function checkScroll(page: Page, pre: Locator, expected: string) {
  await pre.waitFor();
  assert.equal(await pre.locator("code").textContent(), expected);
  const measured = await metrics(pre);
  assert.equal(measured.padding, "0px");
  assert.equal(measured.whiteSpace, "pre");
  assert.equal(measured.tabSize, "2");
  assert.equal(measured.overflow, "visible");
  assert.ok(measured.width > 0 && measured.right <= page.viewportSize()!.width);
  assert.ok(measured.left >= 0);
  assert.ok(measured.scrollWidth > measured.width, JSON.stringify(measured));
  await pre.focus();
  assert.equal(await pre.evaluate((element) => element === document.activeElement), true);
  await page.keyboard.press("ArrowRight");
  await page.waitForFunction(() => document.activeElement!.scrollLeft > 0);
  const end = await pre.evaluate((element) => {
    element.scrollLeft = element.scrollWidth;
    element.scrollTop = element.scrollHeight;
    return { left: element.scrollLeft, top: element.scrollTop };
  });
  assert.ok(end.left > 0);
  const tail = await pre.evaluate((element) => {
    const code = element.querySelector("code")!;
    let offset = 0;
    let last = 0;
    let length = 0;
    for (const line of (code.textContent || "").split("\n")) {
      if (line.length > length) {
        length = line.length;
        last = offset + length - 1;
      }
      offset += line.length + 1;
    }
    const walker = document.createTreeWalker(code, NodeFilter.SHOW_TEXT);
    let node = walker.nextNode();
    while (node) {
      const size = node.textContent?.length || 0;
      if (last < size) {
        const range = document.createRange();
        range.setStart(node, last);
        range.setEnd(node, last + 1);
        return {
          right: range.getBoundingClientRect().right,
          edge: element.getBoundingClientRect().right,
        };
      }
      last -= size;
      node = walker.nextNode();
    }
    throw new Error("Missing longest-line end");
  });
  assert.ok(tail.right <= tail.edge, JSON.stringify(tail));
  if (measured.scrollHeight > measured.height) assert.ok(end.top > 0);
  assert.equal(await pre.locator("code").textContent(), expected);
  await pre.evaluate((element) => {
    element.blur();
    element.scrollTo({ left: 0, top: 0, behavior: "instant" });
  });
  return measured;
}

await test(
  "gallery code preserves whole source, scrolls within narrow columns and follows themes",
  {
    skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1",
  },
  async () => {
    const fixture: Fixture = JSON.parse(
      await readFile(
        path.join(repository, "tests/_fixtures/differential/musea/code-display.json"),
        "utf8",
      ),
    );
    const source = await readFile(
      path.join(repository, "examples/vite-musea/src/components/MuseaButton.vue"),
      "utf8",
    );
    const variants = [...source.matchAll(/<variant\s+([^>]*)>([\s\S]*?)<\/variant>/g)].map(
      (match) => ({
        name: match[1].match(/name="([^"]+)"/)![1],
        template: match[2].trim(),
        isDefault: match[1].includes(" default"),
        skipVrt: match[1].includes("skip-vrt"),
      }),
    );
    const script = source.match(/<script setup[^>]*>([\s\S]*?)<\/script>/)![1];
    const art = {
      path: "src/components/MuseaButton.vue",
      metadata: {
        title: "Button",
        component: "MuseaButton",
        category: "Components",
        tags: [],
        status: "ready",
      },
      variants: [
        ...variants,
        { name: "Code display", template: fixture.template, isDefault: false, skipVrt: true },
      ],
      hasScriptSetup: true,
      hasScript: false,
      styleCount: 1,
      scriptSetupContent: script,
      scriptSetupIsolated: true,
    };
    const palette = {
      title: "MuseaButton",
      groups: [],
      json: "",
      typescript: "",
      controls: [
        {
          name: "label",
          control: "text",
          default_value: fixture.label,
          options: [],
          required: false,
        },
      ],
    };
    const server = await createServer({
      configFile: false,
      root: gallery,
      base: "/__musea__/",
      plugins: [vue()],
      server: { host: "127.0.0.1", port: 0 },
      cacheDir: path.join(output, "vite-cache"),
    });
    const browser = await chromium.launch();
    const observations: unknown[] = [];
    const errors: string[] = [];
    try {
      await mkdir(output, { recursive: true });
      await server.listen();
      const address = server.httpServer!.address();
      assert.ok(address && typeof address !== "string");
      const context = await browser.newContext({
        permissions: ["clipboard-read", "clipboard-write"],
      });
      const page = await context.newPage();
      page.on("pageerror", (error) => {
        errors.push(String(error));
      });
      // The real gallery consumes explicit fixture API responses. This qualifies
      // code presentation, not native parsing or the preview compiler.
      await page.route("**/__musea__/api/**", async (route) => {
        const name = new URL(route.request().url()).pathname;
        const data = name.endsWith("/palette")
          ? palette
          : name.endsWith("/analysis")
            ? { props: [], emits: [] }
            : name.endsWith("/docs")
              ? {
                  markdown: "```vue\n" + fixture.template + "\n```",
                  title: "Button",
                  variant_count: art.variants.length,
                }
              : name.endsWith("/tokens")
                ? {
                    categories: [],
                    tokenMap: {},
                    meta: { filePath: "", tokenCount: 0, primitiveCount: 0, semanticCount: 0 },
                  }
                : name.endsWith("/arts")
                  ? [art]
                  : art;
        await route.fulfill({ json: data });
      });
      await page.route("**/__musea__/preview?**", (route) =>
        route.fulfill({
          contentType: "text/html",
          body: "<!doctype html><html><body>Code-display fixture preview</body></html>",
        }),
      );
      await page.goto(`http://127.0.0.1:${address.port}/__musea__/`);
      await page.locator(".art-item").first().click();
      await page.locator('[title="View source"]').last().click();
      const pre = page.locator(".source-pre");
      for (const width of [1280, 768, 390, 320]) {
        await page.setViewportSize({ width, height: 900 });
        for (const theme of ["light", "dark"]) {
          await page.evaluate(
            (name) => document.documentElement.setAttribute("data-musea-theme", name),
            theme,
          );
          observations.push({
            panel: "template-before-check",
            width,
            theme,
            measured: await metrics(pre),
          });
          const measured = await checkScroll(page, pre, fixture.template);
          assert.ok(measured.scrollHeight > measured.height);
          observations.push({ panel: "template", width, theme, measured });
          await pre.scrollIntoViewIfNeeded();
          await pre
            .locator("..")
            .screenshot({ path: path.join(output, `template-${width}-${theme}.png`) });
        }
      }
      await page.locator(".source-copy-btn").click();
      assert.equal(await page.evaluate(() => navigator.clipboard.readText()), fixture.template);
      await page.locator(".tab-btn").filter({ hasText: "Props" }).click();
      const usage = page.locator(".props-usage-code");
      const expected = `<script setup lang="ts">\nimport "../theme.css";\ndefineProps<{\n  variant?: "default" | "primary" | "secondary";\n  size?: "sm" | "md" | "lg";\n  disabled?: boolean;\n}>();\n</script>\n\n<template>\n  <MuseaButton label="${fixture.label}" />\n</template>`;
      for (const width of [1280, 768, 390, 320]) {
        await page.setViewportSize({ width, height: 900 });
        for (const theme of ["light", "dark"]) {
          await page.evaluate(
            (name) => document.documentElement.setAttribute("data-musea-theme", name),
            theme,
          );
          observations.push({
            panel: "usage-before-check",
            width,
            theme,
            measured: await metrics(usage),
          });
          const measured = await checkScroll(page, usage, expected);
          observations.push({ panel: "usage", width, theme, measured });
          await usage.scrollIntoViewIfNeeded();
          await usage
            .locator("..")
            .screenshot({ path: path.join(output, `usage-${width}-${theme}.png`) });
        }
      }
      await page.locator(".props-copy-btn").click();
      assert.equal(await page.evaluate(() => navigator.clipboard.readText()), expected);
      await checkScroll(
        page,
        page.locator(".props-json-code"),
        JSON.stringify({ label: fixture.label }, null, 2),
      );
      const colors: Record<string, string> = {};
      for (const theme of ["light", "dark", "system"]) {
        await page.evaluate(
          (name) => document.documentElement.setAttribute("data-musea-theme", name),
          theme,
        );
        for (const scheme of ["light", "dark"] as const) {
          await page.emulateMedia({ colorScheme: scheme });
          colors[`${theme}/${scheme}`] = await usage
            .locator(".hljs-name")
            .first()
            .evaluate((element) => getComputedStyle(element).color);
        }
      }
      assert.notEqual(colors["light/light"], colors["dark/light"]);
      assert.equal(colors["light/light"], colors["light/dark"]);
      assert.equal(colors["dark/light"], colors["dark/dark"]);
      assert.equal(colors["system/light"], colors["light/light"]);
      assert.equal(colors["system/dark"], colors["dark/dark"]);
      observations.push({ panel: "theme", colors });
      assert.deepEqual(errors, []);
    } finally {
      await mkdir(output, { recursive: true });
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify({ fixture, originalButtonSource: source, observations, errors }, null, 2),
      );
      await browser.close();
      await server.close();
    }
  },
);
