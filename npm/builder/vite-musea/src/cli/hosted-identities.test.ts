import assert from "node:assert/strict";
import { createServer } from "node:http";
import test from "node:test";
import { loadHostedGallery } from "./hosted.ts";
import { createVrtOptions } from "./commands.ts";
import { parseArgs } from "./index.ts";

void test("hosted captures require complete portable identities from the versioned manifest", async () => {
  const arts = ["left", "right"].map((side) => ({
    path: `/build/${side}/Button.art.vue`,
    metadata: { title: side },
    variants: [{ name: "Default", skipVrt: false }],
  }));
  const original = {
    arts,
    previews: Object.fromEntries(arts.map((art) => [art.path, { Default: "preview/art.html" }])),
    snapshotIdentityVersion: 1,
    snapshotIdentities: Object.fromEntries(
      arts.map((art, index) => [art.path, `${index === 0 ? "left" : "right"}/Button.art.vue`]),
    ),
  };
  let payload: unknown = original;
  const server = createServer((_request, response) => {
    response.setHeader("Content-Type", "application/json");
    response.end(JSON.stringify(payload));
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  const url = `http://127.0.0.1:${address.port}/gallery/`;
  try {
    const hosted = await loadHostedGallery(url);
    assert.equal(hosted.snapshotIdentities[arts[0].path], "left/Button.art.vue");
    assert.equal(hosted.snapshotIdentities[arts[1].path], "right/Button.art.vue");
    for (const version of [undefined, 0, 2, "1"]) {
      payload = { ...original, snapshotIdentityVersion: version };
      await assert.rejects(loadHostedGallery(url), /requires snapshot identity version 1.*rebuild/);
    }
    payload = { ...original, snapshotIdentities: {} };
    await assert.rejects(loadHostedGallery(url), /Missing hosted snapshot identity/);
    for (const identity of [
      "/build/Button.art.vue",
      "../Button.art.vue",
      "left\\Button.art.vue",
      "left//Button.art.vue",
    ]) {
      payload = {
        ...original,
        snapshotIdentities: { ...original.snapshotIdentities, [arts[0].path]: identity },
      };
      await assert.rejects(loadHostedGallery(url), /Invalid project-relative snapshot identity/);
    }
    payload = {
      ...original,
      snapshotIdentities: { [arts[0].path]: "Button.art.vue", [arts[1].path]: "Button.art.vue" },
    };
    await assert.rejects(loadHostedGallery(url), /Duplicate hosted snapshot identity/);
  } finally {
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
  }
});

void test("CLI explicitly passes legacy adoption and exact hosted identities into the runner", () => {
  const options = parseArgs(["--adopt-legacy-snapshots"]);
  options.projectRoot = "/project";
  options.snapshotIdentities = { "/build/Button.art.vue": "src/Button.art.vue" };
  const vrt = createVrtOptions(options);
  assert.equal(vrt.adoptLegacySnapshots, true);
  assert.equal(vrt.projectRoot, "/project");
  assert.deepEqual(vrt.snapshotIdentities, options.snapshotIdentities);
  assert.equal(createVrtOptions(parseArgs([])).adoptLegacySnapshots, undefined);
});
