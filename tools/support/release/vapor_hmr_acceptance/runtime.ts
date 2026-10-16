import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { publicNative } from "../public_acceptance/installed.ts";

const output = path.resolve(process.env.PUBLIC_OUTPUT!);
const preparation = JSON.parse(fs.readFileSync(path.join(output, "preparation.json"), "utf8"));
const publication = JSON.parse(fs.readFileSync(path.join(output, "publication.json"), "utf8"));
const root = fs.realpathSync(process.cwd());
assert.equal(root, preparation.consumer);
const require = createRequire(path.join(root, "package.json"));
const digest = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
for (const item of [...preparation.fixtures, ...preparation.harness]) {
  assert.equal(digest(fs.readFileSync(path.join(root, item.file))), item.sha256, item.file);
}
const { native, packages, loaded } = publicNative(preparation.version);
assert.equal(typeof native.compileSfc, "function");
if (!preparation.baseline) {
  for (const item of packages) {
    const authority = publication.npm.find((entry: { name: string }) => entry.name === item.name);
    assert.ok(authority, item.name);
    assert.equal(item.integrity, authority.integrity);
    assert.equal(item.resolved, authority.tarball);
  }
}
const lock = JSON.parse(fs.readFileSync("package-lock.json", "utf8"));
const runtimes = ["vue", "@vue/runtime-vapor", "vue-router"].map((name) => {
  const entry = lock.packages[`node_modules/${name}`];
  assert.equal(entry.version, name === "vue-router" ? "4.5.1" : preparation.vueVersion);
  assert.equal(entry.link, undefined);
  assert.equal(new URL(entry.resolved).origin, "https://registry.npmjs.org");
  assert.match(entry.integrity, /^sha512-[A-Za-z0-9+/]+={0,2}$/u);
  return { name, version: entry.version, resolved: entry.resolved, integrity: entry.integrity };
});
const { createServer } = await import("vite");
const { chromium } = await import("playwright");
const pluginEntry = fs.realpathSync(require.resolve("@vizejs/vite-plugin"));
const { default: vize } = await import(pluginEntry);
const receipt: Record<string, unknown> = {
  schema: "vize-9180-installed-vite-hmr-v1",
  preparation,
  packages,
  loaded,
  runtimes,
  plugin: { entry: pluginEntry, sha256: digest(fs.readFileSync(pluginEntry)) },
  observations: [],
  saves: [],
  errors: [],
  success: false,
};
const persist = () =>
  fs.writeFileSync(path.join(output, "hmr.json"), JSON.stringify(receipt, null, 2) + "\n");
