// Actual pinned Nuxt/Vite builds and whole owned component outputs, no generated-code rewrite.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { asArray, asStringArray, at, parseJson, readJson } from "../../json-value.ts";
import {
  freePort,
  importChecked,
  importDefault,
  loadChromium,
  previewPort,
  viteRuntimeFunctions,
} from "./host-types.ts";
import type { Browser, PreviewServer, ViteRuntimeHost } from "./host-types.ts";
import { save } from "./javascript-workspace-project.ts";
import type { RenderedPair, WorkspaceContext } from "./javascript-workspace-project.ts";
import { verifyNuxtSourceBindingEvents } from "./source-binding.ts";

type Adapter = "stock" | "source";
interface BrowserEvents {
  pageErrors: string[];
  consoleMessages: { type: string; text: string }[];
  responses: { url: string; status: number }[];
}
interface RenderedRow {
  adapter: Adapter;
  html: string;
  before: string;
  after: string;
}

function command(context: WorkspaceContext, name: string, args: string[], adapter: Adapter): void {
  const log = path.join(context.artifacts, `${name}.log`);
  const fd = fs.openSync(log, "w");
  let result: ReturnType<typeof spawnSync>;
  try {
    result = spawnSync(process.execPath, args, {
      cwd: path.join(context.project, "apps/nuxt"),
      env: { ...context.environment, USE_VIZE: adapter === "source" ? "1" : "0" },
      stdio: ["ignore", fd, fd],
      timeout: 180_000,
    });
  } finally {
    fs.closeSync(fd);
  }
  save(context.artifacts, `${name}-process.json`, {
    args,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, `retained actual ${name}.log`);
}
export async function browserOutput(
  browser: Browser,
  url: string,
  expected: RenderedPair,
  directory: string,
): Promise<{ before: string; after: string }> {
  const page = await browser.newPage();
  const { pageErrors, consoleMessages, responses }: BrowserEvents = {
    pageErrors: [],
    consoleMessages: [],
    responses: [],
  };
  page.on("pageerror", (error) => pageErrors.push(String(error)));
  page.on("console", (message) =>
    consoleMessages.push({ type: message.type(), text: message.text() }),
  );
  page.on("response", (response) =>
    responses.push({ url: response.url(), status: response.status() }),
  );
  try {
    const response = await page.goto(url, { waitUntil: "networkidle" });
    assert.equal(response?.status(), 200);
    const before = await page.locator('[data-test="workspace"]').evaluate((node) => node.outerHTML);
    await page.locator('[data-test="counter"]').click();
    await page.getByRole("button", { name: "Count: 3" }).waitFor();
    const after = await page.locator('[data-test="workspace"]').evaluate((node) => node.outerHTML);
    const result = {
      before,
      after,
      documentHtml: await page.content(),
      pageErrors,
      consoleMessages,
      responses,
    };
    save(directory, "browser.json", result);
    assert.equal(before, expected.initial);
    assert.equal(after, expected.clicked);
    assert.deepEqual(pageErrors, []);
    assert.deepEqual(
      consoleMessages.filter((row) => row.type === "error"),
      [],
    );
    assert.ok(responses.every((row) => row.status < 400));
    return { before, after };
  } finally {
    save(directory, "browser-events.json", { pageErrors, consoleMessages, responses });
    await page.close();
  }
}
async function vite(context: WorkspaceContext, browser: Browser): Promise<RenderedRow[]> {
  const application = path.join(context.project, "apps/vite");
  const host = path.join(application, "workspace-runtime.mjs");
  fs.writeFileSync(
    host,
    'export { build, preview } from "vite";\nexport { createSSRApp } from "vue";\nexport { renderToString } from "vue/server-renderer";\n',
  );
  const api = await importChecked<ViteRuntimeHost>(host, viteRuntimeFunctions);
  const results: RenderedRow[] = [];
  for (const adapter of ["stock", "source"] as const) {
    process.env.USE_VIZE = adapter === "source" ? "1" : "0";
    const directory = path.join(context.artifacts, adapter);
    fs.mkdirSync(directory);
    const common = {
      root: application,
      configFile: path.join(application, "vite.config.mjs"),
      logLevel: "warn",
    };
    const client = await api.build({
      ...common,
      build: { minify: false, outDir: path.join(directory, "client"), emptyOutDir: true },
    });
    save(directory, "client-bundle.json", at(client, "output"));
    const ssr = await api.build({
      ...common,
      build: {
        minify: false,
        outDir: path.join(directory, "ssr"),
        emptyOutDir: true,
        ssr: path.join(application, "src/ssr.mjs"),
        rollupOptions: {
          external: ["vue", "vue/server-renderer", "@vue/server-renderer"],
          output: { entryFileNames: "entry.mjs", chunkFileNames: "[name]-[hash].mjs" },
        },
      },
    });
    save(directory, "ssr-bundle.json", at(ssr, "output"));
    const link = path.join(directory, "ssr/node_modules");
    fs.symlinkSync(path.join(context.project, "node_modules"), link);
    let html: string;
    try {
      const component = await importDefault(path.join(directory, "ssr/entry.mjs"));
      html = await api.renderToString(api.createSSRApp(component));
      fs.writeFileSync(path.join(directory, "ssr.html"), html);
      assert.equal(html, context.corpus.expected.Vite.initial);
    } finally {
      fs.unlinkSync(link);
    }
    let server: PreviewServer | undefined;
    try {
      server = await api.preview({
        ...common,
        configFile: false,
        build: { outDir: path.join(directory, "client") },
        preview: { host: "127.0.0.1", port: 0 },
      });
      const rendered = await browserOutput(
        browser,
        `http://127.0.0.1:${previewPort(server)}/`,
        context.corpus.expected.Vite,
        directory,
      );
      results.push({ adapter, html, ...rendered });
    } finally {
      await server?.close();
    }
  }
  assertSameRows(results);
  return results;
}
function assertSameRows(results: readonly RenderedRow[]): void {
  const [stock, source] = results;
  assert.ok(stock && source);
  assert.deepEqual(stock.html, source.html);
  assert.deepEqual(stock.before, source.before);
  assert.deepEqual(stock.after, source.after);
}
async function nuxt(context: WorkspaceContext, browser: Browser): Promise<RenderedRow[]> {
  const application = path.join(context.project, "apps/nuxt");
  const require = createRequire(path.join(application, "package.json"));
  const cli = path.join(path.dirname(require.resolve("nuxt/package.json")), "bin/nuxt.mjs");
  command(context, "source-prepare", [cli, "prepare"], "source");
  if (context.cohort.id === "nuxt3") {
    assert.equal(
      at(readJson(path.join(application, "tsconfig.json")), "extends"),
      "./.nuxt/tsconfig.json",
    );
    assert.ok(fs.existsSync(path.join(application, ".nuxt/tsconfig.json")));
  } else {
    const references = asStringArray(
      asArray(at(readJson(path.join(application, "tsconfig.json")), "references")).map((row) =>
        at(row, "path"),
      ),
      "references",
    );
    assert.deepEqual(references, [
      "./.nuxt/tsconfig.app.json",
      "./.nuxt/tsconfig.server.json",
      "./.nuxt/tsconfig.shared.json",
      "./.nuxt/tsconfig.node.json",
    ]);
    for (const reference of references)
      assert.ok(fs.existsSync(path.resolve(application, reference)));
  }
  assert.ok(fs.existsSync(path.join(application, ".nuxt/imports.d.ts")));
  const results: RenderedRow[] = [];
  for (const adapter of ["stock", "source"] as const) {
    command(context, `${adapter}-build`, [cli, "build"], adapter);
    const directory = path.join(context.artifacts, adapter);
    fs.mkdirSync(directory);
    const serverPort = await freePort();
    const fd = fs.openSync(path.join(directory, "server.log"), "w");
    const server = spawn(process.execPath, [path.join(application, ".output/server/index.mjs")], {
      cwd: application,
      env: { ...process.env, HOST: "127.0.0.1", PORT: String(serverPort), NO_COLOR: "1" },
      stdio: ["ignore", fd, fd],
    });
    const exited = new Promise<{ code: number | null; signal: string | null }>((resolve) =>
      server.once("exit", (code, signal) => resolve({ code, signal })),
    );
    try {
      let response: Response | undefined;
      for (let attempt = 0; attempt < 100; attempt++) {
        try {
          response = await fetch(`http://127.0.0.1:${serverPort}/`, {
            signal: AbortSignal.timeout(1000),
          });
          break;
        } catch {
          assert.equal(server.exitCode, null);
          await new Promise((resolve) => setTimeout(resolve, 100));
        }
      }
      assert.ok(response);
      assert.equal(response.status, 200);
      const documentHtml = await response.text();
      fs.writeFileSync(path.join(directory, "ssr.html"), documentHtml);
      const matches = [...documentHtml.matchAll(/<main data-test="workspace">[\s\S]*?<\/main>/g)];
      assert.equal(matches.length, 1);
      const html = matches[0]?.[0] ?? "";
      assert.equal(html, context.corpus.expected.Nuxt.initial);
      const rendered = await browserOutput(
        browser,
        `http://127.0.0.1:${serverPort}/`,
        context.corpus.expected.Nuxt,
        directory,
      );
      results.push({ adapter, html, ...rendered });
    } finally {
      if (server.exitCode === null) server.kill("SIGTERM");
      save(directory, "server-exit.json", await exited);
      fs.closeSync(fd);
    }
  }
  assertSameRows(results);
  return results;
}
export async function runtime(root: string, context: WorkspaceContext): Promise<RenderedRow[]> {
  const browser = await loadChromium(root).launch();
  let result: RenderedRow[];
  try {
    result = context.cohort.nuxt ? await nuxt(context, browser) : await vite(context, browser);
  } finally {
    await browser.close();
  }
  const events = fs
    .readFileSync(context.binding.calls, "utf8")
    .trim()
    .split("\n")
    .map((row): unknown => parseJson(row));
  const sourceBinding = verifyNuxtSourceBindingEvents(context.binding, events);
  save(context.artifacts, "runtime.json", { result, sourceBinding });
  return result;
}
