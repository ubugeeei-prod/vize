import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { pathToFileURL } from "node:url";
import vm from "node:vm";

const require = createRequire(new URL("../package.json", import.meta.url));
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

function removeHeadingHashRepair(directory: string) {
  for (const extension of ["mjs", "cjs"]) {
    const serverEntry = path.join(directory, "dist", `dev-server.${extension}`);
    let source = fs.readFileSync(serverEntry, "utf8");
    const repairStart = source.indexOf(
      "// Vize docs integration: repair the native core before native asset hashing.",
    );
    const repairEnd = source.indexOf("async function externalizeSharedPageAssets(", repairStart);
    assert.ok(repairStart >= 0 && repairEnd > repairStart, `${extension}: heading repair function`);
    source = source.slice(0, repairStart) + source.slice(repairEnd);
    source = replaceExactly(
      source,
      "\tconst generatePagesExternalized = void 0;",
      "\tconst generatePagesExternalized = mod.generateSsgHtmlPagesExternalized;",
      `${extension}: native pre-externalization`,
    );
    source = replaceExactly(
      source,
      "\tconst preparedPages = context.ssgOptions.bare ? generatedPages : generatedPages.map((page) => ({ ...page, html: vizeRepairSsgHashTargets(page.html) }));\n\tconst optimizedOutput = context.ssgOptions.externalizeAssets !== false ? await externalizePendingPageAssets(preparedPages, context.outDir, context.base) : {\n\t\tpages: preparedPages,",
      "\tconst optimizedOutput = context.ssgOptions.externalizeAssets !== false ? await externalizePendingPageAssets(generatedPages, context.outDir, context.base) : {\n\t\tpages: generatedPages,",
      `${extension}: heading repair call`,
    );
    fs.writeFileSync(serverEntry, source);

    const indexEntry = path.join(directory, "dist", `index.${extension}`);
    source = fs.readFileSync(indexEntry, "utf8");
    source = replaceExactly(
      source,
      '\t\t\t\tif (err instanceof Error && err.message.startsWith("Vize docs:")) throw err;\n',
      "",
      `${extension}: heading repair failure propagation`,
    );
    fs.writeFileSync(indexEntry, source);
  }
}

function replaceExactly(source: string, from: string, to: string, label: string): string {
  assert.equal(source.split(from).length, 2, label);
  return source.replace(from, to);
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
    "3.8.2",
  );
  const installed = fs.readFileSync(path.join(originalRoot, "dist/dev-server.mjs"), "utf8");
  assert(
    installed.includes("function vizeRepairSsgHashTargets"),
    "the authenticated dependency patch must be installed",
  );
  const provider = path.join(output, "provider");
  fs.mkdirSync(provider, { recursive: true });
  fs.cpSync(path.join(originalRoot, "dist"), path.join(provider, "dist"), { recursive: true });
  fs.copyFileSync(path.join(originalRoot, "package.json"), path.join(provider, "package.json"));
  fs.symlinkSync(path.resolve(originalRoot, "../.."), path.join(provider, "node_modules"), "dir");
  // Ox Content now splits its SSG implementation into dev-server.{mjs,cjs}.
  // Keep the unpatched control at a distinct module URL so Node cannot retain
  // it when the repaired build is loaded later in this qualification process.
  const controlProvider = path.join(output, "provider-control");
  fs.cpSync(provider, controlProvider, { recursive: true });
  removeHeadingHashRepair(controlProvider);
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
  async function compile(name: string, commonJs = false, bare = false, packageRoot = provider) {
    const entry = path.join(packageRoot, "dist", commonJs ? "index.cjs" : "index.mjs");
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
  const before = await compile("before", false, false, controlProvider);
  const originalCore = verifyCoreAsset(before, false);
  const esm = fs.readFileSync(path.join(provider, "dist/dev-server.mjs"), "utf8");
  const cjs = fs.readFileSync(path.join(provider, "dist/dev-server.cjs"), "utf8");
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
  const outsideMention = input.replace(
    "untouched-prefix",
    "ordinary oxContentHashTarget paragraph",
  );
  assert(repair(outsideMention).startsWith("ordinary oxContentHashTarget paragraph"));
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
    input.replace(
      "const scrollToHash = () => {",
      "const oxContentHashTarget = () => null;\nconst scrollToHash = () => {",
    ),
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
  fs.appendFileSync(
    path.join(root, "content/index.md"),
    "\nOrdinary oxContentHashTarget paragraph.\n",
  );
  const mentioned = await compile("ordinary-mention");
  assert.deepEqual(verifyCoreAsset(mentioned, true), repairedCore);
  fs.appendFileSync(path.join(root, "content/index.md"), "\n<!-- ox-content:scripts:start -->\n");
  await assert.rejects(compile("refused"), /Vize docs:/);
  return {
    before,
    after,
    originalCore: originalCore.filename,
    repairedCore: repairedCore.filename,
  };
}
