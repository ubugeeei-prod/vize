// #8631: first Nuxt dev response, actual CSS transport, and hydration stability.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixture = path.join(root, "tests/_fixtures/_projects/nuxt-dev-ssr-style");
const artifacts = path.resolve(process.argv[2] ?? path.join(os.tmpdir(), "vize-nuxt-dev-style"));
const dependencies = fs.realpathSync(process.argv[3] ?? path.join(fixture, "node_modules"));
const requireTests = createRequire(path.join(root, "tests/package.json"));
const { chromium } = requireTests("@playwright/test");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const inputs = ["package.json", "nuxt.config.ts", "app/app.vue", "app/pages/index.vue"];
const inputHashes = Object.fromEntries(
  inputs.map((name) => [name, sha256(fs.readFileSync(path.join(fixture, name)))]),
);
const write = (name, value) =>
  fs.writeFileSync(path.join(artifacts, name), JSON.stringify(value, null, 2) + "\n");
fs.mkdirSync(artifacts, { recursive: true });
const versions = {};
for (const [name, expected] of [
  ["nuxt", "4.5.0"],
  ["vue", "3.5.40"],
  ["@vizejs/nuxt", "0.443.0"],
  ["@vizejs/vite-plugin", "0.443.0"],
  ["@vizejs/native", "0.443.0"],
]) {
  const actual = JSON.parse(
    fs.readFileSync(path.join(dependencies, name, "package.json"), "utf8"),
  ).version;
  assert.equal(actual, expected, `${name} must use the pinned public version`);
  versions[name] = actual;
}
function files(dir) {
  return fs
    .readdirSync(dir, { withFileTypes: true })
    .flatMap((entry) =>
      entry.isDirectory() ? files(path.join(dir, entry.name)) : [path.join(dir, entry.name)],
    );
}
const publishedPackages = Object.fromEntries(
  ["nuxt", "vite-plugin"]
    .flatMap((name) => files(path.join(dependencies, "@vizejs", name)))
    .map((filename) => [filename, sha256(fs.readFileSync(filename))]),
);

function copyProject(variant) {
  const workspace = path.join(artifacts, variant);
  assert.equal(fs.existsSync(workspace), false, "use a new evidence directory");
  for (const name of inputs) {
    const target = path.join(workspace, name);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(fixture, name), target);
  }
  if (variant === "stock") {
    const config = path.join(workspace, "nuxt.config.ts");
    fs.writeFileSync(
      config,
      fs.readFileSync(config, "utf8").replace('modules: ["@vizejs/nuxt"]', "modules: []"),
    );
  }
  const modules = path.join(workspace, "node_modules");
  fs.mkdirSync(path.join(modules, "@vizejs"), { recursive: true });
  for (const name of fs.readdirSync(dependencies)) {
    if (["@vizejs", ".cache"].includes(name)) continue;
    fs.symlinkSync(path.join(dependencies, name), path.join(modules, name), "dir");
  }
  fs.mkdirSync(path.join(modules, ".cache"));
  for (const name of fs.readdirSync(path.join(dependencies, "@vizejs"))) {
    const installed = path.join(dependencies, "@vizejs", name);
    const target = path.join(modules, "@vizejs", name);
    if (!["nuxt", "vite-plugin"].includes(name)) {
      fs.symlinkSync(installed, target, "dir");
      continue;
    }
    fs.cpSync(installed, target, { recursive: true });
    if (variant === "candidate") {
      const source = path.join(
        root,
        name === "nuxt" ? "npm/framework/nuxt/dist" : "npm/builder/vite/dist",
      );
      assert.equal(
        fs.existsSync(path.join(source, "index.mjs")),
        true,
        "build candidate adapters first",
      );
      fs.rmSync(path.join(target, "dist"), { recursive: true });
      fs.cpSync(source, path.join(target, "dist"), { recursive: true });
    }
  }
  return workspace;
}

