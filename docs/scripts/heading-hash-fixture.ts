import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { pathToFileURL } from "node:url";
import vm from "node:vm";

const require = createRequire(new URL("../package.json", import.meta.url));
const patchFile = path.resolve(
  import.meta.dirname,
  "../../patches/@ox-content__vite-plugin@2.88.0.patch",
);
export const headingCases = [
  ["ordinary", "#ordinary"],
  ["推奨される開始点", "#%E6%8E%A8%E5%A5%A8%E3%81%95%E3%82%8C%E3%82%8B%E9%96%8B%E5%A7%8B%E7%82%B9"],
  ["100%complete", "#100%25complete"],
  ["literal%20token", "#literal%2520token"],
  ["percent%heading", "#percent%heading"],
  ["not%20decoded", "#not%20decoded"],
  ["space heading", "#space%20heading"],
  ["3:api.[x]", "#3%3Aapi.%5Bx%5D"],
  ["api:escape", "#api\\:escape"],
] as const;
export const missingHashes = ["#not-found", "#missing%ZZ", "#%E0%A4%A", "#"];

function applyPatch(directory: string, reverse = false) {
  const result = spawnSync(
    "patch",
    ["--batch", "--fuzz=0", "-p1", ...(reverse ? ["-R"] : []), "-i", patchFile],
    { cwd: directory, encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stdout + result.stderr);
}

export function verifyCoreAsset(directory: string, repaired: boolean) {
  const assets = fs
    .readdirSync(path.join(directory, "assets"))
    .filter((name) => /^ox-content-core-[a-f0-9]{10}\.js$/.test(name));
  assert.equal(assets.length, 1);
  const filename = assets[0]!;
  const source = fs.readFileSync(path.join(directory, "assets", filename), "utf8");
  const digest = createHash("sha256").update(source).digest("hex").slice(0, 10);
  assert.equal(
    filename,
    `ox-content-core-${digest}.js`,
    "native asset filename must match its actual bytes",
  );
  const html = fs.readFileSync(path.join(directory, "index.html"), "utf8");
  assert(
    html.includes(`/assets/${filename}`),
    "generated page must reference the content-hashed asset",
  );
  assert.equal(source.includes("const oxContentHashTarget ="), repaired);
  return { filename, source, digest };
}

export async function buildHeadingHashFixture(output: string) {
  const originalRoot = path.dirname(
    path.dirname(fs.realpathSync(require.resolve("@ox-content/vite-plugin"))),
  );
  assert.equal(
    JSON.parse(fs.readFileSync(path.join(originalRoot, "package.json"), "utf8"))?.version,
    "2.88.0",
  );
  const installed = fs.readFileSync(path.join(originalRoot, "dist/index.mjs"), "utf8");
  if (process.env.GITHUB_ACTIONS === "true")
    assert(
      installed.includes("function vizeRepairSsgHashTargets"),
      "Actions must install the authenticated pnpm patch",
    );
  const provider = path.join(output, "provider");
  fs.mkdirSync(provider, { recursive: true });
  fs.cpSync(path.join(originalRoot, "dist"), path.join(provider, "dist"), { recursive: true });
  fs.copyFileSync(path.join(originalRoot, "package.json"), path.join(provider, "package.json"));
  fs.symlinkSync(path.resolve(originalRoot, "../.."), path.join(provider, "node_modules"), "dir");
  if (installed.includes("function vizeRepairSsgHashTargets")) applyPatch(provider, true);
  const root = path.join(output, "fixture");
  fs.mkdirSync(path.join(root, "content"), { recursive: true });
  fs.writeFileSync(path.join(root, "index.html"), "<!doctype html><title>SSG input</title>");
  const links = headingCases
    .map(([id, hash], index) => `<a data-heading-case="${index}" href="${hash}">${id}</a>`)
    .join("\n");
  const headings = headingCases
    .map(([id]) => `<div style="height:900px"></div><h2 id="${id}">${id}</h2>`)
    .join("\n");
  fs.writeFileSync(
    path.join(root, "content/index.md"),
    `# Heading controls\n\n${links}\n${headings}\n<div style="height:1200px"></div>\n`,
  );
  const { build } = (await import(
    pathToFileURL(require.resolve("vite")).href
  )) as typeof import("vite");
  async function compile(name: string, commonJs = false, bare = false) {
    const entry = path.join(provider, "dist", commonJs ? "index.cjs" : "index.mjs");
    const plugin = commonJs
      ? (require(entry) as typeof import("@ox-content/vite-plugin"))
      : ((await import(
          pathToFileURL(entry).href + `?${name}`
        )) as typeof import("@ox-content/vite-plugin"));
    const directory = path.join(output, name);
    await build({
      root,
      logLevel: "error",
      plugins: plugin.oxContent({
        srcDir: "content",
        outDir: directory,
        ssg: { generateOgImage: false, bare },
        ogImage: false,
        docs: false,
        search: false,
        ogViewer: false,
      }),
      build: { outDir: directory },
    });
    return directory;
  }
  const before = await compile("before");
  const originalCore = verifyCoreAsset(before, false);
  applyPatch(provider);
  const esm = fs.readFileSync(path.join(provider, "dist/index.mjs"), "utf8");
  const cjs = fs.readFileSync(path.join(provider, "dist/index.cjs"), "utf8");
  const extract = (source: string) =>
    source.slice(
      source.indexOf("function vizeRepairSsgHashTargets("),
      source.indexOf("async function externalizeSharedPageAssets("),
    );
  assert.equal(extract(esm), extract(cjs), "ESM and CJS must apply the same pre-hashing repair");
  const repair = vm.runInNewContext(extract(esm) + "\nvizeRepairSsgHashTargets") as (
    html: string,
  ) => string;
  const start = "<!-- ox-content:scripts:start -->",
    end = "<!-- ox-content:scripts:end -->";
  const input = `untouched-prefix${start}<script>${originalCore.source}</script>${end}untouched-suffix`;
  assert(repair(input).startsWith("untouched-prefix"));
  assert(repair(input).endsWith("untouched-suffix"));
  for (const malformed of [
    input.replace(start, ""),
    input + start,
    input.replace(end, ""),
    input.replace(end, start + end),
    input.replace("document.querySelector(hash)", "null"),
    input.replace(
      "const scrollToHash = () => {",
      "const scrollToHash = () => {\ndocument.querySelector(hash);",
    ),
    input + "oxContentHashTarget",
  ])
    assert.throws(() => repair(malformed), /Vize docs:/);
  const after = await compile("after");
  const commonJs = await compile("commonjs", true);
  const repairedCore = verifyCoreAsset(after, true);
  assert.notEqual(repairedCore.filename, originalCore.filename);
  assert.equal(fs.existsSync(path.join(after, "assets", originalCore.filename)), false);
  assert.deepEqual(verifyCoreAsset(commonJs, true), repairedCore);
  const bare = await compile("bare", false, true);
  const bareHtml = fs.readFileSync(path.join(bare, "index.html"), "utf8");
  assert(!bareHtml.includes("ox-content-core-"), "bare output must not acquire a runtime");
  fs.appendFileSync(path.join(root, "content/index.md"), "\noxContentHashTarget\n");
  await assert.rejects(compile("refused"), /Vize docs:/);
  return {
    before,
    after,
    originalCore: originalCore.filename,
    repairedCore: repairedCore.filename,
  };
}
