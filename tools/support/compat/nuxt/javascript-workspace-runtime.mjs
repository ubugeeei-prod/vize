// Actual pinned Nuxt/Vite builds and whole owned component outputs, no generated-code rewrite.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { save } from "./javascript-workspace-project.mjs";
import { verifyNuxtSourceBindingEvents } from "./source-binding.mjs";

function command(context, name, args, adapter) {
  const log = path.join(context.artifacts, `${name}.log`);
  const fd = fs.openSync(log, "w");
  let result;
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
async function port() {
  const socket = net.createServer();
  await new Promise((resolve, reject) => {
    socket.once("error", reject);
    socket.listen(0, "127.0.0.1", resolve);
  });
  const value = socket.address().port;
  await new Promise((resolve) => socket.close(resolve));
  return value;
}
export async function browserOutput(browser, url, expected, directory) {
  const page = await browser.newPage();
  const pageErrors = [],
    consoleMessages = [],
    responses = [];
  page.on("pageerror", (error) => pageErrors.push(String(error)));
  page.on("console", (message) =>
    consoleMessages.push({ type: message.type(), text: message.text() }),
  );
  page.on("response", (response) =>
    responses.push({ url: response.url(), status: response.status() }),
  );
  try {
    const response = await page.goto(url, { waitUntil: "networkidle" });
    assert.equal(response.status(), 200);
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
async function vite(context, browser) {
  const application = path.join(context.project, "apps/vite");
  const host = path.join(application, "workspace-runtime.mjs");
  fs.writeFileSync(
    host,
    'export { build, preview } from "vite";\nexport { createSSRApp } from "vue";\nexport { renderToString } from "vue/server-renderer";\n',
  );
  const api = await import(pathToFileURL(host).href);
  const results = [];
  for (const adapter of ["stock", "source"]) {
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
    save(directory, "client-bundle.json", client.output);
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
    save(directory, "ssr-bundle.json", ssr.output);
    const link = path.join(directory, "ssr/node_modules");
    fs.symlinkSync(path.join(context.project, "node_modules"), link);
    let html;
    try {
      const { default: component } = await import(
        pathToFileURL(path.join(directory, "ssr/entry.mjs")).href
      );
      html = await api.renderToString(api.createSSRApp(component));
      fs.writeFileSync(path.join(directory, "ssr.html"), html);
      assert.equal(html, context.corpus.expected.Vite.initial);
    } finally {
      fs.unlinkSync(link);
    }
    let server;
    try {
      server = await api.preview({
        ...common,
        configFile: false,
        build: { outDir: path.join(directory, "client") },
        preview: { host: "127.0.0.1", port: 0 },
      });
      const address = server.httpServer.address();
      const rendered = await browserOutput(
        browser,
        `http://127.0.0.1:${address.port}/`,
        context.corpus.expected.Vite,
        directory,
      );
      results.push({ adapter, html, ...rendered });
    } finally {
      await server?.close();
    }
  }
  assert.deepEqual(results[0].html, results[1].html);
  assert.deepEqual(results[0].before, results[1].before);
  assert.deepEqual(results[0].after, results[1].after);
  return results;
}
async function nuxt(context, browser) {
  const application = path.join(context.project, "apps/nuxt");
  const require = createRequire(path.join(application, "package.json"));
  const cli = path.join(path.dirname(require.resolve("nuxt/package.json")), "bin/nuxt.mjs");
  command(context, "source-prepare", [cli, "prepare"], "source");
  if (context.cohort.id === "nuxt3") {
    assert.equal(
      JSON.parse(fs.readFileSync(path.join(application, "tsconfig.json"))).extends,
      "./.nuxt/tsconfig.json",
    );
    assert.ok(fs.existsSync(path.join(application, ".nuxt/tsconfig.json")));
  } else {
    const configured = JSON.parse(fs.readFileSync(path.join(application, "tsconfig.json")));
    assert.deepEqual(
      configured.references.map((row) => row.path),
      [
        "./.nuxt/tsconfig.app.json",
        "./.nuxt/tsconfig.server.json",
        "./.nuxt/tsconfig.shared.json",
        "./.nuxt/tsconfig.node.json",
      ],
    );
    for (const row of configured.references)
      assert.ok(fs.existsSync(path.resolve(application, row.path)));
  }
  assert.ok(fs.existsSync(path.join(application, ".nuxt/imports.d.ts")));
  const results = [];
  for (const adapter of ["stock", "source"]) {
    command(context, `${adapter}-build`, [cli, "build"], adapter);
    const directory = path.join(context.artifacts, adapter);
    fs.mkdirSync(directory);
    const serverPort = await port();
    const fd = fs.openSync(path.join(directory, "server.log"), "w");
    const server = spawn(process.execPath, [path.join(application, ".output/server/index.mjs")], {
      cwd: application,
      env: { ...process.env, HOST: "127.0.0.1", PORT: String(serverPort), NO_COLOR: "1" },
      stdio: ["ignore", fd, fd],
    });
    const exited = new Promise((resolve) =>
      server.once("exit", (code, signal) => resolve({ code, signal })),
    );
    try {
      let response;
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
      assert.equal(matches[0][0], context.corpus.expected.Nuxt.initial);
      const rendered = await browserOutput(
        browser,
        `http://127.0.0.1:${serverPort}/`,
        context.corpus.expected.Nuxt,
        directory,
      );
      results.push({ adapter, html: matches[0][0], ...rendered });
    } finally {
      if (server.exitCode === null) server.kill("SIGTERM");
      save(directory, "server-exit.json", await exited);
      fs.closeSync(fd);
    }
  }
  assert.deepEqual(results[0].html, results[1].html);
  assert.deepEqual(results[0].before, results[1].before);
  assert.deepEqual(results[0].after, results[1].after);
  return results;
}
export async function runtime(root, context) {
  const { chromium } = createRequire(path.join(root, "tests/package.json"))("@playwright/test");
  const browser = await chromium.launch();
  let result;
  try {
    result = context.cohort.nuxt ? await nuxt(context, browser) : await vite(context, browser);
  } finally {
    await browser.close();
  }
  const events = fs
    .readFileSync(context.binding.calls, "utf8")
    .trim()
    .split("\n")
    .map((row) => JSON.parse(row));
  const sourceBinding = verifyNuxtSourceBindingEvents(context.binding, events);
  save(context.artifacts, "runtime.json", { result, sourceBinding });
  return result;
}
