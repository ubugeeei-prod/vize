import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createServer as createTcpServer } from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Page } from "playwright";
import vize from "../../vite/src/index.ts";
import { musea } from "./plugin/index.ts";

export const repository = fileURLToPath(new URL("../../../../", import.meta.url));
export const variants = {
  Controls: [
    ["Default", "Ordinary"],
    ["Custom theme", "Custom"],
  ],
  Keys: [
    ["__proto__", "Proto"],
    ["constructor", "Constructor"],
    ["hasOwnProperty", "Own property"],
  ],
} as const;

export async function unusedPort(): Promise<number> {
  const listener = createTcpServer();
  await new Promise<void>((resolve, reject) => {
    listener.once("error", reject);
    listener.listen(0, "127.0.0.1", resolve);
  });
  const address = listener.address();
  assert.ok(address && typeof address !== "string");
  await new Promise<void>((resolve, reject) =>
    listener.close((error) => (error ? reject(error) : resolve())),
  );
  return address.port;
}

export async function fixture(surface: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), `musea-native-literal-${surface}-`));
  const output = path.join(repository, "artifacts/musea-native-literal-variants", surface);
  await mkdir(output, { recursive: true });
  await mkdir(path.join(root, "src"));
  const corpus = JSON.parse(
    await readFile(
      path.join(repository, "tests/_fixtures/differential/musea/literal-variant-keys.json"),
      "utf8",
    ),
  );
  const sources: unknown[] = [];
  for (const input of corpus.sources as Array<{ path: string; sha256: string }>) {
    const bytes = await readFile(path.join(repository, input.path));
    assert.equal(createHash("sha256").update(bytes).digest("hex"), input.sha256);
    await cp(path.join(repository, input.path), path.join(root, "src", path.basename(input.path)));
    sources.push({ ...input, bytes: bytes.toString() });
  }
  await writeFile(path.join(root, "index.html"), "<!doctype html><html><body></body></html>");
  const config = {
    root,
    configFile: false as const,
    resolve: {
      alias: { vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")) },
    },
    plugins: [
      vize(),
      musea({
        include: ["src/**/*.art.vue"],
        vrt: {
          viewports: [{ name: "authored", width: 320, height: 180 }],
          snapshotDir: "reviewed-baselines",
          threshold: 0,
        },
      }),
    ],
  };
  return {
    root,
    output,
    sources,
    config,
    async retain() {
      await cp(root, path.join(output, "project"), { recursive: true });
      await rm(root, { recursive: true, force: true });
    },
  };
}

export async function previews(page: Page, title: keyof typeof variants) {
  await page.locator(".art-item").filter({ hasText: title }).click();
  for (let index = 0; index < variants[title].length; index++) {
    await page
      .frameLocator(".variant-card iframe")
      .nth(index)
      .getByRole("button", { name: variants[title][index][1], exact: true })
      .waitFor({ timeout: 15_000 });
  }
  return page.evaluate(() =>
    [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")].map((frame) => ({
      url: frame.src,
      variant: frame.contentDocument?.querySelector("[data-variant]")?.getAttribute("data-variant"),
      buttons: [...(frame.contentDocument?.querySelectorAll("button") ?? [])].map(
        (button) => button.textContent,
      ),
      unresolved: frame.contentDocument?.querySelectorAll("museacomponent").length,
    })),
  );
}

export async function capture(page: Page, output: string, phase: string) {
  const response = page.waitForResponse((item) => item.url().endsWith("/api/run-vrt"));
  await page.getByRole("button", { name: "Run VRT", exact: true }).click();
  const actual = await response;
  const raw = await actual.text();
  const destination = path.join(output, phase);
  await mkdir(destination);
  await writeFile(path.join(destination, "response.json"), raw);
  assert.equal(actual.status(), 200, raw);
  const data = JSON.parse(raw);
  assert.equal(data.success, true, raw);
  for (let index = 0; index < data.results.length; index++) {
    await cp(data.results[index].snapshotPath, path.join(destination, `${index}.png`));
  }
  await cp(data.artifacts.jsonReportPath, path.join(destination, "report.json"));
  await cp(data.artifacts.htmlReportPath, path.join(destination, "report.html"));
  return data as {
    summary: { total: number; new: number; passed: number; failed: number };
    results: Array<{ variantName: string; snapshotPath: string; diffPercentage: number }>;
  };
}
