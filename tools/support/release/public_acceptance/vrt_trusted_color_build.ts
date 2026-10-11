import assert from "node:assert/strict";
import { cp, mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Evidence } from "./vrt_artifacts.ts";
import { sha256, sides, type Side } from "./vrt_fixtures.ts";
import { hostedPath } from "./vrt_host.ts";
import { literalVariants, readLiteralVrtInputs } from "./vrt_literal.ts";
import type { LiteralBuild } from "./vrt_trusted_ui.ts";

const sourceHashes = {
  left: "ecbe89ffd4276aff872c00a258622f12cba608315f35f464392bfd33b40c088b",
  right: "5b5a1bfe6f0eea822548d2e4671fbff615842287b9ab52197c6080774d7bdd5a",
};
interface ColorManifest {
  snapshotIdentityVersion: number;
  snapshotIdentities: Record<string, string>;
  previews: Record<string, Record<string, string>>;
  arts: Array<{ path: string; metadata: { title: string }; variants: Array<{ name: string }> }>;
}
export interface TrustedColorBuild {
  literal: LiteralBuild;
  consumer: string;
  authored: Record<Side, Buffer>;
  arts: Record<Side, string>;
  manifest: ColorManifest;
  configBytes: Buffer;
}

export function changedColorSource(bytes: Buffer, green: boolean) {
  assert.equal(sha256(bytes), sourceHashes.left);
  assert.equal(bytes.toString().split("#0000ff").length, 2);
  return green ? Buffer.from(bytes.toString().replace("#0000ff", "#00ff00")) : bytes;
}

/** Exact existing same-basename inputs are additive; the original HTTP campaign is untouched. */
export async function prepareTrustedColors(build: LiteralBuild, consumer: string, e: Evidence) {
  const authored = {} as Record<Side, Buffer>,
    arts = {} as Record<Side, string>;
  for (const side of sides) {
    authored[side] = await readFile(path.join(consumer, "vrt-inputs", side, "Button.art.vue"));
    assert.equal(sha256(authored[side]), sourceHashes[side]);
    arts[side] = path.join(build.root, "src", side, "Button.art.vue");
  }
  const result: TrustedColorBuild = {
    literal: build,
    consumer,
    authored,
    arts,
    manifest: { ...build.manifest },
    configBytes: await readFile(build.config),
  };
  await rebuildTrustedColors(result, e, "public-authored-blue", false);
  return result;
}

/** Published Vite/plugins rebuild physical authored input, then delete ALL sources before return. */
export async function rebuildTrustedColors(
  input: TrustedColorBuild,
  e: Evidence,
  phase: string,
  green: boolean,
) {
  const build = input.literal;
  assert.deepEqual(await readFile(build.config), input.configBytes);
  const literal = readLiteralVrtInputs(input.consumer);
  for (const source of literal) {
    const file = path.join(build.root, "src", path.basename(source.path));
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, source.bytes, { flag: "wx" });
    await e.save(`trusted-public/${phase}/inputs/${path.basename(source.path)}`, source.bytes);
  }
  for (const side of sides) {
    assert.equal(sha256(input.authored[side]), sourceHashes[side]);
    const bytes =
      side === "left" ? changedColorSource(input.authored.left, green) : input.authored.right;
    await mkdir(path.dirname(input.arts[side]), { recursive: true });
    await writeFile(input.arts[side], bytes, { flag: "wx" });
    await e.save(`trusted-public/${phase}/inputs/${side}/Button.art.vue`, bytes);
  }
  await e.save(`trusted-public/${phase}/inputs/vite.config.mjs`, input.configBytes);
  const { build: viteBuild } = await import("vite");
  await viteBuild({
    root: build.root,
    configFile: build.config,
    logLevel: "warn",
    build: { outDir: build.directory, emptyOutDir: true },
  });
  const raw = await readFile(path.join(build.directory, "gallery/vrt/api/static.json"));
  const manifest = JSON.parse(raw.toString()) as ColorManifest;
  assert.equal(manifest.snapshotIdentityVersion, 1);
  assert.equal(manifest.arts.length, 4);
  assert.deepEqual(manifest.arts.map((art) => art.metadata.title).sort(), [
    "Controls",
    "Keys",
    "Left",
    "Right",
  ]);
  const urls = new Set<string>();
  for (const art of manifest.arts) {
    const relative = path.relative(build.root, art.path).split(path.sep).join("/");
    assert.equal(manifest.snapshotIdentities[art.path], relative);
    const title = art.metadata.title;
    const expectedPath =
      title === "Controls" || title === "Keys"
        ? path.join(build.root, "src", `${title}.art.vue`)
        : input.arts[title === "Left" ? "left" : "right"];
    assert.equal(art.path, expectedPath);
    const names =
      title === "Controls" || title === "Keys"
        ? literalVariants[title].map(([name]) => name)
        : ["Default"];
    assert.ok(
      title === "Controls" ||
        title === "Keys" ||
        sides.some((side) => input.arts[side] === art.path),
    );
    assert.deepEqual(
      art.variants.map((variant) => variant.name),
      names,
    );
    assert.equal(Object.hasOwn(manifest.previews, art.path), true);
    assert.deepEqual(Object.keys(manifest.previews[art.path]), names);
    for (const name of names) {
      assert.equal(Object.hasOwn(manifest.previews[art.path], name), true);
      const value = manifest.previews[art.path][name];
      assert.equal(typeof value, "string");
      assert.ok(value.startsWith(`${hostedPath}preview/`));
      const url = new URL(value, "https://manifest.invalid");
      assert.equal(url.origin, "https://manifest.invalid");
      assert.equal(url.search + url.hash, "");
      assert.equal(urls.has(value), false);
      urls.add(value);
      const file = path.resolve(build.directory, url.pathname.slice("/built/".length));
      assert.ok(file.startsWith(build.directory + path.sep));
      assert.equal((await stat(file)).isFile(), true);
    }
  }
  assert.equal(urls.size, 7);
  if (green) assert.deepEqual(manifest.snapshotIdentities, input.manifest.snapshotIdentities);
  input.manifest = manifest;
  // The existing literal helper keeps a literal-only typed view of the complete emitted manifest.
  build.manifest = {
    ...manifest,
    arts: manifest.arts.filter(
      (art) => art.metadata.title === "Controls" || art.metadata.title === "Keys",
    ),
  } as LiteralBuild["manifest"];
  build.raw = raw;
  await e.save(`trusted-public/${phase}/static.json`, raw);
  await cp(build.directory, path.join(e.output, "trusted-public", phase, "dist"), {
    recursive: true,
  });
  await rm(path.join(build.root, "src"), { recursive: true });
  await assert.rejects(stat(path.join(build.root, "src")), { code: "ENOENT" });
  for (const side of sides) await assert.rejects(stat(input.arts[side]), { code: "ENOENT" });
  e.records.push({
    phase,
    manifestSha256: sha256(raw),
    ownPreviewURLs: 7,
    configSha256: sha256(input.configBytes),
    authoredSourcesDeleted: true,
    colors: sides.map((side) => ({ side, originalSha256: sha256(input.authored[side]) })),
  });
}
