import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import test from "node:test";
import { chromium } from "playwright";
import {
  instrumentObserver,
  observerEvidence,
  waitForNewNativeDelivery,
} from "./variant-navigation-browser.observer.ts";

void test(
  "raw observer instrumentation preserves actual Chromium delivery and callback",
  {
    skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1",
  },
  async () => {
    const browser = await chromium.launch();
    const page = await browser.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(String(error)));
    await instrumentObserver(page);
    const input = await readFile(
      new URL("./variant-navigation-browser.instrumentation.js", import.meta.url),
    );
    const output = new URL(
      "../../../../artifacts/musea-native-literal-variants/navigation-probe/",
      import.meta.url,
    );
    await mkdir(output, { recursive: true });
    try {
      await page.goto(
        "data:text/html,<body data-variant-name='transport' style='height:100px'>probe</body>",
      );
      await page.evaluate(() => {
        Reflect.set(window, "__deliveredProbeEntries", 0);
        const observer = new IntersectionObserver((entries, observer) => {
          const records = Reflect.get(window, "__museaNavigationProbe").records;
          const record = records.find(
            (record: { observer: IntersectionObserver }) => record.observer === observer,
          );
          Reflect.set(window, "__probeArrayIdentity", record.deliveries.includes(entries));
          Reflect.set(window, "__probeObserverIdentity", record.observer === observer);
          Reflect.set(
            window,
            "__deliveredProbeEntries",
            Reflect.get(window, "__deliveredProbeEntries") + entries.length,
          );
        });
        observer.observe(document.body);
      });
      await waitForNewNativeDelivery(page, 0);
      const evidence = await observerEvidence(page);
      assert.deepEqual(errors, []);
      assert.equal(evidence.records.length, 1);
      assert.equal(evidence.records[0].targets[0][0].name, "transport");
      assert.equal(evidence.records[0].targets[0][0].isIntersecting, true);
      assert.equal(await page.evaluate(() => Reflect.get(window, "__probeArrayIdentity")), true);
      assert.equal(await page.evaluate(() => Reflect.get(window, "__probeObserverIdentity")), true);
      assert.equal(
        await page.evaluate(() => Reflect.get(window, "__deliveredProbeEntries")),
        evidence.records[0].targets.reduce((count, entries) => count + entries.length, 0),
      );
    } finally {
      await writeFile(
        new URL("receipt.json", output),
        JSON.stringify(
          {
            errors,
            scope: "Chromium instrumentation transport only; no Vize native producer qualification",
            platform: process.platform,
            arch: process.arch,
            browserVersion: browser.version(),
            rawInstrumentationSha256: createHash("sha256").update(input).digest("hex"),
            evidence: await observerEvidence(page),
            html: await page.content(),
          },
          null,
          2,
        ),
      );
      await writeFile(new URL("instrumentation.js", output), input);
      assert.deepEqual(
        await readFile(new URL("./variant-navigation-browser.instrumentation.js", import.meta.url)),
        input,
      );
      await browser.close();
    }
  },
);