persist();
const server = await createServer({
  root,
  configFile: false,
  plugins: [vize({ vapor: true })],
  resolve: { alias: { vue: require.resolve("vue/dist/vue.runtime.esm-bundler.js") } },
  server: { host: "127.0.0.1", port: 0, watch: { usePolling: true, interval: 100 } },
  logLevel: "warn",
});
const browser = await chromium.launch({ headless: true });
const context = await browser.newContext();
await context.tracing.start({ screenshots: true, snapshots: true, sources: true });
const page = await context.newPage();
const browserErrors: string[] = [];
page.on("pageerror", (error) => browserErrors.push(String(error)));
page.on("console", (message) => {
  if (["error", "warning"].includes(message.type())) browserErrors.push(message.text());
});
const observations: Record<string, unknown>[] = [];
const saves: Record<string, unknown>[] = [];
receipt.observations = observations;
receipt.saves = saves;
receipt.errors = browserErrors;
const observe = async (phase: string) => {
  const result =
    await page.evaluate(`({ phase: ${JSON.stringify(phase)}, origin: performance.timeOrigin,
    html: document.querySelector('#app').innerHTML, state: window.__acceptance })`);
  observations.push(result);
  persist();
  return result;
};
const waitCanvas = async (text: string) => {
  await page.waitForFunction(
    (expected) => document.querySelector("#canvas")?.textContent === expected,
    text,
  );
};
const saveChild = (revision: string) => {
  const file = path.join(root, "src/Canvas.vue");
  const before = fs.readFileSync(file, "utf8");
  const after = before.replace(/const revision = '[a-z]+';/u, `const revision = '${revision}';`);
  assert.notEqual(before, after);
  fs.writeFileSync(file, after);
  saves.push({
    revision,
    at: new Date().toISOString(),
    before,
    after,
    beforeSha256: digest(before),
    afterSha256: digest(after),
  });
  persist();
};
try {
  await server.listen();
  const address = server.httpServer!.address();
  assert.ok(address && typeof address === "object");
  await page.goto(`http://127.0.0.1:${address.port}/room/canvas`);
  await waitCanvas("room-0:first");
  const initial = await observe("mount");
  assert.equal(initial.state.vueVersion, preparation.vueVersion);
  assert.deepEqual(initial.state.errors, []);
  assert.deepEqual(initial.state.warnings, []);
  await page.locator("#increment").click();
  await waitCanvas("room-1:first");
  await observe("increment-before-hmr");
  for (const [index, revision] of ["second", "third"].entries()) {
    saveChild(revision);
    if (preparation.baseline) {
      await page.waitForFunction(() =>
        (window as unknown as { __acceptance: { errors: string[] } }).__acceptance.errors.some(
          (error) => /Cannot read properties of undefined \(reading 'inRoom'\)/u.test(error),
        ),
      );
      const failure = await observe("expected-public-baseline-failure");
      assert.equal(failure.origin, initial.origin);
      assert.equal(
        failure.state.events.filter((event: unknown) => event === "parent-setup").length,
        1,
      );
      assert.ok(
        failure.state.hmr.some((event: { phase: string }) => event.phase === "afterUpdate"),
      );
      assert.ok(
        !failure.state.hmr.some((event: { phase: string }) => event.phase === "beforeFullReload"),
      );
      receipt.expectedReportedFailureObserved = true;
      break;
    }
    await waitCanvas(`room-${index + 1}:${revision}`);
    await page.waitForFunction(
      (count) =>
        (
          window as unknown as { __acceptance: { hmr: { phase: string }[] } }
        ).__acceptance.hmr.filter((event) => event.phase === "afterUpdate").length === count,
      index + 1,
    );
    await observe(`save-${revision}`);
    await page.locator("#increment").click();
    await waitCanvas(`room-${index + 2}:${revision}`);
    await observe(`increment-after-${revision}`);
  }
  if (!preparation.baseline) {
    await page.locator("#to-away").click();
    await page.waitForFunction(() => document.querySelector("#away")?.textContent === "room-3");
    await observe("away");
    await page.locator("#to-canvas").click();
    await waitCanvas("room-3:third");
    const final = await observe("return-to-canvas");
    assert.deepEqual(final.state.events, ["parent-setup"]);
    assert.equal(
      final.state.hmr.filter((event: { phase: string }) => event.phase === "afterUpdate").length,
      2,
    );
    assert.equal(
      final.state.hmr.filter((event: { phase: string }) => event.phase === "beforeUpdate").length,
      2,
    );
    for (const observation of observations) {
      const state = observation.state as {
        errors: unknown[];
        warnings: unknown[];
        hmr: { phase: string }[];
      };
      assert.equal(observation.origin, initial.origin, "physical edits must preserve the document");
      assert.deepEqual(state.errors, []);
      assert.deepEqual(state.warnings, []);
      assert.ok(!state.hmr.some((event) => event.phase === "beforeFullReload"));
    }
    assert.deepEqual(browserErrors, []);
  }
  receipt.success = true;
  console.log(
    JSON.stringify({
      version: preparation.version,
      baseline: preparation.baseline,
      observations: observations.length,
      success: true,
    }),
  );
} catch (error) {
  receipt.failure = String(error);
  await observe("failure").catch(() => {});
  throw error;
} finally {
  persist();
  await page.screenshot({ path: path.join(output, "final.png"), fullPage: true }).catch(() => {});
  await context.tracing.stop({ path: path.join(output, "trace.zip") });
  await browser.close();
  await server.close();
  fs.copyFileSync("package-lock.json", path.join(output, "package-lock.json"));
}
