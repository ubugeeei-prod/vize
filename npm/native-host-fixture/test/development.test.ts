import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { startNativeDevelopment } from "../scripts/development.ts";
import { createNativeRefreshFixture } from "../scripts/refresh-fixture.ts";

test("native development refuses invalid ports before starting a compiler or listener", async () => {
  for (const port of [-1, 65536, NaN, Infinity, 1.5]) {
    await assert.rejects(startNativeDevelopment({ port }), /port must be an integer/);
  }
});

test("a loopback native development server emits real SFC HMR and refuses non-native styles", async () => {
  const output = mkdtempSync(path.join(os.tmpdir(), "vize-native-development-test-"));
  const fixture = await createNativeRefreshFixture(
    output,
    process.env.VIZE_NATIVE_HOST_COMPILER_PATH,
  );
  try {
    const page = await fetch(fixture.origin);
    assert.equal(page.status, 200);
    assert.match(await page.text(), /src="\/@vite\/client"/);
    assert.equal(page.headers.get("access-control-allow-origin"), null);
    assert.ok(
      page.headers
        .get("content-security-policy")
        ?.includes(`connect-src 'self' ${fixture.origin.replace("http:", "ws:").slice(0, -1)}`),
    );
    const entryResponse = await fetch(new URL("src/entry.ts", fixture.origin));
    assert.equal(entryResponse.status, 200);
    const entry = await entryResponse.text();
    const application = /import\s+App\s+from\s+["']([^"']+)["']/.exec(entry)?.[1];
    assert.ok(application);
    const readApplication = async (status = 200) => {
      const response = await fetch(new URL(application, fixture.origin));
      assert.equal(response.status, status);
      return response.text();
    };
    const initial = await readApplication();
    assert.match(initial, /__VUE_HMR_RUNTIME__/);
    assert.match(initial, /Count:/);
    assert.doesNotMatch(initial, /Refreshed count:/);
    fixture.observe("VIZE_NATIVE_TEMPLATE_");
    fixture.observe("REFRESH_READY\n");
    await fixture.waitForTemplateUpdate();
    const updated = await readApplication();
    assert.match(updated, /Refreshed count:/);
    assert.match(updated, /__hmrUpdateType\s*=\s*"template-only"/);
    fixture.finish();
    const style = /import\s+["']([^"']+\.__vize_style_[^"']+)["']/.exec(updated)?.[1];
    assert.ok(style);
    assert.equal((await fetch(new URL(style, fixture.origin))).status, 200);
    const source = path.join(output, "development-source/src/App.vue");
    writeFileSync(
      source,
      readFileSync(source, "utf8").replace(
        /<style[^>]*>[\s\S]*?<\/style>/,
        "<style>native-text { color: red }</style>",
      ),
    );
    await fixture.waitForSourceRefusal();
    const cached = await fetch(new URL(style, fixture.origin));
    assert.equal(cached.status, 200);
    assert.doesNotMatch(await cached.text(), /color:\s*red/);
    const uncachedStyle = new URL(style, fixture.origin);
    uncachedStyle.searchParams.set("native-admission-probe", "fresh");
    const response = await fetch(uncachedStyle);
    assert.equal(response.status, 500);
    const invalid = await response.text();
    assert.match(invalid, /canonical SFC sections and empty native styles/);
  } finally {
    await fixture.close();
  }
});
