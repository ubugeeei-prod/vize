import assert from "node:assert/strict";
import test from "node:test";

import { createPreviewFixtureServer } from "./browser-fixtures/preview-server.ts";
import { MuseaTestRunner } from "./test-runner.ts";
import type { ArtFileInfo } from "./types/index.ts";

void test(
  "the interaction runner returns browser trace without screenshot work",
  { skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1" },
  async () => {
    const server = await createPreviewFixtureServer(
      '<main class="musea-variant"><button>Increment</button></main>',
      "interaction-trace.js",
    );
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    const address = server.address();
    assert.ok(address && typeof address !== "string");
    const art: ArtFileInfo = {
      path: "src/Counter.art.vue",
      metadata: { title: "Counter", tags: [], status: "ready" },
      variants: [
        {
          name: "Interactive",
          template: "<Counter />",
          isDefault: true,
          skipVrt: false,
          play: "async () => {}",
        },
      ],
      hasScriptSetup: true,
      hasScript: false,
      styleCount: 0,
    };
    const url = `http://127.0.0.1:${address.port}/preview`;
    const runner = new MuseaTestRunner({
      previewUrls: { [art.path]: { Interactive: url } },
    });

    try {
      await runner.init();
      const [result] = await runner.runAllTests([art], url);
      assert.equal(result.status, "passed");
      assert.equal(result.duration, 2);
      assert.deepEqual(result.trace, [
        {
          kind: "action",
          name: "click",
          status: "passed",
          duration: 1,
          target: "button",
        },
      ]);
    } finally {
      await runner.close();
      await new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      );
    }
  },
);
