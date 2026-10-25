import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";
import {
  basePath,
  createFixtureServer,
  prepareContext,
} from "./global-toolbar.browser-fixtures.ts";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
const output =
  process.env.MUSEA_HOME_PROOF_DIR || path.join(repository, "artifacts/musea-home-layout");
const titles = [
  "OrganizationConfigurationPermissionsAdministrationSummary",
  "構成管理と権限を確認するためのコンポーネント🧭🧑🏽‍💻",
  "Short title",
];

await test(
  "home cards retain long names, status and tags within narrow and multi-column galleries",
  { skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1", timeout: 60_000 },
  async () => {
    const server = await createFixtureServer(output);
    const browser = await chromium.launch();
    const observations: unknown[] = [];
    const errors: string[] = [];
    try {
      await mkdir(output, { recursive: true });
      await server.listen();
      const address = server.httpServer!.address();
      assert.ok(address && typeof address !== "string");
      const context = await browser.newContext();
      await prepareContext(context);
      await context.route(`**${basePath}/api/arts`, (route) =>
        route.fulfill({
          json: titles.map((title, index) => ({
            path: `Example${index}.art.vue`,
            metadata: {
              title,
              status: "ready",
              tags: ["organizationconfigurationpermissions", "production", "権限管理🧭"],
            },
            variants: [{ name: "Default", template: "<output />", isDefault: true }],
            hasScriptSetup: false,
            hasScript: false,
            styleCount: 0,
          })),
        }),
      );
      const page = await context.newPage();
      page.on("pageerror", (error) => errors.push(String(error)));
      await page.goto(`http://127.0.0.1:${address.port}${basePath}/`);
      await page.locator(".home-card").last().waitFor();
      for (const viewport of [
        { width: 320, height: 700 },
        { width: 375, height: 667 },
        { width: 390, height: 844 },
        { width: 768, height: 1024 },
        { width: 1024, height: 768 },
        { width: 1280, height: 720 },
      ]) {
        await page.setViewportSize(viewport);
        await page.waitForFunction(
          (width) => Boolean(document.querySelector(".sidebar-wrapper")) === width > 768,
          viewport.width,
        );
        const state = await page.evaluate(() => {
          const content = document.querySelector(".content")!;
          const grid = document.querySelector(".home-category-grid")!;
          const area = grid.getBoundingClientRect();
          return {
            viewport: [innerWidth, innerHeight],
            documentWidth: document.documentElement.scrollWidth,
            content: [content.clientWidth, content.scrollWidth],
            grid: [grid.clientWidth, grid.scrollWidth],
            cards: [...document.querySelectorAll<HTMLAnchorElement>(".home-card")].map((card) => {
              const rect = card.getBoundingClientRect();
              const title = card.querySelector(".home-card-title")!;
              const badge = card.querySelector(".status-badge")!;
              const status = badge.getBoundingClientRect();
              const statusText = document.createRange();
              statusText.selectNodeContents(badge);
              return {
                title: title.textContent,
                href: card.getAttribute("href"),
                size: [card.clientWidth, card.scrollWidth],
                bounds: [rect.left - area.left, area.right - rect.right],
                status: [status.left - rect.left, rect.right - status.right],
                statusLines: [...statusText.getClientRects()].filter((line) => line.width > 0)
                  .length,
                tags: [...card.querySelectorAll(".home-card-tag")].map((tag) =>
                  tag.textContent?.trim(),
                ),
              };
            }),
          };
        });
        observations.push(state);
        assert.equal(state.documentWidth, viewport.width, JSON.stringify(state));
        assert.equal(state.content[0], state.content[1], JSON.stringify(state));
        assert.equal(state.grid[0], state.grid[1], JSON.stringify(state));
        assert.deepEqual(
          state.cards.map((card) => card.title),
          titles,
        );
        for (const [index, card] of state.cards.entries()) {
          assert.equal(card.size[0], card.size[1], JSON.stringify(state));
          assert.ok(
            card.bounds.every((offset) => offset >= -1),
            JSON.stringify(state),
          );
          assert.ok(
            card.status.every((offset) => offset >= 0),
            JSON.stringify(state),
          );
          assert.equal(card.statusLines, 1, JSON.stringify(state));
          assert.equal(card.href, `${basePath}/component/Example${index}.art.vue`);
          assert.deepEqual(card.tags, [
            "#organizationconfigurationpermissions",
            "#production",
            "#権限管理🧭",
          ]);
        }
        await page.screenshot({
          path: path.join(output, `${viewport.width}x${viewport.height}.png`),
        });
      }
      assert.deepEqual(errors, []);
    } finally {
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify({ observations, errors }, null, 2),
      );
      await browser.close();
      await server.close();
    }
  },
);
