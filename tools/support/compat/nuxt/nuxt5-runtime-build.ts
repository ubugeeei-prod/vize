// #11793: unchanged reported input, source-built packages, isolated Nitro output.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { at, readJson } from "../../json-value.ts";
import { freePort, loadChromium } from "./host-types.ts";
import { prepareNuxtSourceBinding } from "./source-binding.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixtureRelative = "tools/support/compat/nuxt/fixtures/nuxt5-runtime-build";
const fixture = path.join(root, fixtureRelative);
const output = path.resolve(process.argv[2] ?? "");
assert.ok(process.argv[2] && process.argv[3], "usage: nuxt5-runtime-build.ts OUTPUT RUNTIME_NODE");
const runtimeNode = fs.realpathSync(process.argv[3]);
assert.equal(fs.existsSync(output), false, "retain a fresh packet");
fs.mkdirSync(output, { recursive: true });
const sha256 = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
assert.equal(
  process.execPath,
  process.env.VIZE_NUXT_SOURCE_NODE,
  "use the captured source runtime",
);
assert.equal(sha256(process.execPath), process.env.VIZE_NUXT_SOURCE_NODE_SHA256);
const runtimeProbe = spawnSync(
  runtimeNode,
  [
    "--input-type=module",
    "--eval",
    "console.log(JSON.stringify({version:process.version,execPath:process.execPath,platform:process.platform,arch:process.arch}))",
  ],
  { encoding: "utf8" },
);
assert.equal(runtimeProbe.error, undefined);
assert.equal(runtimeProbe.status, 0);
const runtime: unknown = JSON.parse(runtimeProbe.stdout);
assert.equal(at(runtime, "version"), "v26.13.0", "use the reported consumer Node version");
assert.equal(at(runtime, "execPath"), runtimeNode);
assert.equal(at(runtime, "platform"), process.platform);
assert.equal(at(runtime, "arch"), process.arch);
const runtimeSha256 = sha256(runtimeNode);
fs.writeFileSync(
  path.join(output, "runtime.json"),
  JSON.stringify(
    {
      producer: {
        version: process.version,
        execPath: process.execPath,
        sha256: sha256(process.execPath),
      },
      consumer: runtime,
      sha256: runtimeSha256,
    },
    null,
    2,
  ) + "\n",
);
const pins = readJson(path.join(fixture, "provenance.json"));
for (const file of ["package.json", "nuxt.config.ts", "app/app.vue", "package-lock.json"]) {
  assert.equal(sha256(path.join(fixture, file)), at(pins, "files", file));
  fs.mkdirSync(path.dirname(path.join(output, "inputs", file)), { recursive: true });
  fs.copyFileSync(path.join(fixture, file), path.join(output, "inputs", file));
}
const versions: Record<string, unknown> = {};
for (const [name, expected] of [
  ["nuxt", "5.0.0-2611031025-9496b7e"],
  ["nitro", "3.0.261029-rc"],
  ["vite", "8.3.5"],
  ["vue", "3.5.44"],
  ["@vizejs/nuxt", "0.676.0"],
  ["@vizejs/vite-plugin", "0.676.0"],
  ["@vizejs/native", "0.676.0"],
  ["vize", "0.676.0"],
] as const) {
  versions[name] = at(
    readJson(path.join(fixture, "node_modules", name, "package.json")),
    "version",
  );
  fs.writeFileSync(path.join(output, "versions.json"), JSON.stringify(versions, null, 2) + "\n");
  assert.equal(versions[name], expected, `pinned ${name}`);
}
const sourceBinding = prepareNuxtSourceBinding(root, output, { fixtureDirectory: fixtureRelative });
const swaps = [
  ["nuxt", "npm/framework/nuxt"],
  ["vite-plugin", "npm/builder/vite"],
] as const;
const dist: Record<string, Record<string, string>> = {};
function hashes(directory: string): Record<string, string> {
  return Object.fromEntries(
    fs
      .readdirSync(directory, { recursive: true, encoding: "utf8" })
      .sort()
      .flatMap((name) => {
        const file = path.join(directory, name);
        return fs.statSync(file).isFile() ? [[name, sha256(file)]] : [];
      }),
  );
}
let server: ReturnType<typeof spawn> | undefined;
let browser: Awaited<ReturnType<ReturnType<typeof loadChromium>["launch"]>> | undefined;
let lazyServer: ReturnType<typeof spawn> | undefined;
let lazyBrowser: Awaited<ReturnType<ReturnType<typeof loadChromium>["launch"]>> | undefined;
let lazyProject: string | undefined;
const swapped: string[] = [];
try {
  for (const [name, source] of swaps) {
    const installed = path.join(fixture, "node_modules/@vizejs", name, "dist");
    const candidate = path.join(root, source, "dist");
    dist[name] = hashes(candidate);
    fs.cpSync(installed, path.join(output, `published-${name}-dist`), { recursive: true });
    swapped.push(name);
    fs.rmSync(installed, { recursive: true });
    fs.cpSync(candidate, installed, { recursive: true });
    assert.deepEqual(hashes(installed), dist[name]);
  }
  fs.writeFileSync(path.join(output, "candidate-dist.json"), JSON.stringify(dist, null, 2) + "\n");
  for (const directory of [".nuxt", ".output"]) {
    fs.rmSync(path.join(fixture, directory), { recursive: true, force: true });
  }
  const log = fs.openSync(path.join(output, "build.log"), "w");
  let built;
  try {
    built = spawnSync(runtimeNode, ["node_modules/nuxt/bin/nuxt.mjs", "build"], {
      cwd: fixture,
      env: {
        ...sourceBinding.environment,
        PATH: `${path.dirname(runtimeNode)}${path.delimiter}${process.env.PATH}`,
        NO_COLOR: "1",
        NUXT_TELEMETRY_DISABLED: "1",
      },
      stdio: ["ignore", log, log],
      timeout: 180_000,
    });
  } finally {
    fs.closeSync(log);
  }
  fs.writeFileSync(
    path.join(output, "build-process.json"),
    JSON.stringify({
      command: [runtimeNode, "node_modules/nuxt/bin/nuxt.mjs", "build"],
      cwd: fixture,
      pid: built.pid,
      runtimeSha256,
      status: built.status,
      signal: built.signal,
      error: built.error?.message ?? null,
    }) + "\n",
  );
  assert.equal(built.error, undefined);
  assert.equal(built.status, 0, "retain the complete failing build log");
  const binding = sourceBinding.verify();
  const isolated = path.join(output, "isolated");
  fs.cpSync(path.join(fixture, ".output"), isolated, { recursive: true });
  for (let directory = isolated; ; directory = path.dirname(directory)) {
    assert.equal(
      fs.existsSync(path.join(directory, "node_modules/vue")),
      false,
      "output cannot borrow project Vue",
    );
    if (directory === path.dirname(directory)) break;
  }
  const client = path.join(isolated, "public/_nuxt");
  const chunks = fs
    .readdirSync(client)
    .filter((name) => name.endsWith(".js"))
    .map((name) => {
      const file = path.join(client, name);
      const code = fs.readFileSync(file, "utf8");
      const bareRuntime = [
        ...code.matchAll(
          /(?:\bfrom\s*|\bimport\s*(?:\(\s*)?)["']((?:vue(?:\/[^"']*)?|@vue\/[^"']+))["']/g,
        ),
      ].map((match) => ({ offset: match.index, specifier: match[1] }));
      return { name, sha256: sha256(file), bareRuntime };
    });
  fs.writeFileSync(path.join(output, "client-chunks.json"), JSON.stringify(chunks, null, 2) + "\n");
  assert.ok(chunks.length > 0);
  assert.deepEqual(
    chunks.flatMap((chunk) => chunk.bareRuntime),
    [],
    "browser closure has no bare Vue imports",
  );
  const port = await freePort();
  const serverLog = fs.openSync(path.join(output, "server.log"), "w");
  server = spawn(runtimeNode, [path.join(isolated, "server/index.mjs")], {
    cwd: isolated,
    env: {
      ...process.env,
      PATH: `${path.dirname(runtimeNode)}${path.delimiter}${process.env.PATH}`,
      HOST: "127.0.0.1",
      PORT: String(port),
    },
    stdio: ["ignore", serverLog, serverLog],
  });
  fs.writeFileSync(
    path.join(output, "server-process.json"),
    JSON.stringify(
      {
        command: [runtimeNode, path.join(isolated, "server/index.mjs")],
        cwd: isolated,
        pid: server.pid,
        runtimeSha256,
      },
      null,
      2,
    ) + "\n",
  );
  fs.closeSync(serverLog);
  const url = `http://127.0.0.1:${port}/`;
  let ready = false;
  for (let attempt = 0; attempt < 100; attempt++) {
    try {
      const response = await fetch(url, { signal: AbortSignal.timeout(1_000) });
      const html = await response.text();
      fs.writeFileSync(path.join(output, "ssr.html"), html);
      fs.writeFileSync(
        path.join(output, "ssr-response.json"),
        JSON.stringify({ status: response.status, headers: Object.fromEntries(response.headers) }) +
          "\n",
      );
      assert.equal(response.status, 200, "isolated SSR must render without project dependencies");
      assert.match(html, /<button\b[^>]*>0<\/button>/);
      ready = true;
      break;
    } catch (error) {
      if (fs.existsSync(path.join(output, "ssr-response.json"))) throw error;
      if (server.exitCode !== null) throw new Error("isolated SSR server exited", { cause: error });
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.ok(ready, "isolated SSR server starts");
  browser = await loadChromium(root).launch();
  const page = await browser.newPage();
  const errors: string[] = [];
  const recordError = (error: string) => {
    errors.push(error);
    fs.appendFileSync(path.join(output, "browser-errors.jsonl"), JSON.stringify(error) + "\n");
  };
  page.on("pageerror", (error) => recordError(String(error)));
  page.on("console", (message) => {
    if (["warning", "error"].includes(message.type())) recordError(message.text());
  });
  await page.goto(url, { waitUntil: "networkidle" });
  fs.writeFileSync(path.join(output, "browser-before.html"), await page.content());
  await page.getByRole("button", { name: "0" }).click();
  fs.writeFileSync(path.join(output, "browser-after-click.html"), await page.content());
  await page.getByRole("button", { name: "1" }).waitFor();
  fs.writeFileSync(path.join(output, "browser.html"), await page.content());
  fs.writeFileSync(
    path.join(output, "browser-errors.json"),
    JSON.stringify(errors, null, 2) + "\n",
  );
  assert.deepEqual(errors, []);
  const browserVersion = browser.version();

  await browser.close();
  browser = undefined;
  server.kill("SIGTERM");
  server = undefined;

  const lazyInputDirectory = path.join(fixture, "lazy-hydration");
  const lazyEvidence = path.join(output, "lazy-hydration");
  const lazyInputs = path.join(lazyEvidence, "inputs");
  fs.mkdirSync(lazyInputs, { recursive: true });
  const lazyPins = readJson(path.join(lazyInputDirectory, "provenance.json"));
  for (const file of ["app.vue.txt", "Hydrated.vue.txt"] as const) {
    const source = path.join(lazyInputDirectory, file);
    assert.equal(sha256(source), at(lazyPins, "files", file));
    fs.copyFileSync(source, path.join(lazyInputs, file));
  }
  fs.copyFileSync(
    path.join(lazyInputDirectory, "provenance.json"),
    path.join(lazyInputs, "provenance.json"),
  );

  const lazyProjectDirectory = fs.mkdtempSync(
    path.join(path.dirname(output), "nuxt5-lazy-hydration-"),
  );
  lazyProject = lazyProjectDirectory;
  for (const file of ["package.json", "package-lock.json", "nuxt.config.ts"] as const) {
    fs.copyFileSync(path.join(fixture, file), path.join(lazyProjectDirectory, file));
  }
  fs.mkdirSync(path.join(lazyProjectDirectory, "app", "components"), { recursive: true });
  fs.copyFileSync(
    path.join(lazyInputDirectory, "app.vue.txt"),
    path.join(lazyProjectDirectory, "app", "app.vue"),
  );
  fs.copyFileSync(
    path.join(lazyInputDirectory, "Hydrated.vue.txt"),
    path.join(lazyProjectDirectory, "app", "components", "Hydrated.vue"),
  );
  fs.symlinkSync(
    path.join(fixture, "node_modules"),
    path.join(lazyProjectDirectory, "node_modules"),
    process.platform === "win32" ? "junction" : "dir",
  );

  const lazyBuildLog = fs.openSync(path.join(lazyEvidence, "build.log"), "w");
  let lazyBuild;
  try {
    lazyBuild = spawnSync(runtimeNode, ["node_modules/nuxt/bin/nuxt.mjs", "build"], {
      cwd: lazyProjectDirectory,
      env: {
        ...sourceBinding.environment,
        PATH: `${path.dirname(runtimeNode)}${path.delimiter}${process.env.PATH}`,
        NO_COLOR: "1",
        NUXT_TELEMETRY_DISABLED: "1",
      },
      stdio: ["ignore", lazyBuildLog, lazyBuildLog],
      timeout: 180_000,
    });
  } finally {
    fs.closeSync(lazyBuildLog);
  }
  fs.writeFileSync(
    path.join(lazyEvidence, "build-process.json"),
    JSON.stringify({
      command: [runtimeNode, "node_modules/nuxt/bin/nuxt.mjs", "build"],
      cwd: lazyProjectDirectory,
      pid: lazyBuild.pid,
      runtimeSha256,
      status: lazyBuild.status,
      signal: lazyBuild.signal,
      error: lazyBuild.error?.message ?? null,
    }) + "\n",
  );
  assert.equal(lazyBuild.error, undefined);
  assert.equal(lazyBuild.status, 0, "Nuxt 5 lazy hydration fixture builds");

  const lazyIsolated = path.join(lazyEvidence, "isolated");
  fs.cpSync(path.join(lazyProjectDirectory, ".output"), lazyIsolated, { recursive: true });
  const lazyPort = await freePort();
  const lazyServerLog = fs.openSync(path.join(lazyEvidence, "server.log"), "w");
  lazyServer = spawn(runtimeNode, [path.join(lazyIsolated, "server/index.mjs")], {
    cwd: lazyIsolated,
    env: {
      ...process.env,
      PATH: `${path.dirname(runtimeNode)}${path.delimiter}${process.env.PATH}`,
      HOST: "127.0.0.1",
      PORT: String(lazyPort),
    },
    stdio: ["ignore", lazyServerLog, lazyServerLog],
  });
  fs.closeSync(lazyServerLog);
  const lazyUrl = `http://127.0.0.1:${lazyPort}/`;
  let lazyHtml = "";
  for (let attempt = 0; attempt < 100; attempt++) {
    try {
      const response = await fetch(lazyUrl, { signal: AbortSignal.timeout(1_000) });
      lazyHtml = await response.text();
      assert.equal(response.status, 200, "lazy hydration SSR must render");
      break;
    } catch (error) {
      if (lazyServer.exitCode !== null) {
        throw new Error("lazy hydration SSR server exited", { cause: error });
      }
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  fs.writeFileSync(path.join(lazyEvidence, "ssr.html"), lazyHtml);
  assert.equal(lazyHtml.match(/hydrated-marker/g)?.length, 3);
  assert.doesNotMatch(lazyHtml, /\bhydrate-(?:on-visible|after|never)\b/);

  lazyBrowser = await loadChromium(root).launch();
  const lazyPage = await lazyBrowser.newPage();
  const lazyErrors: string[] = [];
  lazyPage.on("pageerror", (error) => lazyErrors.push(String(error)));
  lazyPage.on("console", (message) => {
    if (["warning", "error"].includes(message.type())) lazyErrors.push(message.text());
  });
  await lazyPage.goto(lazyUrl, { waitUntil: "domcontentloaded" });
  const hydrationState = () =>
    lazyPage
      .locator("#lazy .hydrated")
      .evaluateAll((elements) => elements.map((element) => element.getAttribute("data-hydrated")));
  await lazyPage.waitForFunction(
    () =>
      [...document.querySelectorAll("#lazy .hydrated")][0]?.getAttribute("data-hydrated") ===
      "true",
  );
  const initialHydration = await hydrationState();
  assert.deepEqual(initialHydration, ["true", "false", "false"]);
  await lazyPage.waitForFunction(
    () =>
      [...document.querySelectorAll("#lazy .hydrated")][1]?.getAttribute("data-hydrated") ===
      "true",
    undefined,
    { timeout: 5_000 },
  );
  const settledHydration = await hydrationState();
  assert.deepEqual(settledHydration, ["true", "true", "false"]);
  assert.deepEqual(lazyErrors, []);
  fs.writeFileSync(
    path.join(lazyEvidence, "browser.json"),
    JSON.stringify({ initialHydration, settledHydration, errors: lazyErrors }, null, 2) + "\n",
  );

  assert.equal(sha256(runtimeNode), runtimeSha256, "consumer Node identity remains unchanged");
  fs.writeFileSync(
    path.join(output, "report.json"),
    JSON.stringify(
      {
        passed: true,
        source: binding,
        dist,
        versions,
        producerNode: process.version,
        consumerNode: { identity: runtime, sha256: runtimeSha256 },
        platform: process.platform,
        arch: process.arch,
        browser: browserVersion,
        counter: { before: 0, after: 1 },
        lazyHydration: {
          ssrAttributesLeaked: 0,
          initial: initialHydration,
          settled: settledHydration,
        },
        isolatedSsr: 200,
        bareRuntimeImports: 0,
      },
      null,
      2,
    ) + "\n",
  );
} catch (error) {
  fs.writeFileSync(
    path.join(output, "failure.txt"),
    error instanceof Error ? (error.stack ?? String(error)) : String(error),
  );
  throw error;
} finally {
  await browser?.close();
  await lazyBrowser?.close();
  server?.kill("SIGTERM");
  lazyServer?.kill("SIGTERM");
  if (lazyProject) fs.rmSync(lazyProject, { recursive: true, force: true });
  for (const name of swapped) {
    const installed = path.join(fixture, "node_modules/@vizejs", name, "dist");
    fs.rmSync(installed, { recursive: true });
    fs.cpSync(path.join(output, `published-${name}-dist`), installed, { recursive: true });
  }
}
