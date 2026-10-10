import assert from "node:assert/strict";
import { createServer } from "node:http";
import test from "node:test";
import { loadHostedGallery } from "./hosted.ts";

void test("hosted manifests preserve exact preview URLs and reject missing or foreign previews", async () => {
  const art = {
    path: "/build-machine/Only.art.vue",
    metadata: { title: "Only" },
    variants: [{ name: "Default", skipVrt: false }],
  };
  let payload: unknown = {
    arts: [art],
    previews: { [art.path]: { Default: "/site/components/preview/exact.html?theme=dark" } },
  };
  const requests: string[] = [];
  const server = createServer((request, response) => {
    requests.push(request.url ?? "");
    response.setHeader("Content-Type", "application/json");
    response.end(
      JSON.stringify({
        snapshotIdentityVersion: 1,
        snapshotIdentities: { [art.path]: "Only.art.vue" },
        ...(payload as object),
      }),
    );
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  const origin = `http://127.0.0.1:${address.port}`;
  const url = `${origin}/site/components/?discard=yes#discard`;
  try {
    const hosted = await loadHostedGallery(url);
    assert.deepEqual(hosted.arts, [art]);
    assert.equal(
      hosted.previewUrls[art.path].Default,
      `${origin}/site/components/preview/exact.html?theme=dark`,
    );
    assert.equal(requests[0], "/site/components/api/static.json");
    for (const value of [
      "https://foreign.invalid/preview.html",
      "/site/components-other/preview.html",
      "../outside.html",
      "/site/components/%2e%2e/outside.html",
    ]) {
      payload = { arts: [art], previews: { [art.path]: { Default: value } } };
      await assert.rejects(loadHostedGallery(url), /outside its gallery/);
    }
    payload = { arts: [art], previews: { [art.path]: {} } };
    await assert.rejects(loadHostedGallery(url), /Missing hosted preview/);
    payload = { arts: [art, art], previews: { [art.path]: { Default: "preview/exact.html" } } };
    await assert.rejects(loadHostedGallery(url), /Duplicate hosted art/);
    payload = {
      arts: [{ ...art, variants: [...art.variants, ...art.variants] }],
      previews: { [art.path]: { Default: "preview/exact.html" } },
    };
    await assert.rejects(loadHostedGallery(url), /Duplicate hosted variant/);
    payload = { arts: [{ ...art, variants: [{ name: "Default" }] }], previews: {} };
    await assert.rejects(loadHostedGallery(url), /Missing hosted previews/);
    payload = { arts: [], previews: [] };
    await assert.rejects(loadHostedGallery(url), /must contain arts and previews/);
    for (const invalid of ["file:///local/gallery", "https://user:password@host.invalid/gallery"]) {
      await assert.rejects(
        loadHostedGallery(invalid),
        /HTTP\(S\) URL without embedded credentials/,
      );
    }
  } finally {
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
  }
});
