import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { mkdir, mkdtemp, readFile, realpath, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { collectRuntimePackageInventory } from "./runtime-package-inventory.ts";

const hash = (source: string | Buffer) => createHash("sha256").update(source).digest("hex");

async function fixture(run: (f: Awaited<ReturnType<typeof createFixture>>) => Promise<void>) {
  const f = await createFixture();
  try {
    await run(f);
  } finally {
    await rm(f.root, { recursive: true, force: true });
  }
}

async function createFixture() {
  const root = await realpath(await mkdtemp(join(tmpdir(), "runtime-manifest-inventory-")));
  const server = join(root, "deploy/server");
  const modules = join(server, "node_modules");
  const physical = join(modules, ".bun/linked/node_modules/@vue/shared");
  const linked = join(modules, "@vue/shared");
  const runtime = `${JSON.stringify({ name: "observer-fixture", version: "0.0.0", private: true, dependencies: { "@vue/shared": "3.5.46" } }, null, 2)}\n`;
  const source = `${JSON.stringify({ name: "@vue/shared", version: "3.5.46", description: "full source 漢字 😀", exports: { ".": "./index.js" }, custom: { retained: true } }, null, 2)}\n`;
  await mkdir(physical, { recursive: true });
  await mkdir(dirname(linked), { recursive: true });
  await writeFile(join(server, "package.json"), runtime);
  await writeFile(join(physical, "package.json"), source);
  await symlink(relative(dirname(linked), physical), linked, "dir");
  return { root, server, modules, physical, linked, runtime, source };
}

test("linked and retained versions preserve every full manifest source and direct resolution", async () => {
  await fixture(async (f) => {
    const old = join(f.modules, "retained/node_modules/@vue/shared");
    const oldSource = '{"name":"@vue/shared","version":"3.5.43","old":true}\n';
    await mkdir(old, { recursive: true });
    await writeFile(join(old, "package.json"), oldSource);
    const result = await collectRuntimePackageInventory(f.server, f.runtime);
    assert.deepEqual(result.packages, { "@vue/shared": ["3.5.43", "3.5.46"] });
    assert.deepEqual(result.runtimeManifest, {
      path: join(f.server, "package.json"),
      bytes: Buffer.byteLength(f.runtime),
      sha256: hash(f.runtime),
      source: f.runtime,
      name: "observer-fixture",
      version: "0.0.0",
    });
    assert.deepEqual(result.direct, [
      { name: "@vue/shared", version: "3.5.46", manifest: join(f.physical, "package.json") },
    ]);
    assert.deepEqual(
      result.manifests.map(({ path, source, sha256 }) => ({ path, source, sha256 })),
      [
        { path: join(f.physical, "package.json"), source: f.source, sha256: hash(f.source) },
        { path: join(old, "package.json"), source: oldSource, sha256: hash(oldSource) },
        { path: join(f.server, "package.json"), source: f.runtime, sha256: hash(f.runtime) },
      ].sort((a, b) => (a.path < b.path ? -1 : Number(a.path > b.path))),
    );
  });
});

test("physical aliases and directory cycles terminate without losing or duplicating manifests", async () => {
  await fixture(async (f) => {
    await symlink(relative(f.modules, f.physical), join(f.modules, "alias"), "dir");
    await mkdir(join(f.physical, "node_modules"));
    await symlink("..", join(f.physical, "node_modules/self"), "dir");
    await symlink(
      relative(join(f.physical, "node_modules"), f.modules),
      join(f.physical, "node_modules/all"),
      "dir",
    );
    await mkdir(join(f.modules, "empty"));
    await symlink(".", join(f.modules, "empty/again"), "dir");
    const result = await collectRuntimePackageInventory(f.server, f.runtime, {
      maximumEntries: 64,
      maximumManifests: 2,
    });
    assert.deepEqual(result.packages, { "@vue/shared": ["3.5.46"] });
    assert.equal(result.manifests.length, 2);
  });
});

test("a linked package.json preserves its actual physical source", async () => {
  await fixture(async (f) => {
    await rm(join(f.physical, "package.json"));
    await writeFile(join(f.physical, "metadata.json"), f.source);
    await symlink("metadata.json", join(f.physical, "package.json"));
    const result = await collectRuntimePackageInventory(f.server, f.runtime);
    assert.deepEqual(result.direct, [
      { name: "@vue/shared", version: "3.5.46", manifest: join(f.physical, "metadata.json") },
    ]);
    assert.equal(
      result.manifests.find((m) => m.path === join(f.physical, "metadata.json"))?.source,
      f.source,
    );
  });
});

test("ordinary installs keep the prior inventory while unnamed manifests retain full source", async () => {
  await fixture(async (f) => {
    await rm(f.linked);
    await mkdir(join(f.linked, "lib"), { recursive: true });
    await writeFile(join(f.linked, "package.json"), f.source);
    const source = '{"type":"module","fixture":"unnamed metadata"}\n';
    await writeFile(join(f.linked, "lib/package.json"), source);
    const result = await collectRuntimePackageInventory(f.server, f.runtime);
    assert.deepEqual(result.packages, { "@vue/shared": ["3.5.46"] });
    assert.deepEqual(
      result.manifests.find((m) => m.path === join(f.linked, "lib/package.json")),
      {
        path: join(f.linked, "lib/package.json"),
        bytes: Buffer.byteLength(source),
        sha256: hash(source),
        source,
        name: null,
        version: null,
      },
    );
  });
});

test("broken links and link-only cycles refuse a partial inventory", async () => {
  for (const cycle of [false, true])
    await fixture(async (f) => {
      await symlink(cycle ? "other" : "absent", join(f.modules, "broken"), "dir");
      if (cycle) await symlink("broken", join(f.modules, "other"), "dir");
      await assert.rejects(collectRuntimePackageInventory(f.server, f.runtime), {
        code: cycle ? "ELOOP" : "ENOENT",
      });
    });
});

test("missing, foreign and mismatched direct manifests cannot be inferred from declarations", async () => {
  for (const mode of ["missing", "foreign", "version"] as const)
    await fixture(async (f) => {
      await rm(join(f.physical, "package.json"));
      if (mode !== "missing")
        await writeFile(
          join(f.physical, "package.json"),
          JSON.stringify({
            name: mode === "foreign" ? "unrelated" : "@vue/shared",
            version: mode === "version" ? "3.5.43" : "3.5.46",
          }),
        );
      await assert.rejects(
        collectRuntimePackageInventory(f.server, f.runtime),
        mode === "missing" ? { code: "ENOENT" } : /Runtime direct resolution differs/,
      );
    });
});

test("captured runtime source changes, non-exact dependencies and escaped names fail closed", async () => {
  await fixture(async (f) => {
    await assert.rejects(
      collectRuntimePackageInventory(f.server, f.runtime + " "),
      /changed after capture/,
    );
    for (const dependencies of [
      { "@vue/shared": "^3.5.46" },
      { "../outside": "3.5.46" },
      { "@vue/../outside": "3.5.46" },
    ]) {
      const source = JSON.stringify({ dependencies });
      await writeFile(join(f.server, "package.json"), source);
      await assert.rejects(
        collectRuntimePackageInventory(f.server, source),
        /not exact|Invalid runtime dependency name/,
      );
    }
  });
});

test("manifest byte/count bounds include their exact boundary and all physical sources", async () => {
  await fixture(async (f) => {
    const maximum = Math.max(Buffer.byteLength(f.runtime), Buffer.byteLength(f.source));
    const total = Buffer.byteLength(f.runtime) + Buffer.byteLength(f.source);
    assert.deepEqual(
      (
        await collectRuntimePackageInventory(f.server, f.runtime, {
          maximumManifestBytes: maximum,
          maximumTotalManifestBytes: total,
          maximumManifests: 2,
        })
      ).packages,
      { "@vue/shared": ["3.5.46"] },
    );
    await assert.rejects(
      collectRuntimePackageInventory(f.server, f.runtime, { maximumManifestBytes: maximum - 1 }),
      /byte bound/,
    );
    await assert.rejects(
      collectRuntimePackageInventory(f.server, f.runtime, { maximumTotalManifestBytes: total - 1 }),
      /aggregate bound/,
    );
    await assert.rejects(
      collectRuntimePackageInventory(f.server, f.runtime, { maximumManifests: 1 }),
      /manifest count/,
    );
    await assert.rejects(
      collectRuntimePackageInventory(f.server, f.runtime, { maximumEntries: 1 }),
      /entries exceed/,
    );
  });
});

test("invalid limits cannot disable or expand the production bounds", async () => {
  await fixture(async (f) => {
    for (const value of [0, -1, 1.5, NaN, Infinity, 200_001, null, "1"]) {
      await assert.rejects(
        collectRuntimePackageInventory(f.server, f.runtime, { maximumEntries: value as number }),
        /Invalid inventory limit/,
      );
    }
  });
});

test("malformed, non-file and invalid UTF-8 manifests refuse a partial inventory", async () => {
  await fixture(async (f) => {
    await writeFile(join(f.physical, "package.json"), "{");
    await assert.rejects(collectRuntimePackageInventory(f.server, f.runtime), SyntaxError);
    await rm(join(f.physical, "package.json"));
    await mkdir(join(f.physical, "package.json"));
    await assert.rejects(
      collectRuntimePackageInventory(f.server, f.runtime),
      /not a regular file|EISDIR/,
    );
    await rm(join(f.physical, "package.json"), { recursive: true });
    await writeFile(join(f.physical, "package.json"), Buffer.from([0xff]));
    await assert.rejects(collectRuntimePackageInventory(f.server, f.runtime), TypeError);
  });
});

test("a non-regular direct manifest refuses without opening a blocking stream", async () => {
  await fixture(async (f) => {
    const path = join(f.physical, "package.json");
    await rm(path);
    const fifo = spawnSync("mkfifo", [path], { encoding: "utf8", timeout: 1_000 });
    assert.equal(fifo.status, 0, fifo.stderr);
    await assert.rejects(collectRuntimePackageInventory(f.server, f.runtime), /not a regular file/);
  });
});

test("a UTF-8 BOM is not silently removed from manifest source evidence", async () => {
  await fixture(async (f) => {
    const extra = join(f.modules, "extra");
    await mkdir(extra);
    const source = '{"name":"extra","version":"1.0.0"}';
    await writeFile(
      join(extra, "package.json"),
      Buffer.concat([Buffer.from([0xef, 0xbb, 0xbf]), Buffer.from(source)]),
    );
    await assert.rejects(collectRuntimePackageInventory(f.server, f.runtime), SyntaxError);
  });
});

test("a large UTF-8 manifest retains its complete bytes across reads", async () => {
  await fixture(async (f) => {
    const source = JSON.stringify({
      name: "@vue/shared",
      version: "3.5.46",
      description: "漢字😀".repeat(8_000),
    });
    await writeFile(join(f.physical, "package.json"), source);
    const result = await collectRuntimePackageInventory(f.server, f.runtime);
    assert.deepEqual(result.packages, { "@vue/shared": ["3.5.46"] });
    assert.deepEqual(
      result.manifests.find((m) => m.path === join(f.physical, "package.json")),
      {
        path: join(f.physical, "package.json"),
        bytes: Buffer.byteLength(source),
        sha256: hash(source),
        source,
        name: "@vue/shared",
        version: "3.5.46",
      },
    );
  });
});

test("the staged observer binds the corrected inventory and full sources into its receipt", async () => {
  await fixture(async (f) => {
    const old = join(f.modules, "retained/node_modules/@vue/shared");
    await mkdir(old, { recursive: true });
    await writeFile(join(old, "package.json"), '{"name":"@vue/shared","version":"3.5.43"}');
    const project = join(f.root, "project");
    const evidence = join(f.root, "evidence");
    await mkdir(evidence);
    await mkdir(join(project, ".output/public/_nuxt"), { recursive: true });
    await writeFile(join(project, ".output/public/_nuxt/client.js"), "observer fixture asset");
    await writeFile(join(project, "bun.lock"), "public registry metadata fixture");
    await writeFile(join(evidence, "response.html"), "vize-public-nuxt-bun metadata fixture");
    for (const name of ["@vizejs/native", "@vizejs/nuxt", "@vizejs/vite-plugin", "vize"]) {
      const path = join(project, "node_modules", name);
      await mkdir(path, { recursive: true });
      await writeFile(join(path, "package.json"), JSON.stringify({ name, version: "0.813.0" }));
    }
    const program =
      process.env.VIZE_RUNTIME_OBSERVER_SCRIPT ??
      fileURLToPath(
        new URL(
          "./install-build-and-serve-the-exact-public-nuxt-integration-with-bun-2.ts",
          import.meta.url,
        ),
      );
    const runtime = process.versions.bun ? process.execPath : (process.env.BUN_BIN ?? "bun");
    const execute = () =>
      spawnSync(runtime, [program], {
        encoding: "utf8",
        timeout: 5_000,
        env: {
          ...process.env,
          PROJECT: project,
          DEPLOY: dirname(f.server),
          EVIDENCE: evidence,
          RELEASE_VERSION: "0.813.0",
        },
      });
    const result = execute();
    assert.equal(result.status, 0, result.stderr);
    const inventory = await readFile(join(evidence, "server-packages.json"), "utf8");
    assert.deepEqual(JSON.parse(inventory), { "@vue/shared": ["3.5.43", "3.5.46"] });
    const sources = await readFile(join(evidence, "server-manifests.json"), "utf8");
    const receipt = JSON.parse(await readFile(join(evidence, "receipt.json"), "utf8"));
    assert.equal(receipt.runtimeInventorySha256, hash(inventory));
    assert.equal(receipt.runtimeManifestSourcesSha256, hash(sources));
    assert.equal(JSON.parse(sources).runtime.source, f.runtime);
    assert.deepEqual(JSON.parse(sources).direct, [
      { name: "@vue/shared", version: "3.5.46", manifest: join(f.physical, "package.json") },
    ]);
    await rm(evidence, { recursive: true });
    await mkdir(evidence);
    await writeFile(join(evidence, "response.html"), "observer fixture");
    await symlink("absent", join(f.modules, "broken"), "dir");
    const refused = execute();
    assert.equal(refused.status, 1);
    assert.match(refused.stderr, /ENOENT/);
    assert.equal(existsSync(join(evidence, "receipt.json")), false);
    assert.equal(existsSync(join(evidence, "server-packages.json")), false);
    assert.equal(existsSync(join(evidence, "server-manifests.json")), false);
  });
});
