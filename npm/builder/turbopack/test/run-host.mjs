import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import { createServer } from "node:net";
import { inspect as inspectError } from "node:util";
import { chromium } from "playwright";
import { prepareConsumer, consumer, output, run } from "./prepare-consumer.mjs";
import { checkRspackConsumer } from "./run-rspack.mjs";
import { checkHostErrors } from "./run-errors.mjs";
import { normalHostShutdown } from "./host-process.mjs";
import { checkPackedModules } from "./run-modules.mjs";
import { checkAuthoredMaps } from "./run-source-maps.mjs";

const receipt = await prepareConsumer();
const require = createRequire(path.join(consumer, "package.json"));
const next = require.resolve("next/dist/bin/next");
const sourcePath = path.join(consumer, "components/组件 space/FixtureCounter.vue");
const original = await fs.readFile(sourcePath, "utf8");
const packets = [];
const measurements = [];
const browser = await chromium.launch({ headless: true });

async function port() {
  const socket = createServer();
  await new Promise((resolve) => socket.listen(0, "127.0.0.1", resolve));
  const value = socket.address().port;
  await new Promise((resolve) => socket.close(resolve));
  return value;
}

async function launch(mode, label) {
  const selected = await port();
  const start = performance.now();
  const process = spawn(
    globalThis.process.execPath,
    [next, mode, "--hostname", "127.0.0.1", "--port", String(selected)],
    {
      cwd: consumer,
      env: { ...globalThis.process.env, NEXT_TELEMETRY_DISABLED: "1" },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  const stdout = [];
  const stderr = [];
  process.stdout.on("data", (data) => stdout.push(data));
  process.stderr.on("data", (data) => stderr.push(data));
  const closed = new Promise((resolve, reject) => {
    process.once("error", reject);
    process.once("close", (status, signal) => resolve({ status, signal }));
  });
  async function saveExit(result) {
    await fs.writeFile(path.join(output, `${label}-server.stdout`), Buffer.concat(stdout));
    await fs.writeFile(path.join(output, `${label}-server.stderr`), Buffer.concat(stderr));
    await fs.writeFile(
      path.join(output, `${label}-server-exit.json`),
      JSON.stringify(result) + "\n",
    );
  }
  const url = `http://127.0.0.1:${selected}`;
  const deadline = Date.now() + 90_000;
  let response;
  while (Date.now() < deadline) {
    if (process.exitCode !== null) {
      await saveExit(await closed);
      throw new Error(`Next ${mode} exited: ${Buffer.concat(stderr).toString()}`);
    }
    try {
      response = await fetch(url);
      if (response.ok) break;
      await fs.writeFile(path.join(output, `${label}-not-ready.html`), await response.text());
    } catch {
      /* The actual process has not opened its listener yet. */
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  if (!response?.ok) {
    process.kill("SIGTERM");
    await saveExit(await closed);
  }
  assert.ok(response?.ok, `Next ${mode} did not become ready: ${Buffer.concat(stderr).toString()}`);
  const html = await response.text();
  await fs.writeFile(path.join(output, `${label}-ssr.html`), html);
  assert.match(html, /Counter 2 🐱/);
  assert.match(html, /id="child"/);
  measurements.push({ phase: `${label}-ready-and-first-SSR`, ms: performance.now() - start });
  return {
    url,
    async stop() {
      const shutdown = {
        statusBefore: process.exitCode,
        signalBefore: process.signalCode,
        requested: process.kill("SIGTERM"),
      };
      const result = await closed;
      await saveExit({ ...result, shutdown });
      assert.ok(
        normalHostShutdown(result, shutdown),
        `host process must close normally: ${JSON.stringify({ ...result, shutdown })}\n${Buffer.concat(stdout).toString("utf8")}\n${Buffer.concat(stderr).toString("utf8")}`,
      );
    },
  };
}

async function inspect(page, label) {
  const packet = await page.evaluate(() => {
    const style = (selector) => getComputedStyle(document.querySelector(selector));
    const image = document.querySelector("#pixel");
    return {
      html: document.documentElement.outerHTML,
      token: window.fixtureToken,
      text: document.querySelector("#counter").textContent,
      visibleText: document.querySelector("#counter").innerText.trim(),
      color: style("#counter").color,
      padding: [style("#counter").paddingLeft, style("#captured").paddingLeft],
      border: style("#counter").borderTopColor,
      named: style("#named").fontWeight,
      scss: style("#scss").marginLeft,
      external: {
        border: style("#external").borderLeftColor,
        image: style("#external").backgroundImage,
      },
      bound: style("#bound").color,
      outside: style("#outside").color,
      background: style("#counter").backgroundImage,
      image: { src: image.currentSrc, complete: image.complete, width: image.naturalWidth },
      scope: [...document.querySelector("#counter").attributes]
        .filter((attribute) => attribute.name.startsWith("data-v-"))
        .map((attribute) => attribute.name),
    };
  });
  packets.push({ label, ...packet });
  await fs.writeFile(path.join(output, `${label}.json`), JSON.stringify(packet, null, 2) + "\n");
  return packet;
}

async function exercise(mode, label = mode) {
  const server = await launch(mode, label);
  const page = await browser.newPage();
  const errors = [];
  const warnings = [];
  const requests = [];
  const failures = [];
  page.on("pageerror", (error) => errors.push(error.stack ?? error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
    if (message.type() === "warning") warnings.push(message.text());
  });
  page.on("response", (response) =>
    requests.push({ url: response.url(), status: response.status() }),
  );
  try {
    await page.goto(server.url);
    await page.waitForFunction(
      () => Boolean(window.fixtureToken) && document.querySelector("#pixel").complete,
    );
    const initial = await inspect(page, `${label}-initial`);
    assert.equal(initial.visibleText, "Counter 2 🐱");
    assert.equal(initial.color, "rgb(12, 34, 56)");
    assert.deepEqual(initial.padding, ["7px", "7px"]);
    assert.equal(initial.border, "rgb(1, 2, 3)");
    assert.equal(initial.named, "700");
    assert.equal(initial.scss, "9px");
    assert.equal(initial.external.border, "rgb(4, 5, 6)");
    assert.equal(initial.bound, "rgb(22, 33, 44)");
    assert.equal(initial.outside, "rgb(0, 0, 0)");
    assert.equal(initial.image.width, 1);
    assert.equal(initial.scope.length, 1);
    for (const asset of [
      initial.image.src,
      /url\("([^"]+)"\)/.exec(initial.background)?.[1],
      /url\("([^"]+)"\)/.exec(initial.external.image)?.[1],
    ]) {
      assert.ok(asset?.startsWith(server.url + "/_next/static/media/"));
      const response = await fetch(asset);
      assert.equal(response.status, 200);
      const data = Buffer.from(await response.arrayBuffer());
      await fs.writeFile(
        path.join(output, `${label}-${path.basename(new URL(asset).pathname)}`),
        data,
      );
    }
    await page.locator("#counter").click();
    await page.waitForFunction(
      () => document.querySelector("#counter").textContent.trim() === "Counter 3 🐱",
    );
    if (mode === "dev") {
      let current = original;
      async function edit(from, to, predicate, argument) {
        assert.ok(current.includes(from));
        current = current.replace(from, to);
        const start = performance.now();
        await fs.writeFile(sourcePath, current);
        await page.waitForFunction(predicate, argument);
        measurements.push({ phase: from.slice(0, 40), ms: performance.now() - start });
        assert.equal(
          await page.evaluate(() => window.fixtureToken),
          initial.token,
          "host island must not remount",
        );
      }
      await edit(
        "{{ label }} {{ count }}",
        "{{ label }} updated {{ count }}",
        () => document.querySelector("#counter").textContent.trim() === "Counter updated 3 🐱",
      );
      await inspect(page, "dev-template-update");
      await edit(
        "rgb(12, 34, 56)",
        "rgb(32,54,76)",
        () => getComputedStyle(document.querySelector("#counter")).color === "rgb(32, 54, 76)",
      );
      assert.equal((await page.locator("#counter").textContent()).trim(), "Counter updated 3 🐱");
      await inspect(page, "dev-scoped-update");
      await edit("padding: 7px", "padding: 11px", () =>
        ["#counter", "#captured"].every(
          (selector) => getComputedStyle(document.querySelector(selector)).paddingLeft === "11px",
        ),
      );
      assert.equal((await page.locator("#counter").textContent()).trim(), "Counter updated 3 🐱");
      await inspect(page, "dev-module-update");
      await edit(
        "font-weight: 700",
        "font-weight: 600",
        () => getComputedStyle(document.querySelector("#named")).fontWeight === "600",
      );
      await edit(
        "$margin: 9px",
        "$margin: 13px",
        () => getComputedStyle(document.querySelector("#scss")).marginLeft === "13px",
      );
      await inspect(page, "dev-preprocessor-update");
      const dependency = path.join(consumer, "lib/label.js");
      await fs.writeFile(dependency, 'export const label = "Changed";\n');
      await page.waitForFunction(
        () => document.querySelector("#counter").textContent.trim() === "Changed updated 2 🐱",
      );
      assert.equal(await page.evaluate(() => window.fixtureToken), initial.token);
      await inspect(page, "dev-dependency-update");
      await fs.writeFile(dependency, 'export const label = "Counter";\n');
    }
    assert.deepEqual(errors, [], "no browser errors");
    assert.deepEqual(warnings, [], "no hydration or compiler-feature warnings");
    await page.screenshot({ path: path.join(output, `${label}.png`), fullPage: true });
    await fs.writeFile(
      path.join(output, `${label}-network.json`),
      JSON.stringify(requests, null, 2) + "\n",
    );
  } catch (error) {
    failures.push(error);
  } finally {
    for (const cleanup of [
      async () =>
        fs.writeFile(
          path.join(output, `${label}-final.html`),
          await page.content().catch((error) => String(error)),
        ),
      () =>
        fs.writeFile(
          path.join(output, `${label}-browser-errors.json`),
          JSON.stringify({ errors, warnings }, null, 2) + "\n",
        ),
      () => page.close(),
      () => server.stop(),
    ]) {
      try {
        await cleanup();
      } catch (error) {
        failures.push(error);
      }
    }
  }
  if (failures.length > 0) {
    const failure =
      failures.length === 1 ? failures[0] : new AggregateError(failures, `${label} failed`);
    await fs.writeFile(
      path.join(output, `${label}-failure.stderr`),
      inspectError(failure, { depth: 8, colors: false }) + "\n",
    );
    throw failure;
  }
}

try {
  await checkPackedModules(sourcePath, original, measurements);
  await exercise("dev");
  await fs.writeFile(sourcePath, original);
  const errorServer = await launch("dev", "errors");
  try {
    await checkHostErrors(errorServer.url, sourcePath, original, measurements);
  } finally {
    await errorServer.stop();
  }
  for (const name of ["cold", "warm"]) {
    const result = await run(`build-${name}`, process.execPath, [next, "build"], consumer, {
      NEXT_TELEMETRY_DISABLED: "1",
    });
    measurements.push({ phase: `build-${name}`, ms: result.elapsedMs });
    await exercise("start", `start-${name}`);
  }
  const maps = await checkAuthoredMaps(sourcePath, original);
  await checkRspackConsumer(browser, measurements);
  await fs.writeFile(
    path.join(output, "qualification.json"),
    JSON.stringify(
      { ...receipt, complete: true, packets: packets.length, measurements, maps },
      null,
      2,
    ) + "\n",
  );
} finally {
  await fs.writeFile(
    path.join(output, "measurements.json"),
    JSON.stringify(measurements, null, 2) + "\n",
  );
  await browser.close();
}
