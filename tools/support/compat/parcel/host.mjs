import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs/promises";
import http from "node:http";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { summarizeNativeEvents } from "./native-analysis.mjs";
import { validateNativeTargetCustody } from "./native-target.mjs";

const repository = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const arguments_ = process.argv.slice(2);
const recoveryFrontier = arguments_.includes("--recovery-frontier");
const [proofArgument, providerArgument] = arguments_.filter(
  (argument) => argument !== "--recovery-frontier",
);
const proof = path.resolve(proofArgument ?? "target/parcel-host");
// This explicit local-only mode cannot produce a source qualification receipt.
const preflightProvider = providerArgument && path.resolve(providerArgument);
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const processes = [];
const browserEvents = [];
const watchEvents = [];
const started = performance.now();
assert.equal(await fs.stat(proof).catch(() => null), null, "evidence destination must be absent");
await fs.mkdir(proof, { recursive: true });
const write = (name, value) =>
  fs.writeFile(path.join(proof, name), JSON.stringify(value, null, 2) + "\n");

function run(command, args, cwd) {
  const result = spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    env: process.env,
  });
  processes.push({
    command,
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    stdout: result.stdout,
    stderr: result.stderr,
  });
  if (result.status !== 0)
    throw new Error(`${command} exited ${result.status}: ${result.stderr}\n${result.stdout}`);
  return result.stdout;
}

async function files(directory) {
  const result = [];
  for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
    const name = path.join(directory, entry.name);
    if (entry.isDirectory()) result.push(...(await files(name)));
    else if (entry.isFile()) result.push(name);
  }
  return result.sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
}

async function stagePackage(directory, output, nativeVersion) {
  await fs.mkdir(output, { recursive: true });
  const manifest = JSON.parse(await fs.readFile(path.join(directory, "package.json"), "utf8"));
  for (const name of manifest.files) {
    const input = path.join(directory, name);
    if (await fs.stat(input).catch(() => null))
      await fs.cp(input, path.join(output, name), { recursive: true });
  }
  if (manifest.name === "@vizejs/native") {
    const images = (await fs.readdir(directory)).filter((name) => name.endsWith(".node"));
    assert.equal(images.length, 1, "exactly one physical native image is required");
    for (const name of images)
      await fs.copyFile(path.join(directory, name), path.join(output, name));
    manifest.files = [...manifest.files, ...images];
    delete manifest.optionalDependencies;
  } else {
    for (const name of Object.keys(manifest.dependencies)) {
      if (name.startsWith("@vizejs/")) manifest.dependencies[name] = nativeVersion;
    }
    const sourceRequire = createRequire(path.join(directory, "package.json"));
    for (const [name, version] of Object.entries(manifest.dependencies)) {
      if (!version.startsWith("catalog:")) continue;
      const installed = sourceRequire(`${name}/package.json`);
      assert.equal(installed.name, name);
      manifest.dependencies[name] = installed.version;
    }
  }
  delete manifest.devDependencies;
  delete manifest.scripts;
  await fs.writeFile(path.join(output, "package.json"), JSON.stringify(manifest, null, 2) + "\n");
  const packed = JSON.parse(run("npm", ["pack", "--ignore-scripts", "--json"], output))[0];
  const tarball = path.join(output, packed.filename);
  const bytes = await fs.readFile(tarball);
  assert.equal(packed.integrity, `sha512-${createHash("sha512").update(bytes).digest("base64")}`);
  return { ...packed, path: tarball, sha256: sha(bytes) };
}

const source = (label = "Count", color = "tomato") =>
  `<!-- authored 😺 Unicode and CRLF -->\r\n<script lang="ts">\r\nglobalThis.parcelModuleEvaluations = (globalThis.parcelModuleEvaluations || 0) + 1;\r\n</script>\r\n<script setup lang="ts">\r\nimport { ref } from 'vue'\r\nimport { initialCount } from 'fixture-counter'\r\nconst n = ref<number>(initialCount)\r\nconst color = ${JSON.stringify(color)}\r\n</script>\r\n<template><button class="card" @click="n++">${label}: {{ n }}<img src="./icon.svg" alt="Fixture"></button><p class="external">External</p></template>\r\n<style scoped>.card { color: v-bind(color); background-image: url('./icon.svg'); }</style>\r\n<style scoped src="./styles/external é.css"></style>\r\n<style lang="scss" scoped>@use './styles/palette.scss' as p; .card { border: 2px solid p.$border; }</style>\r\n`;
const icon =
  '<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"><path fill="red" d="M0 0h8v8H0z"/></svg>';

