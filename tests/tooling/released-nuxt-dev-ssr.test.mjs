import assert from "node:assert/strict";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { apiRoot, gitBlob } from "../../tools/support/release/typed_defaults/custody.mjs";
import {
  fixtureRoot,
  loadNuxtFixtures,
  pins,
} from "../../tools/support/release/nuxt_dev_style/custody.mjs";
import {
  copyProject,
  projectInputs,
} from "../../tools/support/release/nuxt_dev_style/projects.mjs";
import {
  verifyHydration,
  verifyInitial,
  verifyStock,
} from "../../tools/support/release/nuxt_dev_style/contracts.mjs";
import { waitForTcp } from "../../tools/support/release/nuxt_dev_style/browser.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url));
const source = new Map(
  Object.keys(pins).map((name) => [name, fs.readFileSync(path.join(root, name))]),
);
const initial = {
  background: "rgb(18, 24, 30)",
  color: "rgb(240, 246, 252)",
  border: "rgb(40, 200, 90)",
  borderWidth: "3px",
  padding: "24px",
  fontSize: "20px",
  lineHeight: "28px",
  width: 420,
  height: 86,
  links: ["http://127.0.0.1:18931/page.css"],
  hydrated: "false",
};
const css = [{ status: 200, contentType: "text/css", body: ".card{}" }];

await test("original Nuxt reproduction, dependency lock and observer remain whole authenticated H blobs", async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-nuxt-fixture-"));
  try {
    const treeSha = "a".repeat(40),
      head = "b".repeat(40);
    const entries = [...source].map(([name, bytes]) => ({
      path: name,
      mode: "100644",
      type: "blob",
      sha: gitBlob(bytes),
    }));
    const blobs = new Map([...source.values()].map((bytes) => [gitBlob(bytes), bytes]));
    const read = async (url) => {
      if (url === `${apiRoot}/git/trees/${treeSha}?recursive=1`)
        return { sha: treeSha, truncated: false, tree: entries };
      const sha = url.split("/").at(-1),
        bytes = blobs.get(sha);
      return { sha, encoding: "base64", size: bytes.length, content: bytes.toString("base64") };
    };
    const fixture = await loadNuxtFixtures(head, treeSha, path.join(directory, "valid"), read);
    assert.equal(fixture.authorities.length, 6);
    const body = Buffer.from("changed source");
    entries[0].sha = gitBlob(body);
    blobs.set(entries[0].sha, body);
    await assert.rejects(
      loadNuxtFixtures(head, treeSha, path.join(directory, "changed"), read),
      /original Nuxt fixture changed/,
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

await test("published and stock projects invert their finite edits to all original manifest/config/SFC bytes", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-nuxt-project-"));
  try {
    const dependencies = path.join(directory, "public-packages");
    fs.mkdirSync(path.join(dependencies, "node_modules"), { recursive: true });
    for (const variant of ["stock", "baseline", "published"]) {
      const project = copyProject(
        path.join(directory, variant),
        { source },
        dependencies,
        variant,
        "0.449.0",
      );
      for (const name of projectInputs) {
        let actual = fs.readFileSync(path.join(project, name), "utf8");
        if (variant === "stock" && name === "nuxt.config.ts")
          actual = actual.replace("modules: []", 'modules: ["@vizejs/nuxt"]');
        assert.equal(actual, source.get(`${fixtureRoot}/${name}`).toString("utf8"));
      }
      let manifest = fs.readFileSync(path.join(project, "package.json"), "utf8");
      if (variant === "published")
        manifest = manifest.replace('"@vizejs/nuxt": "0.449.0"', '"@vizejs/nuxt": "0.443.0"');
      assert.equal(manifest, source.get(`${fixtureRoot}/package.json`).toString("utf8"));
      assert.equal(
        fs.realpathSync(path.join(project, "node_modules")),
        fs.realpathSync(path.join(dependencies, "node_modules")),
      );
      assert.equal(fs.existsSync(path.join(project, "dist")), false);
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

await test("first-response and hydration laws refuse missing CSS, geometry shifts and incomplete vectors", () => {
  verifyInitial("published", initial, css);
  verifyInitial("baseline", { ...initial, background: "rgba(0, 0, 0, 0)", links: [] }, []);
  verifyHydration(initial, { ...initial, hydrated: "true" });
  verifyStock(initial, initial);
  assert.throws(() =>
    verifyInitial("published", { ...initial, background: "rgba(0, 0, 0, 0)" }, css),
  );
  assert.throws(() => verifyInitial("published", initial, []));
  assert.throws(() => verifyInitial("published", initial, [{ ...css[0], status: 404 }]));
  assert.throws(() =>
    verifyInitial("published", initial, [{ ...css[0], contentType: "text/html" }]),
  );
  for (const key of ["color", "fontSize", "lineHeight", "height", "width"]) {
    const changed = {
      ...initial,
      [key]: typeof initial[key] === "number" ? initial[key] + 1 : "changed",
    };
    assert.throws(() => verifyStock(changed, initial));
    assert.throws(() => verifyHydration(initial, { ...changed, hydrated: "true" }));
  }
  assert.throws(() => verifyHydration(initial, { ...initial, hydrated: "false" }));
});

await test("readiness performs only a TCP connection and sends no HTTP warmup", async () => {
  let received = 0;
  const server = net.createServer((socket) => {
    socket.on("data", (chunk) => {
      received += chunk.length;
    });
    socket.on("end", () => socket.end());
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  try {
    await waitForTcp(server.address().port, { exitCode: null });
    await new Promise((resolve) => setImmediate(resolve));
    assert.equal(received, 0);
    await assert.rejects(
      waitForTcp(server.address().port, { exitCode: 1 }),
      /dev server exited before listening/,
    );
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});
