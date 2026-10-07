// Capture independent official compiler evidence; never write to the source fixture.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdir, readFile, readdir, realpath, writeFile } from "node:fs/promises";
import { basename, isAbsolute, relative, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const compilerIdentity = {
  package: "@vue/compiler-sfc",
  version: "3.5.26",
  entrypoint: "dist/compiler-sfc.esm-browser.js",
  url: "https://unpkg.com/@vue/compiler-sfc@3.5.26/dist/compiler-sfc.esm-browser.js",
  sha256: "4bd5cbcf9bdae4e4264be4ddb14e416074ad96909c56ba9b604e93d5b76b06ec",
};
const licensePins = {
  "LICENSE.md": "d2f621f59aa4c10eab79b6333e59d9d3d5b53307dcfd16dbd75e40e679e84965",
  "LICENSE_EE.md": "6110c69fd3b92928328a89e863dc2c862ea449f2335c352a9c4a137a36b13040",
};
const [sourceArgument, outputArgument, bundleArgument, ...extra] = process.argv.slice(2);
if (!sourceArgument || !outputArgument || !bundleArgument || extra.length) {
  throw new Error("usage: n8n-official-compiler-custody.mjs <n8n-root> <outdir> <bundle>");
}
const sourceRoot = await realpath(sourceArgument);
const outputRoot = await resolvedOutput(outputArgument);
if (inside(sourceRoot, outputRoot)) throw new Error("outdir must be outside read-only n8n");
await mkdir(outputRoot, { recursive: true });
if ((await readdir(outputRoot)).length) throw new Error("outdir must be empty");
const manifestPath =
  process.env.VIZE_N8N_CASES_MANIFEST ??
  fileURLToPath(
    new URL(
      "../../../tests/_fixtures/differential/compiler/n8n-adoption/cases.json",
      import.meta.url,
    ),
  );
const manifestBytes = await readFile(manifestPath);
const manifest = JSON.parse(manifestBytes.toString("utf8"));
if (
  manifest.repository !== "https://github.com/n8n-io/n8n" ||
  manifest.revision !== "e882e8a483f433facb47bab9b407d0ec00a81172" ||
  manifest.cases.length !== 10 ||
  new Set(manifest.cases.map((entry) => entry.path)).size !== 10
)
  throw new Error("unexpected original-source custody manifest");
const revision = execFileSync("git", ["-C", sourceRoot, "rev-parse", "HEAD"], {
  encoding: "utf8",
}).trim();
if (revision !== manifest.revision) throw new Error("n8n revision does not match manifest");
const bundleBytes = await readFile(bundleArgument);
verifyHash(bundleBytes, compilerIdentity.sha256, "official compiler bundle");
const compilerPath = resolve(outputRoot, "compiler/compiler-sfc.esm-browser.mjs");
await save(compilerPath, bundleBytes);
const compiler = await import(pathToFileURL(compilerPath).href);
if (compiler.version !== compilerIdentity.version)
  throw new Error("official compiler version mismatch");
await save(resolve(outputRoot, "cases.json"), manifestBytes);
const licenses = [];
for (const [name, hash] of Object.entries(licensePins)) {
  const bytes = await readFile(resolve(sourceRoot, name));
  verifyHash(bytes, hash, name);
  await save(resolve(outputRoot, "licenses", name), bytes);
  licenses.push({ path: name, sha256: hash, bytes: bytes.length });
}

const modes = [
  { name: "function-no-prefix", mode: "function", prefixIdentifiers: false },
  { name: "module-prefix", mode: "module", prefixIdentifiers: true },
];
const records = [];
for (const [index, entry] of manifest.cases.entries()) {
  const filename = await realpath(resolve(sourceRoot, entry.path));
  if (!inside(sourceRoot, filename)) throw new Error(`case escapes source root: ${entry.path}`);
  const bytes = await readFile(filename);
  verifyHash(bytes, entry.sha256, entry.path);
  const source = bytes.toString("utf8");
  const directory = `cases/${String(index + 1).padStart(2, "0")}-${basename(entry.path, ".vue")}`;
  await save(resolve(outputRoot, directory, "original.vue"), bytes);
  const id = `data-v-${entry.sha256.slice(0, 8)}`;
  const parsed = compiler.parse(source, { filename, sourceMap: true, ignoreEmpty: false });
  await json(resolve(outputRoot, directory, "parse.json"), parsed);
  const descriptor = parsed.descriptor;
  const isTS = ["ts", "tsx"].includes(descriptor.scriptSetup?.lang ?? descriptor.script?.lang);
  const scriptExtension = isTS ? "ts" : "js";
  const record = {
    ...entry,
    bytes: bytes.length,
    directory,
    parseErrors: parsed.errors,
    templates: [],
    scripts: [],
    componentModules: [],
    styles: [],
  };
  if (!descriptor.template) throw new Error(`original has no template: ${entry.path}`);
  await save(resolve(outputRoot, directory, "template.txt"), descriptor.template.content);

  for (const sourceMap of [false, true]) {
    for (const mode of modes) {
      const compilerOptions = options(mode);
      const recipe = {
        filename,
        id,
        sourceMap,
        scoped: mode.mode === "module" && descriptor.styles.some((style) => style.scoped),
        compilerOptions,
        inMap: sourceMap ? descriptor.template.map : undefined,
      };
      const capture = attempt(() =>
        compiler.compileTemplate({ source: descriptor.template.content, ...recipe }),
      );
      const name = `template-${mode.name}-map-${sourceMap}`;
      await json(resolve(outputRoot, directory, `${name}.json`), { recipe, ...capture });
      if (capture.status === "returned")
        await save(resolve(outputRoot, directory, `${name}.js`), capture.result.code);
      record.templates.push(summary(name, capture));
    }

    const scriptOptions = { id, sourceMap, inlineTemplate: false, isProd: false };
    const script = attempt(() => compiler.compileScript(descriptor, scriptOptions));
    await json(resolve(outputRoot, directory, `script-map-${sourceMap}.json`), {
      recipe: scriptOptions,
      ...script,
    });
    record.scripts.push(summary(`script-map-${sourceMap}`, script));
    if (script.status === "returned") {
      await save(
        resolve(outputRoot, directory, `script-map-${sourceMap}.${scriptExtension}`),
        script.result.content,
      );
      for (const mode of modes) {
        const compilerOptions = {
          ...options(mode),
          isTS,
          bindingMetadata: script.result.bindings,
        };
        const recipe = {
          filename,
          id,
          sourceMap,
          scoped: mode.mode === "module" && descriptor.styles.some((style) => style.scoped),
          compilerOptions,
          inMap: sourceMap ? descriptor.template.map : undefined,
        };
        const bound = attempt(() =>
          compiler.compileTemplate({ source: descriptor.template.content, ...recipe }),
        );
        const name = `template-real-bindings-${mode.name}-map-${sourceMap}`;
        await json(resolve(outputRoot, directory, `${name}.json`), { recipe, ...bound });
        if (bound.status === "returned")
          await save(
            resolve(outputRoot, directory, `${name}.${scriptExtension}`),
            bound.result.code,
          );
        record.templates.push(summary(name, bound));
      }
    }

    const inlineOptions = {
      id,
      sourceMap,
      inlineTemplate: true,
      isProd: false,
      templateOptions: { sourceMap, compilerOptions: options(modes[1]) },
    };
    const component = attempt(() => compiler.compileScript(descriptor, inlineOptions));
    const name = `component-inline-module-map-${sourceMap}`;
    await json(resolve(outputRoot, directory, `${name}.json`), {
      recipe: inlineOptions,
      templateInlined: Boolean(descriptor.scriptSetup),
      ...component,
    });
    if (component.status === "returned")
      await save(
        resolve(outputRoot, directory, `${name}.${scriptExtension}`),
        component.result.content,
      );
    record.componentModules.push(summary(name, component));

    for (const [styleIndex, style] of descriptor.styles.entries()) {
      const recipe = {
        filename,
        id,
        scoped: Boolean(style.scoped),
        modules: Boolean(style.module),
        preprocessLang: style.lang,
        inMap: sourceMap ? style.map : undefined,
        postcssOptions: { map: sourceMap ? { inline: false, annotation: false } : false },
      };
      const capture = attempt(() => compiler.compileStyle({ source: style.content, ...recipe }));
      const name = `style-${styleIndex}-map-${sourceMap}`;
      await json(resolve(outputRoot, directory, `${name}.json`), { recipe, ...capture });
      record.styles.push(summary(name, capture));
    }
  }
  records.push(record);
}
await json(resolve(outputRoot, "capture.json"), {
  schemaVersion: 1,
  compiler: compilerIdentity,
  nodeVersion: process.version,
  repository: manifest.repository,
  revision,
  sourceRoot,
  manifestSha256: sha256(manifestBytes),
  licenses,
  records,
  scope:
    "Compile evidence from complete unchanged originals; no runtime replay or Vite/plugin assembly is claimed. Inline component modules are the official compileScript inlineTemplate result; style results are separate. External type/preprocessor refusals are preserved, never replaced with invented binding metadata. Object graph references use JSON Pointer $ref; bigint values use $bigint.",
});
console.log(
  JSON.stringify({ outputRoot, compiler: compilerIdentity, revision, cases: records.length }),
);

function inside(root, target) {
  const path = relative(root, target);
  return path === "" || (!isAbsolute(path) && path !== ".." && !path.startsWith(`..${sep}`));
}

async function resolvedOutput(path) {
  let parent = resolve(path);
  const suffix = [];
  for (;;) {
    try {
      return resolve(await realpath(parent), ...suffix);
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
      suffix.unshift(basename(parent));
      parent = resolve(parent, "..");
    }
  }
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function verifyHash(bytes, expected, name) {
  if (sha256(bytes) !== expected) throw new Error(`SHA256 mismatch: ${name}`);
}

function options(mode) {
  return {
    mode: mode.mode,
    prefixIdentifiers: mode.prefixIdentifiers,
    cacheHandlers: false,
    hoistStatic: true,
    comments: false,
  };
}

function attempt(run) {
  const warnings = [];
  const originalWarn = console.warn;
  console.warn = (...args) => warnings.push(args.map(String).join(" "));
  try {
    return { status: "returned", warnings, result: run() };
  } catch (error) {
    return { status: "threw", warnings, error };
  } finally {
    console.warn = originalWarn;
  }
}

function summary(name, capture) {
  return {
    name,
    status: capture.status,
    warnings: capture.warnings,
    errors: capture.result?.errors ?? (capture.error ? [capture.error] : []),
  };
}

async function save(path, bytes) {
  await mkdir(resolve(path, ".."), { recursive: true });
  await writeFile(path, bytes);
}

async function json(path, value) {
  await save(path, `${JSON.stringify(value, graphReplacer(), 2)}\n`);
}

function graphReplacer() {
  const paths = new WeakMap();
  return function (key, value) {
    if (typeof value === "bigint") return { $bigint: String(value) };
    if (value === null || typeof value !== "object") return value;
    if (paths.has(value)) return { $ref: paths.get(value) };
    const escaped = key.replaceAll("~", "~0").replaceAll("/", "~1");
    const pointer = key === "" ? "#" : `${paths.get(this) ?? "#"}/${escaped}`;
    paths.set(value, pointer);
    let result = value;
    if (value instanceof Error)
      result = {
        name: value.name,
        ...Object.fromEntries(Object.getOwnPropertyNames(value).map((name) => [name, value[name]])),
      };
    if (value instanceof Map) result = { $map: [...value] };
    if (value instanceof Set) result = { $set: [...value] };
    paths.set(result, pointer);
    return result;
  };
}