async function makeApp(directory) {
  await fs.mkdir(path.join(directory, "styles"), { recursive: true });
  await fs.writeFile(
    path.join(directory, ".parcelrc"),
    JSON.stringify(
      {
        extends: "@parcel/config-default",
        reporters: ["./native-trace.cjs", "..."],
        transformers: {
          "*.vue": ["@vizejs/parcel-transformer"],
          "vize-component:*.vue": ["@vizejs/parcel-transformer"],
          "*.css": ["@vizejs/parcel-transformer/style", "..."],
        },
      },
      null,
      2,
    ),
  );
  await fs.writeFile(
    path.join(directory, "native-trace.cjs"),
    `const { Reporter } = require('@parcel/plugin');
const fs = require('node:fs/promises');
let tail = Promise.resolve();
module.exports = new Reporter({
  report({ event, options }) {
    const filename = options.env.VIZE_PARCEL_NATIVE_TRACE;
    if (!filename) return;
    if (event.type === 'trace' && event.name.startsWith('napi:')) {
      tail = tail.then(() => fs.appendFile(filename, JSON.stringify(event) + '\\n'));
    }
    return tail;
  }
});
`,
  );
  await fs.writeFile(path.join(directory, ".vizerc.parcel.json"), "{}\n");
  await fs.writeFile(
    path.join(directory, "index.html"),
    '<!doctype html><html><head><meta charset="utf-8"></head><body><div id="app"></div><script type="module" src="./main.js"></script></body></html>',
  );
  await fs.writeFile(
    path.join(directory, "main.js"),
    'import {createApp} from "vue"; import App from "./App.vue"; createApp(App).mount("#app");',
  );
  await fs.writeFile(
    path.join(directory, "entry-server.js"),
    'import {createSSRApp} from "vue"; import {renderToString} from "@vue/server-renderer"; import App from "./App.vue"; export default () => renderToString(createSSRApp(App));',
  );
  await fs.writeFile(path.join(directory, "counter.js"), "export const initialCount = 1;\n");
  await fs.writeFile(path.join(directory, "App.vue"), source());
  await fs.writeFile(path.join(directory, "icon.svg"), icon);
  await fs.writeFile(path.join(directory, "styles", "palette.scss"), "$border: rgb(12, 34, 56);\n");
  await fs.writeFile(
    path.join(directory, "styles", "external é.css"),
    ".external { color: rgb(22, 33, 44); background-image: url('../icon.svg'); }\n",
  );
}

async function captureBuild(name, result) {
  const directory = path.join(proof, name);
  await fs.mkdir(directory, { recursive: true });
  const bundles = [];
  const assets = new Map();
  for (const bundle of result.bundleGraph.getBundles()) {
    bundles.push({
      id: bundle.id,
      name: bundle.name,
      type: bundle.type,
      filePath: bundle.filePath,
      target: bundle.target,
    });
    bundle.traverseAssets((asset) => assets.set(asset.id, asset));
  }
  for (const [id, asset] of assets) {
    const content = await asset.getBuffer();
    const map = await asset.getMap();
    await fs.writeFile(path.join(directory, `${id}.${asset.type}`), content);
    if (map)
      await fs.writeFile(path.join(directory, `${id}.map.json`), JSON.stringify(map.toVLQ()));
  }
  await fs.writeFile(
    path.join(directory, "build.json"),
    JSON.stringify(
      {
        buildTime: result.buildTime,
        bundles,
        assets: [...assets].map(([id, asset]) => ({
          id,
          filePath: asset.filePath,
          type: asset.type,
          uniqueKey: asset.uniqueKey,
        })),
      },
      null,
      2,
    ),
  );
  return { bundles, assets };
}

