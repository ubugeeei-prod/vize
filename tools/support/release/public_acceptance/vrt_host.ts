import assert from "node:assert/strict";
import { cp, mkdir, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import path from "node:path";
import { galleryPath, sha256, sides, writeVrtConfig, type VrtFixture } from "./vrt_fixtures.ts";
import { inventory, type Evidence } from "./vrt_artifacts.ts";

export const hostedPath = `/built${galleryPath}`;
export async function buildHostedVrt(
  f: VrtFixture,
  e: Evidence,
  phase: string,
  green: boolean,
  right = true,
) {
  for (const side of sides) {
    await mkdir(path.dirname(f.arts[side]), { recursive: true });
    if (side === "right" && !right) continue;
    const bytes =
      side === "left" && green
        ? f.authored.left.toString().replace("#0000ff", "#00ff00")
        : f.authored[side];
    await writeFile(f.arts[side], bytes);
    await e.save(`${phase}/inputs/${side}.art.vue`, bytes);
  }
  const setup = `export default async function() {
  await new Promise(resolve => setTimeout(resolve, ${f.delay}));
  Reflect.set(window, '__installedHostedSetupAfterLoad', document.readyState === 'complete');
}\n`;
  await writeFile(path.join(f.root, "vrt-delayed.setup.ts"), setup);
  await e.save(`${phase}/inputs/vrt-delayed.setup.ts`, setup);
  writeVrtConfig(f, true);
  await e.save(`${phase}/inputs/vite.config.mjs`, await readFile(f.config));
  const { build } = await import("vite");
  const directory = path.join(f.root, "dist");
  await build({
    root: f.root,
    configFile: f.config,
    logLevel: "warn",
    build: { outDir: directory, emptyOutDir: true },
  });
  const raw = await readFile(path.join(directory, "gallery/vrt/api/static.json"));
  await e.save(`${phase}/static.json`, raw);
  const manifest = JSON.parse(raw.toString()) as {
    snapshotIdentityVersion: number;
    snapshotIdentities: Record<string, string>;
    previews: Record<string, Record<string, string>>;
    arts: Array<{ path: string; variants: Array<{ name: string }> }>;
  };
  assert.equal(manifest.snapshotIdentityVersion, 1);
  assert.equal(manifest.arts.length, right ? 2 : 1);
  for (const art of manifest.arts) {
    const side = sides.find((side) => f.arts[side] === art.path);
    assert.ok(side, art.path);
    assert.equal(manifest.snapshotIdentities[art.path], `src/${side}/Button.art.vue`);
    assert.deepEqual(
      art.variants.map((variant) => variant.name),
      ["Default"],
    );
    assert.equal(Object.hasOwn(manifest.previews, art.path), true);
    assert.deepEqual(Object.keys(manifest.previews[art.path]), ["Default"]);
    assert.ok(manifest.previews[art.path].Default.startsWith(`${hostedPath}preview/`));
  }
  await cp(directory, path.join(e.output, phase, "dist"), { recursive: true });
  await rm(path.join(f.root, "src"), { recursive: true });
  await rm(path.join(f.root, "vrt-delayed.setup.ts"));
  await assert.rejects(readFile(f.arts.left), { code: "ENOENT" });
  await assert.rejects(readFile(f.arts.right), { code: "ENOENT" });
  e.records.push({
    phase,
    manifest,
    manifestSha256: sha256(raw),
    dist: await inventory(directory),
    authoredSourcesDeleted: true,
  });
  return { directory, manifest, raw };
}

/** Only real emitted static bytes are served; no preview, compiler, API or runner substitution. */
export async function staticVrtHost(directory: string, e: Evidence) {
  const requests: unknown[] = [];
  const saved = new Set<string>();
  let refuseAxe = false;
  const types: Record<string, string> = {
    ".html": "text/html",
    ".js": "text/javascript",
    ".mjs": "text/javascript",
    ".css": "text/css",
    ".json": "application/json",
    ".png": "image/png",
    ".svg": "image/svg+xml",
    ".woff": "font/woff",
    ".woff2": "font/woff2",
  };
  const server = createServer((request, response) => {
    const url = new URL(request.url ?? "/", "http://static.invalid");
    void (async () => {
      let bytes: Buffer,
        status = 200,
        type = "text/plain";
      try {
        assert.ok(url.pathname.startsWith("/built/"));
        const root = await realpath(directory);
        let relative = decodeURIComponent(url.pathname.slice("/built/".length));
        if (relative.endsWith("/")) relative += "index.html";
        let file = path.resolve(root, relative);
        assert.ok(file.startsWith(root + path.sep));
        if (refuseAxe && relative.endsWith("/vendor/axe-core.min.js"))
          throw new Error("Missing axe");
        try {
          file = await realpath(file);
          assert.ok(file.startsWith(root + path.sep));
          bytes = await readFile(file);
        } catch (error) {
          if (
            !url.pathname.startsWith(hostedPath) ||
            path.extname(url.pathname) ||
            !request.headers.accept?.includes("text/html")
          )
            throw error;
          file = path.join(root, "gallery/vrt/index.html");
          bytes = await readFile(file);
        }
        type = types[path.extname(file)] ?? "application/octet-stream";
      } catch {
        status = 404;
        bytes = Buffer.from("Not found");
      }
      const digest = sha256(bytes);
      if (!saved.has(digest)) {
        saved.add(digest);
        await e.save(`http-bodies/${digest}.bin`, bytes);
      }
      requests.push({
        method: request.method,
        target: request.url,
        host: request.headers.host,
        pathname: url.pathname,
        status,
        type,
        bytes: bytes.length,
        sha256: digest,
      });
      response.setHeader("Content-Type", type);
      response.setHeader("Cache-Control", "no-store");
      response.writeHead(status).end(request.method === "HEAD" ? undefined : bytes);
    })().catch((error) => response.destroy(error));
  });
  let address;
  try {
    await new Promise<void>((resolve, reject) => {
      server.once("error", reject);
      server.listen(0, "127.0.0.1", resolve);
    });
    address = server.address();
    assert.ok(address && typeof address !== "string");
  } catch (error) {
    server.closeAllConnections();
    await new Promise<void>((resolve) => server.close(() => resolve()));
    throw error;
  }
  return {
    url: `http://127.0.0.1:${address.port}${hostedPath}`,
    requests,
    refuseAxe() {
      refuseAxe = true;
    },
    close: () =>
      new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      ),
  };
}
