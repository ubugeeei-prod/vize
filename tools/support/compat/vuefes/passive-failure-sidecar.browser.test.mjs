import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import http from "node:http";
import { gzipSync } from "node:zlib";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { performance } from "node:perf_hooks";
import { collectFailureCssom } from "../../../../tests/_helpers/vuefes-failure-cssom.mjs";
import { createVuefesFailureSidecar } from "../../../../tests/_helpers/vuefes-failure-sidecar.mjs";
const { chromium } = createRequire(new URL("../../../../tests/package.json", import.meta.url))(
  "@playwright/test",
);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const intent = { schema: "controlled-sidecar-intent; no native build authority", runId: "control" };
const scratch = () => fs.mkdtempSync(path.join(os.tmpdir(), "vuefes-failure-control-"));
const metadata = (outputDir) => ({
  outputDir,
  callbackStartedAt: performance.now(),
  declaredTimeout: 900_000,
  retry: 0,
});

void test("actual browser retains full decoded transport, redirect/failure refusals and CSS hierarchy/ancestry", async () => {
  const output = scratch();
  const script = Buffer.from(
    `window.transportWitness = true; /*${"actual whole body 🧪".repeat(20_000)}*/`,
  );
  const css =
    '@import url("/import.css"); @layer author { @media (min-width: 1px) { @supports (display: grid) { h2 { font-size: 20px; line-height: 34px; } } } } h2[data-scope] { font-size: 20px; line-height: 34px; }';
  const html =
    '<html><head><link rel="stylesheet" href="/root.css"><link rel="stylesheet" href="/abort.css"><script src="/redirect.js"></script><script src="/missing.js"></script></head><body style="color: rgb(1, 2, 3)"><main><h1>one</h1><h2 data-scope="actual" style="letter-spacing: 1px">two</h2><h3>three</h3><h4>four</h4><h5>five</h5><h6>six</h6></main></body></html>';
  const server = http.createServer((request, response) => {
    if (request.url === "/abort.css") {
      request.socket.destroy();
      return;
    }
    if (request.url === "/redirect.js") {
      response.writeHead(302, { location: "/entry.js" });
      response.end();
      return;
    }
    if (request.url === "/entry.js") {
      response.setHeader("Content-Type", "text/javascript; charset=utf-8");
      response.setHeader("Content-Encoding", "gzip");
      response.setHeader("X-Whole-Witness", ["one", "two"]);
      response.end(gzipSync(script));
      return;
    }
    if (request.url === "/root.css" || request.url === "/import.css") {
      response.setHeader("Content-Type", "text/css");
      response.end(
        request.url === "/root.css" ? css : ":root h2 { font-size: 18px; line-height: 27px; }",
      );
      return;
    }
    if (request.url === "/missing.js") {
      response.writeHead(404, {
        "Content-Type": "text/javascript",
        "Content-Length": Buffer.byteLength("/* actual404 */"),
      });
      response.end("/* actual404 */");
      return;
    }
    response.setHeader("Content-Type", "text/html");
    response.end(html);
  });
  let browser;
  try {
    await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
    browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();
    const sidecar = createVuefesFailureSidecar(
      [{ name: "actual", page }],
      metadata(output),
      intent,
    );
    await page.goto(`http://127.0.0.1:${server.address().port}`, { waitUntil: "load" });
    await page.evaluate(() => {
      const sheet = new CSSStyleSheet();
      sheet.replaceSync("h2 { text-transform: uppercase; }");
      document.adoptedStyleSheets = [sheet];
      const disabled = document.createElement("style");
      disabled.textContent = "h2 { font-size: 99px; }";
      document.head.append(disabled);
      disabled.sheet.disabled = true;
    });
    const original = new Error("actual visual diff ratio control");
    original.matcherResult = { pass: false };
    await sidecar.failed(original);
    const directory = path.join(output, "retry-0");
    const transport = JSON.parse(
      fs.readFileSync(path.join(directory, "actual-transport.json"), "utf8"),
    );
    const body = transport.requests.find((entry) => entry.url.endsWith("/entry.js"));
    assert.equal(body.body.sha256, hash(script));
    assert.deepEqual(fs.readFileSync(path.join(directory, body.body.file)), script);
    assert.equal(body.response.headers["content-encoding"], "gzip");
    assert.match(body.response.bodyEncoding, /browser-decoded.*not compressed wire/);
    assert.deepEqual(
      body.response.headersArray
        .filter((entry) => entry.name.toLowerCase() === "x-whole-witness")
        .map((entry) => entry.value),
      ["one", "two"],
    );
    assert.equal(body.response.fromServiceWorker, false);
    assert.equal(body.response.diskOrMemoryCache.available, false);
    const redirect = transport.requests.find((entry) => entry.response?.status === 302);
    assert.match(redirect.missing, /actual response\.body\(\) refused.*redirect/s);
    assert.equal(body.redirectedFrom, redirect.id);
    assert.match(
      transport.requests.find((entry) => entry.url.endsWith("/abort.css")).missing,
      /request failed/,
    );
    const missing = transport.requests.find((entry) => entry.response?.status === 404);
    assert.equal(missing.response.status, 404);
    assert.equal(missing.complete, false);
    assert.equal(typeof missing.requestFailure, "string");
    assert.match(missing.missing, /request failed/);
    if (missing.body) {
      // Chromium aborts this404 script and actually returns an empty decoded body.
      assert.equal(
        Number(missing.response.headers["content-length"]),
        Buffer.byteLength("/* actual404 */"),
      );
      assert.equal(missing.body.bytes, 0);
      assert.equal(fs.readFileSync(path.join(directory, missing.body.file)).length, 0);
    } else assert.match(missing.response.captureFailure, /actual response\.body\(\) refused/);
    assert.equal(transport.complete, false);
    const dom = JSON.parse(fs.readFileSync(path.join(directory, "actual-cssom.json"), "utf8"));
    assert.equal(dom.packet.headings.length, 6);
    assert.equal(dom.packet.documentHTML.includes('data-scope="actual"'), true);
    assert.equal(
      dom.packet.sheets.some((sheet) => sheet.disabled),
      true,
    );
    assert.equal(dom.packet.adoptedSheets.length, 1);
    const rootSheet = dom.packet.sheets.find((sheet) => sheet.href?.endsWith("/root.css"));
    assert.equal(typeof rootSheet.rules[0].importedSheet, "number");
    assert.match(
      JSON.stringify(rootSheet.rules),
      /CSSLayerBlockRule.*CSSMediaRule.*CSSSupportsRule/,
    );
    const heading = dom.packet.elements[dom.packet.headings[1].element];
    assert.equal(heading.computed["font-size"], "20px");
    assert.equal(heading.computed.color, "rgb(1, 2, 3)");
    assert.equal(heading.computed["letter-spacing"], "1px");
    assert.equal(dom.packet.elements[heading.parent].tag, "MAIN");
    assert.match(dom.packet.boundary, /not atomic screenshot/);
    const final = JSON.parse(fs.readFileSync(path.join(directory, "final.json"), "utf8"));
    assert.equal(final.complete, false);
    assert.equal(
      final.missing.some((entry) => entry.request === redirect.id),
      true,
    );
    assert.equal(final.originalError, original.stack);
  } finally {
    await browser?.close();
    await new Promise((resolve) => server.close(resolve));
    fs.rmSync(output, { recursive: true, force: true });
  }
});

void test("full CSSOM records security omissions without inventing readable sheets", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    await page.setContent("<h2>unreadable sheet control</h2>");
    await page.evaluate(() => {
      const style = document.createElement("style");
      style.textContent = "h2 { color: red; }";
      document.head.append(style);
      Object.defineProperty(style.sheet, "cssRules", {
        get() {
          throw new DOMException("controlled security refusal", "SecurityError");
        },
      });
    });
    const packet = await page.evaluate(collectFailureCssom);
    assert.equal(packet.complete, false);
    assert.match(packet.omissions[0].reason, /SecurityError/);
    assert.equal(packet.sheets[0].complete, false);
    assert.equal(packet.headings.length, 1);
  } finally {
    await browser.close();
  }
});
