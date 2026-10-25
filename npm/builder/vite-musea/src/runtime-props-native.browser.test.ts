import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { chromium, type Browser } from "playwright";
import { createServer, type ViteDevServer } from "vite";
import ts from "typescript";
import vize from "../../vite/src/index.ts";
import { musea } from "./plugin/index.ts";
import { runtimeArt, runtimeComponent, expectedRuntimeDefaults } from "./runtime-props-fixtures.ts";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));

void test(
  "actual native export, HTTP metadata and editable runtime controls agree without executing defaults",
  {
    skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1",
    timeout: 120_000,
  },
  async () => {
    const output = path.join(repository, "artifacts/musea-native-runtime-props");
    const root = path.join(output, "project");
    const componentPath = path.join(root, "src/RuntimeProbe.vue");
    const artPath = path.join(root, "src/RuntimeProbe.art.vue");
    await mkdir(path.dirname(componentPath), { recursive: true });
    await writeFile(componentPath, runtimeComponent);
    await writeFile(artPath, runtimeArt);
    await writeFile(path.join(root, "index.html"), "<!doctype html><html><body></body></html>");
    const observations: Record<string, unknown> = { runtimeComponent, runtimeArt };
    const errors: string[] = [];
    let browser: Browser | undefined;
    let server: ViteDevServer | undefined;
    try {
      // Resolve the published entrypoint, not the fallback or a fixture binding.
      const binding = createRequire(import.meta.url)("@vizejs/native");
      assert.equal(
        typeof binding.analyzeSfc,
        "function",
        "analyzeSfc must be a real exported native API",
      );
      observations.entrypoint = import.meta.resolve("@vizejs/native");
      const cold = binding.analyzeSfc(runtimeComponent, {
        filename: componentPath,
        profile: true,
        cache: false,
      });
      observations.cold = cold;
      assert.equal(
        cold.profile.literalMemberParseCalls,
        0,
        "ASCII/runtime metadata never reparses type members",
      );
      const literalSource = `<script setup lang="ts">defineProps<{ "scope.name"?: string; café?: number; ['a:b']?: boolean; [key]: string }>()</script>`;
      const literalCold = binding.analyzeSfc(literalSource, {
        filename: "Literal.vue",
        profile: true,
      });
      const literalWarm = binding.analyzeSfc(literalSource, {
        filename: "Literal.vue",
        profile: true,
      });
      assert.deepEqual(
        literalCold.props.map((prop: any) => prop.name),
        ["scope.name", "café", "a:b"],
      );
      assert.equal(literalCold.profile.literalMemberParseCalls, 4);
      assert.equal(literalWarm.profile.literalMemberParseCalls, 0);
      assert.equal(literalWarm.profile.cacheHit, true);
      observations.literalProfile = { cold: literalCold, warm: literalWarm };
      const props = new Map<string, any>(cold.props.map((prop: any) => [prop.name, prop]));
      assert.deepEqual(
        [...props.keys()],
        [...Object.keys(expectedRuntimeDefaults), "callback", "opaque"],
      );
      for (const [name, value] of Object.entries(expectedRuntimeDefaults))
        assert.deepEqual(props.get(name).default_value, value, name);
      assert.equal(props.get("count").required, true);
      assert.deepEqual(cold.emits, ["change", "update:modelValue"]);
      for (const name of ["callback", "opaque"]) {
        assert.equal(props.get(name).unsupported_default, true);
        assert.equal(Object.hasOwn(props.get(name), "default_value"), false);
        assert.equal(typeof props.get(name).default_source, "string");
      }
      for (const key of [
        "descriptorParseCalls",
        "analyzeCalls",
        "defaultParseCalls",
        "serializeCalls",
      ])
        assert.equal(cold.profile[key], 1, key);
      const wallStart = performance.now();
      const coldSamples = Array.from(
        { length: 100 },
        () =>
          binding.analyzeSfc(runtimeComponent, {
            filename: componentPath,
            profile: true,
            cache: false,
          }).profile,
      );
      const coldWallMs = performance.now() - wallStart;
      binding.analyzeSfc(runtimeComponent, { filename: componentPath });
      const warmStart = performance.now();
      const warmSamples = Array.from(
        { length: 100 },
        () =>
          binding.analyzeSfc(runtimeComponent, { filename: componentPath, profile: true }).profile,
      );
      const warmWallMs = performance.now() - warmStart;
      for (const sample of warmSamples) {
        assert.equal(sample.cacheHit, true);
        for (const key of [
          "descriptorParseCalls",
          "analyzeCalls",
          "defaultParseCalls",
          "serializeCalls",
        ])
          assert.equal(sample[key], 0, key);
      }
      observations.profile = { samples: 100, coldWallMs, warmWallMs, coldSamples, warmSamples };
      console.log(
        JSON.stringify({
          phase: "runtime-props-native-profile",
          samples: 100,
          coldWallMs,
          warmWallMs,
          cold: cold.profile,
          warm: warmSamples[0],
        }),
      );

      server = await createServer({
        root,
        configFile: false,
        resolve: {
          alias: { vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")) },
        },
        plugins: [vize(), musea({ include: ["src/**/*.art.vue"] })],
        server: { host: "127.0.0.1", port: 0 },
      });
      await server.listen();
      const address = server.httpServer!.address();
      assert.ok(address && typeof address !== "string");
      const gallery = `http://127.0.0.1:${address.port}/__musea__/`;
      const endpoint = `${gallery}api/arts/${encodeURIComponent(artPath)}`;
      const api = async (suffix: string) => {
        const response = await fetch(`${endpoint}/${suffix}`);
        assert.equal(response.status, 200);
        return response.json();
      };
      const analysis = await api("analysis");
      observations.analysis = analysis;
      assert.deepEqual(analysis.props, cold.props);
      const palette = await api("palette");
      observations.palette = palette;
      const controls = new Map<string, any>(
        palette.controls.map((control: any) => [control.name, control]),
      );
      assert.deepEqual(
        [...controls.keys()],
        ["count", "label", "code", "enabled", "items", "data", "callback", "opaque"],
      );
      const expectedControls = {
        label: "text",
        code: "text",
        count: "number",
        enabled: "boolean",
        items: "array",
        data: "object",
        callback: "function",
        opaque: "object",
      };
      for (const [name, kind] of Object.entries(expectedControls))
        assert.equal(controls.get(name).control, kind, name);
      assert.equal(
        controls.get("label").default_value,
        "Variant",
        "authored variant values take precedence",
      );
      assert.equal(controls.get("count").default_value, 9);
      for (const [name, value] of Object.entries(expectedRuntimeDefaults)) {
        if (name !== "label" && name !== "count")
          assert.deepEqual(controls.get(name).default_value, value, name);
      }
      assert.deepEqual(JSON.parse(palette.json).controls, palette.controls);
      assert.deepEqual(
        ts.transpileModule(palette.typescript, { reportDiagnostics: true }).diagnostics,
        [],
      );

      browser = await chromium.launch();
      const page = await browser.newPage();
      page.on("pageerror", (error) => errors.push(String(error)));
      await page.goto(gallery);
      await page.locator(".art-item").filter({ hasText: "Runtime Controls" }).first().click();
      await page.locator(".tab-btn").filter({ hasText: "Props" }).click();
      await page.getByRole("textbox", { name: "code", exact: true }).waitFor();
      assert.equal(
        await page.getByRole("textbox", { name: "code", exact: true }).inputValue(),
        "007",
      );
      await page.getByRole("note").filter({ hasText: "Function props retain" }).waitFor();
      const preview = page.frameLocator('iframe[title="Live component preview"]');
      await preview.locator("[data-runtime-probe]").waitFor();
      await page.waitForFunction(() => {
        const frame = document.querySelector<HTMLIFrameElement>(
          'iframe[title="Live component preview"]',
        );
        return frame?.contentDocument?.querySelector("[data-code]")?.textContent === "007";
      });
      assert.equal(await preview.locator("[data-callback]").textContent(), "function retained");
      assert.equal(await preview.locator("[data-opaque]").textContent(), "2");
      const currentValues = JSON.parse(await page.locator(".props-json-code code").innerText());
      assert.equal(Object.hasOwn(currentValues, "callback"), false);
      assert.equal(Object.hasOwn(currentValues, "opaque"), false);
      await page.getByRole("textbox", { name: "label", exact: true }).fill("Edited 🎨");
      await page.waitForFunction(
        () =>
          document
            .querySelector<HTMLIFrameElement>('iframe[title="Live component preview"]')
            ?.contentDocument?.querySelector("[data-label]")?.textContent === "Edited 🎨",
      );
      observations.currentValues = currentValues;
      await page.screenshot({ path: path.join(output, "runtime-controls.png"), fullPage: true });

      const editedSource = runtimeComponent.replace('default: "007"', 'default: "008"');
      await writeFile(componentPath, editedSource);
      const changed = await api("analysis");
      observations.changed = changed;
      assert.equal(changed.props.find((prop: any) => prop.name === "code").default_value, "008");
      const changedProfile = binding.analyzeSfc(editedSource, {
        filename: componentPath,
        profile: true,
      });
      assert.equal(
        changedProfile.profile.cacheHit,
        true,
        "HTTP and public export share the same native cache",
      );
      assert.equal(await readFile(componentPath, "utf8"), editedSource);
      assert.deepEqual(errors, []);
    } finally {
      observations.errors = errors;
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify(observations, null, 2),
      );
      await browser?.close();
      await server?.close();
    }
  },
);
