import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import { verifyHydration, verifyInitial, verifyStock } from "./contracts.mjs";

export async function waitForTcp(port, child) {
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
export async function observe(page) {
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
async function stop(child) {
  if (child.exitCode !== null || child.signalCode !== null) return;
  child.kill("SIGTERM");
  let timer;
  await Promise.race([
    new Promise((resolve) => child.once("exit", resolve)),
    new Promise((resolve) => {
      timer = setTimeout(() => {
        child.kill("SIGKILL");
        resolve();
      }, 10_000);
    }),
  ]);
  clearTimeout(timer);
  if (child.exitCode === null && child.signalCode === null)
    await new Promise((resolve) => child.once("exit", resolve));
}
export async function replayBrowser(browser, projects, output, assertPackages) {
  const observations = {};
  const write = (variant, value) =>
    fs.writeFileSync(
      path.join(output, `${variant}-observation.json`),
      JSON.stringify(value, null, 2) + "\n",
    );
  for (const [index, variant] of ["stock", "baseline", "published"].entries()) {
    assertPackages();
    const workspace = projects[variant],
      port = 18931 + index;
    const stream = fs.openSync(path.join(output, `${variant}-dev.log`), "wx");
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
    const packet = {
      variant,
      initial: null,
      initialHtml: null,
      css: [],
      hydrated: null,
      passed: false,
    };
    observations[variant] = packet;
    child.on("error", (error) => {
      packet.processError = { name: error.name, message: error.message };
      write(variant, packet);
    });
    try {
      await waitForTcp(port, child); // no HTTP request can warm the first SSR response
      const page = await context.newPage(),
        responses = [];
      page.on("response", (response) => {
        if (response.request().resourceType() === "stylesheet") responses.push(response);
      });
      const response = await page.goto(`http://127.0.0.1:${port}/`, {
        waitUntil: "load",
        timeout: 120_000,
      });
      packet.response = { status: response.status(), headers: response.headers() };
      packet.initialHtml = await response.text();
      fs.writeFileSync(path.join(output, `${variant}-initial.html`), packet.initialHtml);
      packet.initial = await observe(page);
      packet.css = await Promise.all(
        responses.map(async (entry) => ({
          url: entry.url(),
          status: entry.status(),
          contentType: entry.headers()["content-type"],
          body: await entry.text(),
        })),
      );
      write(variant, packet);
      assert.equal(response.status(), 200);
      verifyInitial(variant, packet.initial, packet.css);
      if (variant === "published") {
        const hydration = await browser.newContext({ viewport: { width: 900, height: 600 } });
        try {
          const page = await hydration.newPage();
          await page.goto(`http://127.0.0.1:${port}/`, { waitUntil: "load", timeout: 120_000 });
          await page.locator('.card[data-hydrated="true"]').waitFor({ timeout: 30_000 });
          packet.hydrated = await observe(page);
          write(variant, packet);
          verifyHydration(packet.initial, packet.hydrated);
        } finally {
          await hydration.close();
        }
      }
      assertPackages();
      packet.passed = true;
    } catch (error) {
      packet.failure = { name: error.name, message: error.message };
    } finally {
      try {
        await context.close();
      } finally {
        await stop(child);
        packet.processExit = { code: child.exitCode, signal: child.signalCode };
        fs.closeSync(stream);
        write(variant, packet);
      }
    }
  }
  assert(
    Object.values(observations).every((packet) => packet.passed),
    "published Nuxt replay failed; every observation is retained",
  );
  verifyStock(observations.published.initial, observations.stock.initial);
  return observations;
}