async function waitForTcp(port, child) {
  for (let attempt = 0; attempt < 600; attempt++) {
    assert.equal(child.exitCode, null, "dev server exited before listening");
    const ready = await new Promise((resolve) => {
      const socket = net.connect({ host: "127.0.0.1", port });
      socket.once("connect", () => {
        socket.destroy();
        resolve(true);
      });
      socket.once("error", () => {
        socket.destroy();
        resolve(false);
      });
    });
    if (ready) return;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("dev server did not listen within 60 seconds");
}

async function observe(page) {
  return page.locator(".card").evaluate((card) => {
    const css = getComputedStyle(card),
      rect = card.getBoundingClientRect();
    return {
      background: css.backgroundColor,
      color: css.color,
      border: css.borderTopColor,
      borderWidth: css.borderTopWidth,
      padding: css.padding,
      fontSize: css.fontSize,
      lineHeight: css.lineHeight,
      width: rect.width,
      height: rect.height,
      hydrated: card.getAttribute("data-hydrated"),
      links: [...document.querySelectorAll('head link[rel="stylesheet"]')].map((link) => link.href),
    };
  });
}

const browser = await chromium.launch();
const observations = {};
try {
  for (const [index, variant] of ["stock", "published", "candidate"].entries()) {
    const workspace = copyProject(variant),
      port = Number(process.env.VIZE_NUXT_DEV_STYLE_PORT ?? 18931) + index;
    const stream = fs.openSync(path.join(artifacts, `${variant}-dev.log`), "w");
    const child = spawn(
      process.execPath,
      [
        path.join(workspace, "node_modules/nuxt/bin/nuxt.mjs"),
        "dev",
        "--host",
        "127.0.0.1",
        "--port",
        String(port),
      ],
      {
        cwd: workspace,
        env: { ...process.env, NO_COLOR: "1", NUXT_TELEMETRY_DISABLED: "1" },
        stdio: ["ignore", stream, stream],
      },
    );
    const context = await browser.newContext({
      javaScriptEnabled: false,
      viewport: { width: 900, height: 600 },
    });
    try {
      await waitForTcp(port, child);
      const page = await context.newPage(),
        responses = [];
      page.on("response", (response) => {
        if (response.request().resourceType() === "stylesheet") responses.push(response);
      });
      const response = await page.goto(`http://127.0.0.1:${port}/`, {
        waitUntil: "load",
        timeout: 120_000,
      });
      assert.equal(response.status(), 200);
      const initialHtml = await response.text();
      fs.writeFileSync(path.join(artifacts, `${variant}-initial.html`), initialHtml);
      const initial = await observe(page);
      assert.equal(initial.hydrated, "false", "no application JavaScript ran");
      const css = await Promise.all(
        responses.map(async (entry) => ({
          url: entry.url(),
          status: entry.status(),
          contentType: entry.headers()["content-type"],
          body: await entry.text(),
        })),
      );
      observations[variant] = { initial, initialHtml, css };
      write(`${variant}-observation.json`, observations[variant]);
      if (variant === "published") {
        assert.equal(initial.background, "rgba(0, 0, 0, 0)", "0.443.0 reproduces dev SSR flash");
        assert.equal(initial.links.length, 0);
      } else {
        assert.equal(initial.background, "rgb(18, 24, 30)");
        assert.equal(initial.border, "rgb(40, 200, 90)");
        assert.equal(initial.borderWidth, "3px");
        assert.equal(initial.padding, "24px");
        assert.equal(initial.width, 420);
        assert.equal(css.length > 0, true, "initial HTML must request its page stylesheet");
        for (const entry of css) {
          assert.equal(entry.status, 200, "real stylesheet GET must work");
          assert.match(entry.contentType, /text\/css/);
        }
      }
      if (variant === "candidate") {
        const hydratedContext = await browser.newContext({ viewport: { width: 900, height: 600 } });
        try {
          const hydratedPage = await hydratedContext.newPage();
          await hydratedPage.goto(`http://127.0.0.1:${port}/`, { waitUntil: "load" });
          await hydratedPage.locator('.card[data-hydrated="true"]').waitFor();
          const hydrated = await observe(hydratedPage);
          observations[variant].hydrated = hydrated;
          const { links: initialLinks, hydrated: initialState, ...before } = initial;
          const { links: hydratedLinks, hydrated: hydratedState, ...after } = hydrated;
          assert.equal(initialLinks.length > 0 && hydratedLinks.length > 0, true);
          assert.equal(initialState, "false");
          assert.equal(hydratedState, "true");
          assert.deepEqual(after, before, "hydration cannot alter authored CSS or card dimensions");
          write(`${variant}-observation.json`, observations[variant]);
        } finally {
          await hydratedContext.close();
        }
      }
      console.log(`${variant}: first dev SSR response and stylesheet transport verified`);
    } finally {
      await context.close();
      child.kill("SIGTERM");
      await new Promise((resolve) => {
        if (child.exitCode !== null) resolve();
        else child.once("exit", resolve);
      });
      fs.closeSync(stream);
    }
  }
  const { links, hydrated, ...candidate } = observations.candidate.initial;
  const { links: stockLinks, hydrated: stockHydrated, ...stock } = observations.stock.initial;
  assert.equal(links.length > 0 && stockLinks.length > 0, true);
  assert.equal(hydrated, stockHydrated);
  assert.deepEqual(candidate, stock, "candidate matches stock Vue's authored CSS and geometry");
  for (const [filename, digest] of Object.entries(publishedPackages))
    assert.equal(sha256(fs.readFileSync(filename)), digest);
  const candidateFiles = Object.fromEntries(
    ["npm/framework/nuxt/dist", "npm/builder/vite/dist"]
      .flatMap((dir) => files(path.join(root, dir)))
      .map((filename) => [path.relative(root, filename), sha256(fs.readFileSync(filename))]),
  );
  write("proof.json", {
    issue: 8631,
    sourceSha: process.env.GITHUB_SHA ?? null,
    node: process.version,
    versions,
    inputHashes,
    candidateFiles,
    browser: browser.version(),
    publishedPackageBytesPreserved: true,
    nativeSourceBuilt: false,
    variants: Object.keys(observations),
  });
} finally {
  await browser.close();
}
