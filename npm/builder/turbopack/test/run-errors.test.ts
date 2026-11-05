import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { checkHostErrors } from "./run-errors.mjs";

interface HostResponse {
  readonly status: number;
  readonly html: string;
}

const frameworkError =
  "Could not parse module '[project]/components/组件 space/FixtureCounter.vue.js', file not found";
const vizeDiagnostic = "[vize] components/组件 space/FixtureCounter.vue:\nInvalid end tag.";
const recovered = "<html><body>Counter 2 🐱</body></html>";
const vaporDiagnostic =
  "[vize] components/组件 space/FixtureCounter.vue: Vapor SFC blocks are not supported by the stable Turbopack integration.";
const original = `<template>
  <section>placeholder</section>
</template>
<script setup>
const count = 2;
</script>
`;

void test("checkHostErrors waits through 500 framework error, 500 [vize] diagnostic, then 200 recovered", async () => {
  assert.match(frameworkError, /Could not parse module/);
  assert.match(frameworkError, /FixtureCounter\.vue/);
  assert.match(frameworkError, /file not found/);
  assert.doesNotMatch(frameworkError, /\[vize\]/);
  const responses: readonly HostResponse[] = [
    { status: 500, html: frameworkError },
    { status: 500, html: vizeDiagnostic },
    { status: 200, html: recovered },
    { status: 500, html: vaporDiagnostic },
    { status: 200, html: recovered },
    { status: 500, html: vaporDiagnostic },
    { status: 200, html: recovered },
  ];
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "vize-turbopack-host-errors-"));
  const previousArtifacts = process.env.VIZE_TURBOPACK_ARTIFACTS;
  const previousFetch = globalThis.fetch;
  const seen: HostResponse[] = [];
  let index = 0;
  try {
    process.env.VIZE_TURBOPACK_ARTIFACTS = root;
    const filename = path.join(root, "FixtureCounter.vue");
    await fs.writeFile(filename, original);
    const url = "http://127.0.0.1:9/errors";
    globalThis.fetch = (async (input: string | URL | Request) => {
      const requested =
        typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
      assert.equal(requested, url);
      const next = responses[index];
      index += 1;
      assert.ok(next, "response sequence exhausted");
      seen.push(next);
      return new Response(next.html, { status: next.status });
    }) as typeof fetch;
    const measurements: { phase: string; ms: number }[] = [];
    await checkHostErrors(url, filename, original, measurements);
    assert.equal(index, responses.length);
    assert.deepEqual(
      seen.map((response) => response.status),
      [500, 500, 200, 500, 200, 500, 200],
    );
    assert.equal(seen[0]?.html, frameworkError);
    assert.equal(await fs.readFile(path.join(root, "invalid-sfc.html"), "utf8"), vizeDiagnostic);
    assert.equal(
      await fs.readFile(path.join(root, "invalid-sfc.json"), "utf8"),
      '{"status":500}\n',
    );
    assert.match(await fs.readFile(path.join(root, "recovered-sfc.html"), "utf8"), /Counter 2 🐱/);
    assert.match(
      await fs.readFile(path.join(root, "vapor-presence.html"), "utf8"),
      /Vapor SFC blocks are not supported/,
    );
    assert.equal(await fs.readFile(filename, "utf8"), original);
    assert.deepEqual(
      measurements.map((entry) => entry.phase),
      [
        "invalid-sfc",
        "recovered-sfc",
        "vapor-presence",
        "vapor-presence-recovered",
        "vapor-false-value",
        "vapor-false-value-recovered",
      ],
    );
  } finally {
    globalThis.fetch = previousFetch;
    if (previousArtifacts === undefined) delete process.env.VIZE_TURBOPACK_ARTIFACTS;
    else process.env.VIZE_TURBOPACK_ARTIFACTS = previousArtifacts;
    await fs.rm(root, { recursive: true, force: true });
  }
});
