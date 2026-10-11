import assert from "node:assert/strict";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { parseArgs } from "./cli/index.ts";
import { startHostedVrtSession } from "./cli/serve.ts";
import { sha256 } from "./hosted-vrt-build.fixtures.ts";
import {
  buildLiteralServiceGallery,
  captureLiteralService,
  retainedReports,
  type LiteralTitle,
} from "./hosted-vrt-literal.fixtures.ts";
import {
  createSecureHost,
  launchPublicBrowser,
  setLoopbackPermission,
  trustSecureHost,
  createAddressSpaceObserver,
} from "./hosted-vrt-network.fixtures.ts";
import { previews, variants } from "./literal-variant-browser.fixture.ts";

const sectionIds = {
  Controls: ["variant-default", "variant-custom-theme"],
  Keys: ["variant-proto", "variant-constructor", "variant-hasownproperty"],
};

async function noPersistedBearer(directory: string, token: string) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) await noPersistedBearer(filename, token);
    else if (entry.isFile())
      assert.equal((await readFile(filename)).includes(token), false, filename);
  }
}

void test(
  "public HTTPS source-native hosted VRT preserves five literal variants and each Art's report bytes",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1", timeout: 180000 },
  async (t) => {
    const f = await buildLiteralServiceGallery();
    const owned: {
      host?: Awaited<ReturnType<typeof createSecureHost>>;
      restoreTrust?: () => void;
      session?: Awaited<ReturnType<typeof startHostedVrtSession>>;
      browser?: Awaited<ReturnType<typeof launchPublicBrowser>>;
      spaces?: Awaited<ReturnType<typeof createAddressSpaceObserver>>;
      detachPermission?: () => Promise<void>;
    } = {};
    t.after(async () => {
      const failures: unknown[] = [];
      for (const cleanup of [
        () => owned.detachPermission?.(),
        () => owned.spaces?.close(),
        () => owned.browser?.close(),
        () => owned.session?.close(),
        () => owned.restoreTrust?.(),
        () => owned.host?.close(),
        () => f.retain(),
        () => owned.session && noPersistedBearer(f.output, owned.session.token),
      ]) {
        try {
          await cleanup();
        } catch (error) {
          failures.push(error);
        }
      }
      if (failures.length)
        throw new AggregateError(failures, "Hosted literal fixture cleanup failed");
    });
    const host = (owned.host = await createSecureHost(f.directory));
    owned.restoreTrust = await trustSecureHost(host);
    const local = path.join(f.root, "local-vrt");
    const url = `${host.origin}/built/gallery/`;
    const options = parseArgs(["serve", "--gallery-url", url, "--output", local]);
    options.vrt = {
      viewports: [{ name: "authored", width: 320, height: 180 }],
      threshold: 0,
      capture: { settleTime: 0 },
    };
    const session = (owned.session = await startHostedVrtSession(options, host.spki));
    const browser = (owned.browser = await launchPublicBrowser(host));
    const page = await browser.newPage();
    const spaces = (owned.spaces = await createAddressSpaceObserver(page));
    const errors: string[] = [];
    const consoleErrors: string[] = [];
    const observations: unknown[] = [];
    const first: Partial<Record<LiteralTitle, Buffer[]>> = {};
    page.on("pageerror", (error) => errors.push(String(error)));
    page.on("console", (message) => {
      if (message.type() === "error") consoleErrors.push(message.text());
    });
    try {
      const manifestResponse = await fetch(new URL("api/static.json", url));
      assert.equal(manifestResponse.status, 200);
      assert.deepEqual(Buffer.from(await manifestResponse.arrayBuffer()), f.raw);
      const manifestReceipts: unknown[] = [];
      const emittedUrls = new Set<string>();
      for (const title of ["Controls", "Keys"] as const) {
        const artPath = path.join(f.root, "src", `${title}.art.vue`);
        assert.equal(f.manifest.snapshotIdentities[artPath], `src/${title}.art.vue`);
        const urls = f.manifest.previews[artPath];
        assert.deepEqual(
          Object.keys(urls),
          variants[title].map(([name]) => name),
        );
        for (const [name, button] of variants[title]) {
          assert.equal(Object.hasOwn(urls, name), true);
          assert.equal(typeof urls[name], "string");
          const previewUrl = new URL(urls[name], url);
          assert.equal(previewUrl.origin, host.origin);
          assert.ok(previewUrl.pathname.startsWith("/built/gallery/preview/"));
          assert.equal(emittedUrls.has(previewUrl.href), false);
          emittedUrls.add(previewUrl.href);
          const physical = await readFile(
            path.join(f.directory, previewUrl.pathname.slice("/built/".length)),
          );
          const response = await fetch(previewUrl);
          assert.equal(response.status, 200);
          assert.deepEqual(Buffer.from(await response.arrayBuffer()), physical);
          await page.goto(previewUrl.href);
          await page.getByRole("button", { name: button, exact: true }).waitFor();
          assert.equal(await page.locator("museacomponent").count(), 0);
          manifestReceipts.push({
            title,
            name,
            own: true,
            url: previewUrl.href,
            physicalSha256: sha256(physical),
            html: await page.content(),
          });
        }
      }
      assert.equal(emittedUrls.size, 5);
      observations.push({
        phase: "physical-source-deleted-previews",
        manifestReceipts,
        sources: f.sources,
      });
      await page.goto(url);
      assert.equal(await page.evaluate(() => window.isSecureContext), true);
      owned.detachPermission = await setLoopbackPermission(page, "granted");
      const permission = await page.evaluate(
        async () =>
          (await navigator.permissions.query({ name: "loopback-network" as PermissionName })).state,
      );
      assert.equal(permission, "granted");
      const globals = await page.evaluate(() => {
        const data = Reflect.get(window, "__MUSEA_STATIC_PREVIEWS__");
        return Object.keys(data).map((artPath) => ({
          artPath,
          keys: Object.keys(data[artPath]),
          own: Object.keys(data[artPath]).every(
            (key) => Object.hasOwn(data[artPath], key) && typeof data[artPath][key] === "string",
          ),
          prototypeUnchanged: Object.getPrototypeOf(data[artPath]) === Object.prototype,
          hasOwnToString: Object.hasOwn(data[artPath], "toString"),
        }));
      });
      for (const title of ["Controls", "Keys"] as const) {
        const item = globals.find((value) => value.artPath.endsWith(`/${title}.art.vue`));
        assert.ok(item?.own && item.prototypeUnchanged && !item.hasOwnToString);
        assert.deepEqual(
          item.keys,
          variants[title].map(([name]) => name),
        );
      }
      observations.push({
        phase: "secure-public-loopback",
        permission,
        chromium: browser.version(),
        publicAddressSpaceOverride: new URL(host.origin).host,
        certificateSpki: host.spki,
        certificateSha256: sha256(await readFile(host.certificatePath)),
        globals,
      });
      // Interleave Arts so every later capture also proves the other Art's whole reports unchanged.
      for (const phase of ["new", "match"] as const) {
        for (const title of ["Controls", "Keys"] as const) {
          const other = title === "Controls" ? "Keys" : "Controls";
          const previousOther = await retainedReports(local, other);
          await page.goto(url);
          const rendered = await previews(page, title);
          assert.deepEqual(
            rendered.map((item) => item.sectionName),
            variants[title].map(([name]) => name),
          );
          assert.deepEqual(
            rendered.map((item) => item.sectionId),
            sectionIds[title],
          );
          assert.deepEqual(
            rendered.map((item) => item.ariaControls),
            sectionIds[title],
          );
          assert.equal(
            new Set(rendered.map((item) => item.sectionId)).size,
            variants[title].length,
          );
          assert.ok(
            rendered.every(
              (item, index) =>
                item.unresolved === 0 &&
                item.buttons.includes(variants[title][index][1]) &&
                item.url ===
                  new URL(
                    f.manifest.previews[path.join(f.root, "src", `${title}.art.vue`)][
                      variants[title][index][0]
                    ],
                    url,
                  ).href,
            ),
          );
          await page.locator(".variant-toc-item").last().click();
          await page.locator(".variant-toc-item").first().click();
          await page.locator('.variant-toc-item[aria-current="true"]').first().waitFor();
          assert.equal(
            await page.locator(".variant-toc-item").first().getAttribute("aria-current"),
            "true",
          );
          await page.getByRole("button", { name: "VRT", exact: true }).click();
          await page
            .getByRole("textbox", { name: "VRT endpoint", exact: true })
            .fill(session.endpoint);
          await page.getByLabel("Session token", { exact: true }).fill(session.token);
          await page.getByRole("button", { name: "Connect VRT", exact: true }).click();
          await page.getByText("Connected to your local VRT session.", { exact: true }).waitFor();
          assert.equal(await page.getByLabel("Session token", { exact: true }).count(), 0);
          const result = await captureLiteralService({
            page,
            session,
            origin: host.origin,
            local,
            output: f.output,
            title,
            artPath: path.join(f.root, "src", `${title}.art.vue`),
            phase,
            first: first[title],
          });
          if (phase === "new") first[title] = result.baselines;
          const afterOther = await retainedReports(local, other);
          assert.deepEqual(afterOther, previousOther);
          observations.push({
            ...result.receipt,
            rendered,
            otherArt: {
              title: other,
              unchanged: true,
              reports: Object.entries(afterOther).map(([kind, bytes]) => ({
                kind,
                bytes: bytes.length,
                sha256: sha256(bytes),
              })),
            },
          });
          await page.screenshot({
            path: path.join(f.output, `${title}-${phase}.png`),
            fullPage: true,
          });
          await writeFile(path.join(f.output, `${title}-${phase}.html`), await page.content());
          assert.deepEqual(errors, []);
          assert.deepEqual(consoleErrors, []);
        }
      }
      assert.equal(new Set([...first.Controls!, ...first.Keys!].map(sha256)).size, 5);
      assert.ok(
        spaces.records.some(
          (item) => item.origin === host.origin && item.resourceIPAddressSpace === "Public",
        ),
      );
      assert.ok(
        spaces.records.some(
          (item) =>
            item.origin === session.endpoint &&
            item.initiatorIPAddressSpace === "Public" &&
            item.initiatorIsSecureContext === true,
        ),
      );
      assert.ok(
        spaces.records.some(
          (item) => item.origin === session.endpoint && item.resourceIPAddressSpace === "Loopback",
        ),
      );
      assert.ok(
        host.requests.every(
          (item) => !item.pathname.includes("@vite") && !item.pathname.endsWith(".art.vue"),
        ),
      );
      assert.deepEqual(errors, []);
      assert.deepEqual(consoleErrors, []);
    } finally {
      await mkdir(f.output, { recursive: true });
      const receipt = JSON.stringify(
        {
          observations,
          errors,
          consoleErrors,
          requests: host.requests,
          addressSpaces: spaces.records,
        },
        null,
        2,
      );
      assert.equal(receipt.includes(session.token), false);
      await writeFile(path.join(f.output, "observations.json"), receipt);
    }
  },
);
