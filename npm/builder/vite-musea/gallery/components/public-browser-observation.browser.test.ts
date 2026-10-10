import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { createServer } from "node:http";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { chromium } from "playwright";
import { observeBrowser } from "../../../../../tools/support/release/public_acceptance/browser_observation.ts";
import { frames } from "../../../../../tools/support/release/public_acceptance/gallery.ts";

void test(
  "an actual unchanged public readiness failure retains whole iframe and HTTP evidence without a success receipt",
  { skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1" },
  async () => {
    const directory = await mkdtemp(path.join(os.tmpdir(), "musea-public-failure-"));
    const output = path.join(directory, "musea.json");
    const observer = observeBrowser(output);
    const server = createServer((request, response) => {
      if (request.url === "/missing.js") {
        response.writeHead(404, { "Content-Type": "text/javascript" });
        response.end("// genuinely missing diagnostic module\n");
      } else if (request.url?.startsWith("/preview")) {
        response.writeHead(200, { "Content-Type": "text/html" });
        response.end(`<!doctype html><html lang="en" data-brand="default" data-scheme="light"><body><museacomponent>Default Button</museacomponent><script>
window.__publicSetupCalls=1;window.__publicDocumentToken=crypto.randomUUID();
window.__publicFirstGlobals={brand:'default',scheme:'light',locale:'en'};
throw new Error('actual-preview-diagnostic-error');
</script></body></html>`);
      } else {
        response.writeHead(200, { "Content-Type": "text/html" });
        response.end(
          `<!doctype html><html><body><script src="/missing.js"></script><script>console.error('actual-gallery-diagnostic-console');fetch('/missing.js').then(response=>response.text()).then(text=>window.__actualFailureBody=text);</script><section class="variant-card"><iframe src="/preview?variant=Default"></iframe></section><section class="variant-card"><iframe src="/preview?variant=Second"></iframe></section></body></html>`,
        );
      }
    });
    const browser = await chromium.launch();
    try {
      await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
      const address = server.address();
      assert.ok(address && typeof address !== "string");
      const context = await browser.newContext();
      observer.context(context);
      const page = await context.newPage();
      await page.goto(`http://127.0.0.1:${address.port}/`);
      let failure: unknown;
      await assert.rejects(
        frames(page, { brand: "default", scheme: "light", locale: "en" }).catch(
          (error: unknown) => {
            failure = error;
            throw error;
          },
        ),
        /Timeout 30000ms exceeded/u,
      );
      await observer.failure(browser, failure, { version: "0.439.0", phase: "defaults" });
      const artifacts = path.join(directory, "musea-browser-failure");
      const packet = JSON.parse(await readFile(path.join(artifacts, "failure.json"), "utf8"));
      assert.equal(packet.success, false);
      assert.equal(
        existsSync(output),
        false,
        "diagnostic evidence cannot satisfy the acceptance seal",
      );
      assert.match(packet.error, /Timeout 30000ms exceeded/u);
      const captured = packet.pages[0].frames;
      assert.equal(captured.length, 3);
      for (const frame of captured.slice(1)) {
        assert.deepEqual(frame.state.root, { brand: "default", scheme: "light", lang: "en" });
        assert.equal(frame.state.setupCalls, 1);
        assert.equal(frame.state.buttons, 0);
        assert.equal(frame.state.unresolvedComponents, 1);
        assert.match(frame.state.documentToken, /^[0-9a-f-]{36}$/u);
        const html = await readFile(path.join(artifacts, frame.htmlFile));
        assert.equal(createHash("sha256").update(html).digest("hex"), frame.htmlSha256);
        assert.match(html.toString(), /<museacomponent>Default Button<\/museacomponent>/u);
      }
      assert.ok(
        packet.events.some(
          (event: { type: string; message?: string }) =>
            event.type === "pageerror" &&
            event.message?.includes("actual-preview-diagnostic-error"),
        ),
      );
      assert.ok(
        packet.events.some(
          (event: { type: string; text?: string }) =>
            event.type === "console" && event.text === "actual-gallery-diagnostic-console",
        ),
      );
      const missingResponses = packet.events.filter(
        (event: { type: string; url?: string }) =>
          event.type === "response" && event.url?.endsWith("/missing.js"),
      );
      assert.equal(missingResponses.length, 2);
      assert.ok(missingResponses.every((event: { status: number }) => event.status === 404));
      const missing = missingResponses.find(
        (event: { resourceType: string }) => event.resourceType === "fetch",
      );
      assert.equal(missing.status, 404);
      assert.equal(
        await readFile(path.join(artifacts, missing.bodyFile), "utf8"),
        "// genuinely missing diagnostic module\n",
      );
      assert.equal(
        createHash("sha256")
          .update(await readFile(path.join(artifacts, missing.bodyFile)))
          .digest("hex"),
        missing.bodySha256,
      );
      assert.ok((await readFile(path.join(artifacts, packet.pages[0].screenshotFile))).length > 0);
    } finally {
      await browser.close();
      await new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      );
      await rm(directory, { recursive: true, force: true });
    }
  },
);