async function serve(directory) {
  const server = http.createServer(async (request, response) => {
    try {
      const pathname = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
      const filename = path.resolve(directory, `.${pathname === "/" ? "/index.html" : pathname}`);
      assert.ok(filename.startsWith(directory + path.sep));
      const data = await fs.readFile(filename);
      response.setHeader(
        "Content-Type",
        filename.endsWith(".js")
          ? "text/javascript"
          : filename.endsWith(".css")
            ? "text/css"
            : filename.endsWith(".svg")
              ? "image/svg+xml"
              : "text/html",
      );
      response.end(data);
    } catch {
      response.statusCode = 404;
      response.end("missing");
    }
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  return { server, url: `http://127.0.0.1:${server.address().port}` };
}

async function freePort() {
  const server = http.createServer();
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const port = server.address().port;
  await new Promise((resolve) => server.close(resolve));
  return port;
}

async function eventually(read, expected, label) {
  const deadline = Date.now() + 30_000;
  let actual;
  do {
    actual = await read();
    if (expected(actual)) return actual;
    await new Promise((resolve) => setTimeout(resolve, 50));
  } while (Date.now() < deadline);
  throw new Error(`${label}: ${JSON.stringify(actual)}`);
}

let browser;
let watcher;
let staticServer;
let success = false;
let tracing = false;
try {
  for (const relative of [
    "npm/builder/parcel/src",
    "npm/builder/parcel/package.json",
    "npm/builder/parcel/vite.config.ts",
    "npm/builder/parcel/README.md",
    "tools/support/compat/parcel",
    ".github/workflows/parcel-transformer.yml",
  ]) {
    const destination = path.join(proof, "source", relative);
    await fs.mkdir(path.dirname(destination), { recursive: true });
    await fs.cp(path.join(repository, relative), destination, { recursive: true });
  }
  for (const name of ["NAPI_RS_NATIVE_LIBRARY_PATH", "NAPI_RS_FORCE_WASI"])
    assert.ok(!process.env[name], `${name} would bypass the packaged image`);
  const nativeDirectory = preflightProvider ?? path.join(repository, "npm/native");
  const nativeManifest = JSON.parse(
    await fs.readFile(path.join(nativeDirectory, "package.json"), "utf8"),
  );
  const native = await stagePackage(nativeDirectory, path.join(proof, "native-package"));
  const plugin = await stagePackage(
    path.join(repository, "npm/builder/parcel"),
    path.join(proof, "plugin-package"),
    nativeManifest.version,
  );
  const consumer = path.join(proof, "consumer 😺 space");
  await fs.cp(path.join(repository, "tools/support/compat/parcel/fixtures/host"), consumer, {
    recursive: true,
  });
  run("npm", ["ci", "--ignore-scripts", "--no-audit", "--no-fund"], consumer);
  const lock = sha(await fs.readFile(path.join(consumer, "package-lock.json")));
  run(
    "npm",
    [
      "install",
      "--ignore-scripts",
      "--no-save",
      "--package-lock=false",
      "--no-audit",
      "--no-fund",
      native.path,
      plugin.path,
    ],
    consumer,
  );
  if (!preflightProvider) {
    for (const name of ["dist/index.d.cts", "dist/style.d.cts"]) {
      assert.ok(
        plugin.files.some((file) => file.path === name),
        `packed public declaration missing: ${name}`,
      );
    }
    await fs.writeFile(
      path.join(consumer, "consumer-types.mts"),
      `
import transformer, { type VizeParcelOptions } from '@vizejs/parcel-transformer';
import styles from '@vizejs/parcel-transformer/style';
const standard: VizeParcelOptions = { compiler: { customElements: ['x-view'] }, transformAssetUrls: false };
const vapor: VizeParcelOptions = { compiler: { vapor: true, runtimeModuleName: 'vue' } };
// @ts-expect-error Source-map ownership belongs to Parcel.
const badMap: VizeParcelOptions = { compiler: { sourceMap: false } };
// @ts-expect-error This compiler has no custom-runtime Vapor capability.
const badVapor: VizeParcelOptions = { compiler: { vapor: true, runtimeModuleName: 'custom-runtime' } };
void [transformer, styles, standard, vapor, badMap, badVapor];
`,
    );
    const packageRequire = createRequire(path.join(repository, "npm/builder/parcel/package.json"));
    run(
      process.execPath,
      [
        packageRequire.resolve("typescript/bin/tsc"),
        "--ignoreConfig",
        "--noEmit",
        "--strict",
        "--skipLibCheck",
        "--target",
        "ES2022",
        "--module",
        "NodeNext",
        "--moduleResolution",
        "NodeNext",
        "consumer-types.mts",
      ],
      consumer,
    );
  }
  assert.equal(
    sha(await fs.readFile(path.join(consumer, "package-lock.json"))),
    lock,
    "provider lock must not be rewritten",
  );
  const require = createRequire(path.join(consumer, "package.json"));
  const loaded = require("@vizejs/native");
  const sourceRequire = createRequire(path.join(nativeDirectory, "package.json"));
  assert.equal(loaded.getNativeBuildToken(), sourceRequire("./index.js").getNativeBuildToken());
  assert.match(loaded.getNativeBuildToken(), /^[a-f0-9]{32}$/);
  const runtimeNative = path.dirname(require.resolve("@vizejs/native"));
  const physicalImage = (await fs.readdir(runtimeNative)).filter((name) => name.endsWith(".node"));
  assert.equal(physicalImage.length, 1);
  const imageHash = sha(await fs.readFile(path.join(runtimeNative, physicalImage[0])));
  assert.equal(imageHash, sha(await fs.readFile(path.join(nativeDirectory, physicalImage[0]))));
  let revalidateBuiltSource;
  if (!preflightProvider) {
    const build = JSON.parse(
      await fs.readFile(
        path.join(nativeDirectory, ".artifacts/native/formatter-history/build-receipt.json"),
        "utf8",
      ),
    );
    const { validateCiBuild } = await import(
      pathToFileURL(path.join(nativeDirectory, "scripts/formatter-history-ci-build.mjs"))
    );
    validateCiBuild(nativeDirectory, build);
    assert.equal(build.source.head, run("git", ["rev-parse", "HEAD"], repository).trim());
    assert.equal(build.generated.sha256, imageHash);
    await write("native-build-receipt.json", build);
    revalidateBuiltSource = () => validateCiBuild(nativeDirectory, build);
    const targetCustody = JSON.parse(
      await fs.readFile(
        path.join(
          nativeDirectory,
          ".artifacts/native/formatter-history/parcel-target-custody.json",
        ),
        "utf8",
      ),
    );
    assert.deepEqual(targetCustody, validateNativeTargetCustody(targetCustody.preflight, build));
    await write("native-target-custody.json", targetCustody);
  }
  await makeApp(consumer);
  const manifest = JSON.parse(await fs.readFile(path.join(consumer, "package.json"), "utf8"));
  manifest.alias = { "fixture-counter": "./counter.js" };
  await fs.writeFile(path.join(consumer, "package.json"), JSON.stringify(manifest, null, 2) + "\n");
  const { Parcel } = require("@parcel/core");
  const { chromium } = require("@playwright/test");
  const base = {
    entries: path.join(consumer, "index.html"),
    config: path.join(consumer, ".parcelrc"),
    shouldTrace: true,
    env: { VIZE_PARCEL_NATIVE_TRACE: path.join(proof, "native-trace.jsonl") },
    cacheDir: path.join(consumer, ".parcel-cache"),
    shouldAutoInstall: false,
    defaultTargetOptions: {
      distDir: path.join(consumer, "dist"),
      sourceMaps: true,
      shouldOptimize: true,
    },
  };
  const bundler = new Parcel({ ...base, mode: "production", shouldDisableCache: true });
  const production = await bundler.run();
  const captured = await captureBuild("production", production);
  assert.ok(captured.bundles.some((bundle) => bundle.type === "css"));
  assert.ok(captured.bundles.some((bundle) => bundle.type === "svg"));
  const nativeAssets = [...captured.assets.values()].filter((asset) =>
    asset.uniqueKey?.endsWith("-vize-component"),
  );
  assert.equal(nativeAssets.length, 1);
  assert.ok(await nativeAssets[0].getMap(), "native authored mappings must reach Parcel");
  assert.ok(
    (await nativeAssets[0].getMap()).toVLQ().sourcesContent.includes(source()),
    "virtual component maps must retain the exact authored SFC",
  );
  const moduleCode = await nativeAssets[0].getCode();
  assert.doesNotMatch(moduleCode, /ref<number>/);
  await write("native-module.json", {
    code: moduleCode,
    map: (await nativeAssets[0].getMap()).toVLQ(),
  });
  const browserOptions = {
    headless: true,
    ...(process.env.VIZE_PARCEL_BROWSER ? { executablePath: process.env.VIZE_PARCEL_BROWSER } : {}),
  };
  browser = await chromium.launch(browserOptions);
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  const errors = [];
  page.on("pageerror", (error) => {
    errors.push(error.message);
    browserEvents.push({ type: "pageerror", message: error.message });
  });
  page.on("console", (message) =>
    browserEvents.push({
      type: "console",
      level: message.type(),
      text: message.text(),
      location: message.location(),
    }),
  );
  page.on("response", (response) =>
    browserEvents.push({
      type: "response",
      url: response.url(),
      status: response.status(),
      headers: response.headers(),
    }),
  );
  page.on("framenavigated", (frame) => {
    if (frame === page.mainFrame()) browserEvents.push({ type: "navigation", url: frame.url() });
  });
  page.on("websocket", (socket) =>
    socket.on("framereceived", (frame) =>
      browserEvents.push({
        type: "websocket",
        url: socket.url(),
        payload:
          typeof frame.payload === "string"
            ? frame.payload
            : { base64: frame.payload.toString("base64") },
      }),
    ),
  );
  await page.context().tracing.start({ screenshots: true, snapshots: true });
  tracing = true;
  staticServer = await serve(path.join(consumer, "dist"));
  await page.goto(staticServer.url);
  await page.getByRole("button").waitFor();
  assert.match(await page.getByRole("button").textContent(), /Count: 1/);
  await page.getByRole("button").click();
  assert.match(await page.getByRole("button").textContent(), /Count: 2/);
  const styles = await page.locator(".card").evaluate((element) => ({
    color: getComputedStyle(element).color,
    border: getComputedStyle(element).borderColor,
    scope: [...element.attributes]
      .map((attribute) => attribute.name)
      .filter((name) => name.startsWith("data-v-")),
  }));
  assert.equal(styles.color, "rgb(255, 99, 71)");
  assert.equal(styles.border, "rgb(12, 34, 56)");
  assert.equal(styles.scope.length, 1);
  assert.equal(
    await page.locator(".external").evaluate((element) => getComputedStyle(element).color),
    "rgb(22, 33, 44)",
  );
  await page.locator("img").evaluate((image) => image.decode());
  const imageUrl = await page.locator("img").getAttribute("src");
  assert.notEqual(imageUrl, "./icon.svg");
  const emittedIcon = captured.bundles.find(
    (bundle) => bundle.type === "svg" && path.basename(bundle.filePath) === path.basename(imageUrl),
  );
  assert.ok(emittedIcon, "the template URL must refer to Parcel's emitted asset");
  const servedIcon = Buffer.from(
    await (await fetch(new URL(imageUrl, staticServer.url))).arrayBuffer(),
  );
  assert.equal(sha(servedIcon), sha(await fs.readFile(emittedIcon.filePath)));
  assert.deepEqual(
    await page.locator("img").evaluate((image) => [image.naturalWidth, image.naturalHeight]),
    [8, 8],
  );
  await page.screenshot({ path: path.join(proof, "production.png"), fullPage: true });
  await write("production-browser.json", {
    text: await page.locator("body").innerText(),
    styles,
    imageUrl,
    errors,
  });
  assert.deepEqual(errors, []);
  await new Promise((resolve) => staticServer.server.close(resolve));
  staticServer = undefined;

  // Independent projects may share installed providers, never their source
  // identity or module evaluation state. Identical components retain the same
  // generated digest across project roots, while same-basename files coexist.
  const mirror = path.join(proof, "independent project 😺");
  await makeApp(mirror);
  await fs.copyFile(path.join(consumer, "package.json"), path.join(mirror, "package.json"));
  await fs.copyFile(
    path.join(consumer, "package-lock.json"),
    path.join(mirror, "package-lock.json"),
  );
  await fs.symlink(path.join(consumer, "node_modules"), path.join(mirror, "node_modules"), "dir");
  await fs.mkdir(path.join(mirror, "components"));
  for (const name of ["App.vue", "icon.svg", "styles"])
    await fs.cp(path.join(mirror, name), path.join(mirror, "components", name), {
      recursive: true,
    });
  await fs.writeFile(
    path.join(mirror, "index.html"),
    '<!doctype html><html><head><meta charset="utf-8"></head><body><div id="first"></div><div id="second"></div><div id="third"></div><script type="module" src="./main.js"></script></body></html>',
  );
  await fs.writeFile(
    path.join(mirror, "main.js"),
    'import {createApp} from "vue"; import Root from "./App.vue"; import Nested from "./components/App.vue"; createApp(Root).mount("#first"); createApp(Root).mount("#second"); createApp(Nested).mount("#third");',
  );
  await write("cross-project-source.json", {
    root: mirror,
    sfc: await fs.readFile(path.join(mirror, "App.vue"), "utf8"),
    nestedSfc: await fs.readFile(path.join(mirror, "components/App.vue"), "utf8"),
    main: await fs.readFile(path.join(mirror, "main.js"), "utf8"),
    html: await fs.readFile(path.join(mirror, "index.html"), "utf8"),
    providerLock: sha(await fs.readFile(path.join(mirror, "package-lock.json"))),
  });
  const mirrorBuild = await new Parcel({
    ...base,
    entries: path.join(mirror, "index.html"),
    config: path.join(mirror, ".parcelrc"),
    cacheDir: path.join(mirror, ".parcel-cache"),
    mode: "production",
    shouldDisableCache: true,
    defaultTargetOptions: { ...base.defaultTargetOptions, distDir: path.join(mirror, "dist") },
  }).run();
  const mirrorCapture = await captureBuild("cross-project", mirrorBuild);
  const mirrorModules = [...mirrorCapture.assets.values()].filter((asset) =>
    asset.uniqueKey?.endsWith("-vize-component"),
  );
  assert.equal(mirrorModules.length, 2);
  const mirrorRoot = mirrorModules.find((asset) => asset.filePath === path.join(mirror, "App.vue"));
  assert.ok(mirrorRoot);
  assert.equal(
    sha(await mirrorRoot.getBuffer()),
    sha(await nativeAssets[0].getBuffer()),
    "generated component identity must not include an absolute project root",
  );
  const mirrorServer = await serve(path.join(mirror, "dist"));
  const mirrorPage = await page.context().newPage();
  const mirrorErrors = [];
  mirrorPage.on("pageerror", (error) => mirrorErrors.push(error.message));
  try {
    await mirrorPage.goto(mirrorServer.url);
    await mirrorPage.getByRole("button").first().waitFor();
    await mirrorPage.locator("#first button").click();
    const observations = await mirrorPage.evaluate(() => ({
      texts: [...document.querySelectorAll("button")].map((button) => button.textContent),
      colors: [...document.querySelectorAll("button")].map(
        (button) => getComputedStyle(button).color,
      ),
      scopes: [...document.querySelectorAll("button")].map((button) =>
        button.getAttributeNames().filter((name) => name.startsWith("data-v-")),
      ),
      evaluations: window.parcelModuleEvaluations,
    }));
    await write("cross-project-browser.json", { ...observations, errors: mirrorErrors });
    assert.deepEqual(observations.texts, ["Count: 2", "Count: 1", "Count: 1"]);
    assert.deepEqual(observations.colors, Array(3).fill("rgb(255, 99, 71)"));
    assert.equal(observations.evaluations, 2, "one module evaluation per physical SFC");
    assert.deepEqual(observations.scopes[0], observations.scopes[1]);
    assert.notDeepEqual(observations.scopes[0], observations.scopes[2]);
    assert.deepEqual(mirrorErrors, []);
    assert.equal(await page.evaluate(() => window.parcelModuleEvaluations), 1);
    assert.match(await page.getByRole("button").textContent(), /Count: 2/);
    await mirrorPage.screenshot({ path: path.join(proof, "cross-project.png"), fullPage: true });
  } finally {
    await write("cross-project-errors.json", mirrorErrors);
    await mirrorPage.close();
    await new Promise((resolve) => mirrorServer.server.close(resolve));
  }

  // A real Node target compiles the same SFC through the native SSR backend.
  const ssr = await new Parcel({
    entries: path.join(consumer, "entry-server.js"),
    shouldTrace: true,
    env: base.env,
    cacheDir: path.join(consumer, ".parcel-cache-ssr"),
    config: base.config,
    mode: "production",
    shouldAutoInstall: false,
    shouldDisableCache: true,
    targets: {
      server: {
        context: "node",
        outputFormat: "commonjs",
        includeNodeModules: ["fixture-counter"],
        isLibrary: true,
        distDir: path.join(consumer, "ssr"),
        sourceMap: true,
        optimize: false,
      },
    },
  }).run();
  const ssrCapture = await captureBuild("ssr", ssr);
  const serverBundle = ssrCapture.bundles.find((bundle) => bundle.type === "js");
  assert.ok(serverBundle);
  // Package type:module does not change the explicit CommonJS target bytes.
  const serverEntry = path.join(consumer, "ssr", "entry.cjs");
  await fs.copyFile(serverBundle.filePath, serverEntry);
  const render = require(serverEntry).default;
  const html = await render();
  assert.match(html, /Count: 1/);
  assert.match(html, /data-v-[a-f0-9]+/);
  assert.ok(!html.includes("<style"));
  await write("ssr.json", { html });

  const events = watchEvents;
  const development = [];
  const port = await freePort();
  const hmrPort = await freePort();
  const dev = new Parcel({
    ...base,
    mode: "development",
    defaultTargetOptions: {
      ...base.defaultTargetOptions,
      shouldOptimize: false,
      distDir: path.join(consumer, "development"),
    },
    serveOptions: { host: "127.0.0.1", port },
    hmrOptions: { host: "127.0.0.1", port: hmrPort },
  });
  watcher = await dev.watch((error, event) =>
    events.push(
      error ? { type: "error", diagnostics: error.diagnostics, message: error.message } : event,
    ),
  );
  const waitBuild = async (offset, type = "buildSuccess") => {
    const event = await eventually(
      () =>
        events
          .slice(offset)
          .find((event) => ["buildSuccess", "buildFailure", "error"].includes(event.type)),
      Boolean,
      `waiting for ${type}`,
    );
    development.push({
      type: event.type,
      buildTime: event.buildTime,
      diagnostics: event.diagnostics,
    });
    if (event.type === "buildSuccess")
      await captureBuild(`development-${development.length}`, event);
    else await write(`development-${development.length}-failure.json`, event.diagnostics);
    assert.equal(event.type, type);
    return event;
  };
  await waitBuild(0);
  await page.goto(`http://127.0.0.1:${port}`);
  await eventually(
    () => page.getByRole("button").textContent(),
    (value) => /Count: 1/.test(value),
    "development render",
  );
  await page.evaluate(() => {
    window.parcelNavigationMarker = "same-document";
  });
  await page.getByRole("button").click();
  await eventually(
    () => page.getByRole("button").textContent(),
    (value) => /Count: 2/.test(value),
    "development interaction",
  );
  assert.equal(await page.evaluate(() => window.parcelModuleEvaluations), 1);
  let offset = events.length;
  await fs.writeFile(
    path.join(consumer, "styles", "external é.css"),
    ".external { color: rgb(55, 66, 77); background-image: url('../icon.svg'); }\n",
  );
  await waitBuild(offset);
  await eventually(
    () => page.locator(".external").evaluate((element) => getComputedStyle(element).color),
    (value) => value === "rgb(55, 66, 77)",
    "external CSS HMR",
  );
  assert.match(
    await page.getByRole("button").textContent(),
    /Count: 2/,
    "CSS HMR retains component state",
  );
  assert.equal(
    await page.evaluate(() => window.parcelModuleEvaluations),
    1,
    "external CSS must not execute the component module again",
  );
  const nativeEventsBeforeSass = (await fs.readFile(path.join(proof, "native-trace.jsonl"), "utf8"))
    .trim()
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line));
  offset = events.length;
  await fs.writeFile(path.join(consumer, "styles", "palette.scss"), "$border: rgb(65, 43, 21);\n");
  await waitBuild(offset);
  await eventually(
    () => page.locator(".card").evaluate((element) => getComputedStyle(element).borderColor),
    (value) => value === "rgb(65, 43, 21)",
    "Sass dependency HMR",
  );
  assert.match(await page.getByRole("button").textContent(), /Count: 2/);
  assert.equal(
    await page.evaluate(() => window.parcelModuleEvaluations),
    1,
    "Sass dependency invalidation must not execute the component module again",
  );
  assert.equal(await page.evaluate(() => window.parcelNavigationMarker), "same-document");
  const nativeEventsAfterSass = (await fs.readFile(path.join(proof, "native-trace.jsonl"), "utf8"))
    .trim()
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line));
  const compileCalls = (events) =>
    new Set(
      events.filter((event) => event.name === "napi:compileSfc").map((event) => event.args.callId),
    ).size;
  assert.ok(compileCalls(nativeEventsBeforeSass) > 0, "actual compiler calls must be captured");
  assert.equal(
    compileCalls(nativeEventsAfterSass),
    compileCalls(nativeEventsBeforeSass),
    "Sass dependency changes must not invoke compileSfc",
  );
  await write("sass-native-operations.json", {
    before: nativeEventsBeforeSass,
    after: nativeEventsAfterSass,
    compileCallsBefore: compileCalls(nativeEventsBeforeSass),
    compileCallsAfter: compileCalls(nativeEventsAfterSass),
  });
  offset = events.length;
  await fs.writeFile(path.join(consumer, "counter.js"), "export const initialCount = 3;\n");
  await waitBuild(offset);
  await eventually(
    () => page.getByRole("button").textContent(),
    (value) => /Count: 3/.test(value),
    "script dependency HMR",
  );
  assert.equal(await page.evaluate(() => window.parcelModuleEvaluations), 2);
  offset = events.length;
  await fs.writeFile(path.join(consumer, "App.vue"), source("Updated"));
  await waitBuild(offset);
  await eventually(
    () => page.getByRole("button").textContent(),
    (value) => /Updated: 3/.test(value),
    "component HMR",
  );
  assert.equal(await page.evaluate(() => window.parcelModuleEvaluations), 3);
  await page.getByRole("button").click();
  offset = events.length;
  const inlineSource = source("Updated").replace(
    "background-image:",
    "border-radius: 3px; background-image:",
  );
  await fs.writeFile(path.join(consumer, "App.vue"), inlineSource);
  const inlineUpdate = await waitBuild(offset);
  const inlineModules = new Map();
  for (const bundle of inlineUpdate.bundleGraph.getBundles()) {
    bundle.traverseAssets((asset) => {
      if (asset.uniqueKey?.endsWith("-vize-component")) inlineModules.set(asset.id, asset);
    });
  }
  assert.equal(inlineModules.size, 1);
  const inlineModule = [...inlineModules.values()][0];
  assert.ok(
    (await inlineModule.getMap()).toVLQ().sourcesContent.includes(inlineSource),
    "inline style edits must refresh the full authored source map",
  );
  await eventually(
    () => page.locator(".card").evaluate((element) => getComputedStyle(element).borderRadius),
    (value) => value === "3px",
    "inline SFC style edit",
  );
  await eventually(
    () => page.getByRole("button").textContent(),
    (value) => /Updated: 3/.test(value),
    "SFC edits conservatively reload the component",
  );
  assert.equal(await page.evaluate(() => window.parcelModuleEvaluations), 4);
  assert.equal(
    await page.evaluate(() => window.parcelNavigationMarker),
    "same-document",
    "component updates must not reload the document",
  );
  offset = events.length;
  await fs.writeFile(
    path.join(consumer, ".vizerc.parcel.json"),
    JSON.stringify({ transformAssetUrls: false }),
  );
  await waitBuild(offset);
  await eventually(
    () => page.locator("img").getAttribute("src"),
    (value) => value === "./icon.svg",
    "tracked configuration invalidation",
  );
  if (recoveryFrontier) {
    offset = events.length;
    await fs.writeFile(path.join(consumer, "App.vue"), "<template><div></template>");
    await waitBuild(offset, "buildFailure");
    offset = events.length;
    await fs.writeFile(path.join(consumer, ".vizerc.parcel.json"), "{}\n");
    await fs.writeFile(path.join(consumer, "App.vue"), source("Recovered"));
    await waitBuild(offset);
    await eventually(
      () => page.getByRole("button").textContent(),
      (value) => /Recovered: 3/.test(value),
      "watch recovery",
    );
    offset = events.length;
    await fs.writeFile(path.join(consumer, "App.vue"), "<template><b></template>");
    await waitBuild(offset, "buildFailure");
    offset = events.length;
    const repairInputs = [
      source("Discarded"),
      "<template><i></template>",
      source("RapidRecovered"),
    ];
    for (const input of repairInputs) await fs.writeFile(path.join(consumer, "App.vue"), input);
    await eventually(
      () => page.locator("button.card").textContent(),
      (value) => /RapidRecovered: 3/.test(value),
      "rapid repair must settle on the latest authored component",
    );
    const settled = events.slice(offset).findLast((event) => event.type === "buildSuccess");
    assert.ok(settled);
    await captureBuild("rapid-repair-final", settled);
    await write("rapid-repair.json", {
      inputs: repairInputs,
      text: await page.locator("body").innerText(),
      events: events
        .slice(offset)
        .map(({ type, diagnostics, buildTime }) => ({ type, diagnostics, buildTime })),
      moduleEvaluations: await page.evaluate(() => window.parcelModuleEvaluations),
      navigationMarker: await page.evaluate(() => window.parcelNavigationMarker ?? null),
    });
    assert.equal(await fs.readFile(path.join(consumer, "App.vue"), "utf8"), repairInputs.at(-1));
  }
  await page.screenshot({ path: path.join(proof, "development.png"), fullPage: true });
  await write("development.json", {
    builds: development,
    text: await page.locator("body").innerText(),
    errors,
  });
  assert.deepEqual(errors, []);
  await watcher.unsubscribe();
  watcher = undefined;

  // The opt-in target is executed against its actual runtime, not inferred from code shape.
  manifest.alias.vue = "vue-vapor";
  await fs.writeFile(path.join(consumer, "package.json"), JSON.stringify(manifest, null, 2) + "\n");
  await fs.writeFile(
    path.join(consumer, ".vizerc.parcel.json"),
    JSON.stringify({ compiler: { vapor: true } }),
  );
  await fs.writeFile(
    path.join(consumer, "App.vue"),
    '<script setup lang="ts">import {ref} from "vue"; const n=ref<number>(1)</script><template><button class="vapor" @click="n++">Vapor: {{ n }}</button></template><style scoped>.vapor {color: rgb(14, 28, 42)}</style>',
  );
  await fs.writeFile(
    path.join(consumer, "main.js"),
    'import {createVaporApp} from "vue"; import App from "./App.vue"; createVaporApp(App).mount("#app");',
  );
  const vapor = await new Parcel({
    ...base,
    mode: "production",
    shouldDisableCache: true,
    defaultTargetOptions: { ...base.defaultTargetOptions, distDir: path.join(consumer, "vapor") },
  }).run();
  await captureBuild("vapor", vapor);
  staticServer = await serve(path.join(consumer, "vapor"));
  await page.goto(staticServer.url);
  await eventually(
    () => page.getByRole("button").textContent(),
    (value) => /Vapor: 1/.test(value),
    "Vapor render",
  );
  await page.getByRole("button").click();
  await eventually(
    () => page.getByRole("button").textContent(),
    (value) => /Vapor: 2/.test(value),
    "Vapor interaction",
  );
  assert.equal(
    await page.locator(".vapor").evaluate((element) => getComputedStyle(element).color),
    "rgb(14, 28, 42)",
  );
  await page.screenshot({ path: path.join(proof, "vapor.png"), fullPage: true });
  await write("vapor-browser.json", { text: await page.locator("body").innerText(), errors });
  assert.deepEqual(errors, []);
  await new Promise((resolve) => staticServer.server.close(resolve));
  staticServer = undefined;
  delete manifest.alias.vue;
  await fs.writeFile(path.join(consumer, "package.json"), JSON.stringify(manifest, null, 2) + "\n");
  await fs.writeFile(
    path.join(consumer, "main.js"),
    'import {createApp} from "vue"; import App from "./App.vue"; createApp(App).mount("#app");',
  );

  const expectedFailures = [
    ["syntax", "<template><div></template>", {}],
    ["module", "<template><p>Hello</p></template><style module>.foo{color:red}</style>", {}],
    [
      "external-script",
      '<script src="./missing.js"></script><template><p>Hello</p></template>',
      {},
    ],
    ["config", source(), { compiler: { filename: "other.vue" } }],
    [
      "vapor-custom-runtime",
      source(),
      { compiler: { vapor: true, runtimeModuleName: "custom-runtime" } },
    ],
  ];
  for (const [name, text, config] of expectedFailures) {
    if (typeof name !== "string" || typeof text !== "string")
      throw new TypeError("Invalid failure fixture");
    await fs.writeFile(path.join(consumer, "App.vue"), text);
    await fs.writeFile(path.join(consumer, ".vizerc.parcel.json"), JSON.stringify(config));
    let failure;
    try {
      await new Parcel({ ...base, mode: "production", shouldDisableCache: true }).run();
    } catch (error) {
      failure = error.diagnostics;
    }
    assert.ok(failure?.length, `${name} must fail closed`);
    await write(`failure-${name}.json`, failure);
  }
  await fs.writeFile(path.join(consumer, "App.vue"), source());
  await fs.writeFile(path.join(consumer, ".vizerc.parcel.json"), "{}\n");

  await page.context().tracing.stop({ path: path.join(proof, "browser-trace.zip") });
  tracing = false;
  await browser.close();
  browser = undefined;
  const rawNativeEvents = (await fs.readFile(path.join(proof, "native-trace.jsonl"), "utf8"))
    .trim()
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line));
  const nativeOperations = summarizeNativeEvents(rawNativeEvents);
  assert.ok(nativeOperations.calls > 0);
  const sourceHead = run("git", ["rev-parse", "HEAD"], repository).trim();
  const sourceStatus = run(
    "git",
    ["status", "--porcelain", "--untracked-files=normal"],
    repository,
  );
  // NAPI legitimately regenerates its tracked JS/declaration entrypoints.
  // The original producer validator authenticates their complete before/after
  // bytes and diff, while refusing every authored source change. Keep the raw
  // status rather than pretending generated outputs are an empty working tree.
  revalidateBuiltSource?.();
  const qualificationScope = recoveryFrontier
    ? "recovery-frontier"
    : "production-ssr-and-normal-development-updates";
  const unqualified = recoveryFrontier ? [] : ["post-error browser recovery (#10736)"];
  await write("native-operations.json", {
    schema: "vize.parcel.native-operations.v1",
    source: sourceHead,
    sourceQualification: !preflightProvider,
    qualificationScope,
    unqualified,
    ...nativeOperations,
  });
  await write("receipt.json", {
    complete: true,
    schema: "vize.parcel.host.v2",
    qualificationScope,
    unqualified,
    sourceQualification: !preflightProvider,
    source: sourceHead,
    sourceStatus,
    node: { version: process.version, platform: process.platform, arch: process.arch },
    native,
    plugin,
    physicalImage: physicalImage[0],
    imageHash,
    nativeBuildToken: loaded.getNativeBuildToken(),
    providerLock: lock,
    developmentHostOptions: {
      shouldContentHash: false,
      browserCache: "Chromium default; cache disabling APIs are not used",
      recoveryFrontier,
      unqualifiedRecovery: "Default error recovery remains a failing frontier: #10736",
    },
    productionBuildTimeMs: production.buildTime,
    ssrBuildTimeMs: ssr.buildTime,
    vaporBuildTimeMs: vapor.buildTime,
    hostElapsedMs: performance.now() - started,
    assertions: [
      "packed-native-image",
      ...(!preflightProvider ? ["packed-public-types"] : []),
      "real-parcel-production",
      "scoped-css",
      "production-css-vars",
      "sass-host-pipeline",
      "external-style-owner",
      "template-and-css-svg",
      "typescript-script",
      "unicode-crlf-maps",
      "alias",
      "browser-interaction",
      "duplicate-components-and-instances",
      "cross-project-isolation",
      "project-relative-generated-identity",
      "node-ssr",
      "external-css-hmr",
      "sass-dependency-hmr",
      "css-dependencies-do-not-evaluate-script",
      "script-updates-evaluate-exactly-once",
      "actual-sass-compiler-call-count",
      "inline-sfc-style-update-and-faithful-map",
      "script-dependency-hmr",
      "component-hmr",
      "configuration-invalidation",
      ...(recoveryFrontier
        ? ["watch-error-recovery", "rapid-repair-settles-on-latest-source"]
        : []),
      "vapor-browser-interaction",
      "invalid-sfc-rejected",
      "unsupported-css-module-rejected",
      "external-script-rejected",
      "unknown-config-rejected",
      "unsupported-custom-vapor-runtime-rejected",
    ],
  });
  success = true;
} finally {
  await write(
    "watch-events.json",
    watchEvents.map(({ type, diagnostics, message, buildTime }) => ({
      type,
      diagnostics,
      message,
      buildTime,
    })),
  );
  if (watcher) await watcher.unsubscribe();
  if (staticServer) await new Promise((resolve) => staticServer.server.close(resolve));
  if (browser && tracing) {
    try {
      await browser
        .contexts()[0]
        ?.tracing.stop({ path: path.join(proof, "browser-trace-failure.zip") });
    } catch (error) {
      browserEvents.push({ type: "trace-error", message: error.message });
    }
  }
  if (browser) await browser.close();
  await write("browser-events.json", browserEvents);
  await write("processes.json", processes);
  const entries = [];
  for (const name of await files(proof)) {
    if (
      name.includes(`${path.sep}node_modules${path.sep}`) ||
      name.endsWith(`${path.sep}manifest.json`)
    )
      continue;
    const bytes = await fs.readFile(name);
    entries.push({ path: path.relative(proof, name), bytes: bytes.length, sha256: sha(bytes) });
  }
  await write("manifest.json", { complete: success, files: entries });
}
