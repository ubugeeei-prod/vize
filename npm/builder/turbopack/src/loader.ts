import path from "node:path";
import { createHash } from "node:crypto";
import {
  compileSfc,
  collectSfcTemplateAssetUrls,
  parseSfc,
  extractSfcSrcInfo,
  generateSfcScopeId,
  rewriteSfcTemplateAssetReferences,
} from "@vizejs/native";
import { MappedModule, parseSourceMap } from "@vizejs/builder-shared/mapped-module";
import {
  createGeneratedNameAllocator,
  hmrImportSnapshot,
} from "@vizejs/builder-shared/module-output";
import { genHotReloadCode, genCSSModuleHotReloadCode } from "@vizejs/builder-shared/hot-reload";
import type { LoaderContext, SfcLoaderOptions } from "./types.ts";

function hash(value: string): string {
  return createHash("sha256").update(value).digest("hex").slice(0, 16);
}

/** Native module shape offsets use UTF-8 bytes; source-map edits use UTF-16. */
function utf16Offset(code: string, offset: number | undefined): number {
  if (offset === undefined || offset < 0 || !Number.isInteger(offset)) {
    throw new Error("[vize] Missing native component export geometry.");
  }
  const bytes = Buffer.from(code);
  if (offset > bytes.length || (offset < bytes.length && (bytes[offset] & 0xc0) === 0x80)) {
    throw new Error("[vize] Invalid native component export geometry.");
  }
  return bytes.subarray(0, offset).toString("utf8").length;
}

export function compileTurbopackSfc(filename: string, source: string, options: SfcLoaderOptions) {
  const root = options.root ?? process.cwd();
  const scopeId = generateSfcScopeId(filename, root, options.isProduction, source);
  const external = extractSfcSrcInfo(source);
  if (external.scriptSrc || external.templateSrc) {
    throw new Error(
      `[vize] ${filename}: external script/template blocks require an inline SFC for this integration.`,
    );
  }
  const descriptor = (() => {
    try {
      return parseSfc(source, { filename, includeAttrs: true });
    } catch (error) {
      throw new Error(
        `[vize] ${filename}: ${error instanceof Error ? error.message : String(error)}`,
        { cause: error },
      );
    }
  })();
  if (descriptor.template?.lang && descriptor.template.lang !== "html") {
    throw new Error(
      `[vize] ${filename}: template preprocessor ${descriptor.template.lang} is not configured for Turbopack.`,
    );
  }
  for (const script of [descriptor.script, descriptor.scriptSetup]) {
    if (script && !Array.isArray(script.attrs)) {
      throw new Error(`[vize] ${filename}: native parsed script attributes are unavailable.`);
    }
    if (script?.attrs?.some(({ name }) => name === "vapor")) {
      throw new Error(
        `[vize] ${filename}: Vapor SFC blocks are not supported by the stable Turbopack integration.`,
      );
    }
    if (script?.lang && !["js", "ts", "javascript", "typescript"].includes(script.lang)) {
      throw new Error(
        `[vize] ${filename}: script language ${script.lang} is not configured for Turbopack.`,
      );
    }
  }
  const compilerOptions = {
    ...options.compilerOptions,
    filename,
    scopeId: `data-v-${scopeId}`,
    mode: "module",
    sourceMap: options.sourceMap ?? true,
    ssr: options.ssr,
    vapor: false,
    isProd: options.isProduction,
    // Let Turbopack see JavaScript and retain the render boundary for HMR.
    isTs: false,
    inlineTemplate: false,
  };
  const compiled = compileSfc(source, compilerOptions);
  if (compiled.errors.length) throw new Error(`[vize] ${filename}:\n${compiled.errors.join("\n")}`);
  if (compiled.customBlocks.length || compiled.macroArtifacts.length) {
    throw new Error(
      `[vize] ${filename}: custom block and macro module loaders are not configured for Turbopack.`,
    );
  }
  const shape = compiled.moduleShape;
  if (!shape)
    throw new Error(
      `[vize] ${filename}: native compiler did not provide component module geometry.`,
    );
  const unique = createGeneratedNameAllocator(compiled.code);
  const component = unique("__vize_component__");
  const emitted = new MappedModule(compiled.code, parseSourceMap(compiled.map));
  if (shape.hasDefaultExport) {
    emitted.replace(
      utf16Offset(compiled.code, shape.defaultExportStart),
      utf16Offset(compiled.code, shape.defaultExportKeywordEnd),
      `const ${component} =`,
    );
  } else if (shape.hasNamedRenderExport || shape.hasNamedSsrRenderExport) {
    const render = shape.hasNamedSsrRenderExport ? "ssrRender" : "render";
    emitted.append(`\nconst ${component} = { ${render} };`);
  } else {
    throw new Error(`[vize] ${filename}: native compiler did not emit a component export.`);
  }
  if (compiled.hasScoped)
    emitted.append(`\n${component}.__scopeId = ${JSON.stringify(`data-v-${scopeId}`)};`);

  const imports: string[] = [];
  const modules: string[] = [];
  const moduleUpdates: string[] = [];
  const moduleNames = new Set<string>();
  for (const style of compiled.styles) {
    if (!style.src && !/\S/.test(style.content)) continue;
    const lang =
      style.lang ?? ((style.src ? path.extname(style.src.split("?", 1)[0]).slice(1) : "") || "css");
    if (!options.styleLanguages.includes(lang)) {
      throw new Error(
        `[vize] ${filename}: configure styleLoaders.${lang} to compile <style lang="${lang}">.`,
      );
    }
    const params = new URLSearchParams({
      type: "style",
      index: String(style.index),
      lang,
      vize: "turbopack",
      id: scopeId,
      owner: path.relative(root, filename).replace(/\\/g, "/"),
    });
    // Keep the conventional Vue query prefix for shared HMR import ownership.
    if (style.scoped) params.set("scoped", `data-v-${scopeId}`);
    if (!style.src) params.set("inline", "");
    if (!options.isProduction) params.set("dev", "");
    const stylePath = style.src ?? `./${path.basename(filename)}`;
    const requestFor = () =>
      `${stylePath}${stylePath.includes("?") ? "&" : "?"}vue&${params.toString()}`;
    const request = requestFor();
    if (style.module) {
      const name = style.moduleName ?? "$style";
      if (moduleNames.has(name))
        throw new Error(`[vize] ${filename}: duplicate CSS module name ${JSON.stringify(name)}.`);
      moduleNames.add(name);
      params.set("module", name);
      const binding = unique(`__vize_style_${style.index}__`);
      imports.push(`import ${binding} from ${JSON.stringify(requestFor())};`);
      modules.push(`${JSON.stringify(name)}: ${binding}`);
      moduleUpdates.push(
        genCSSModuleHotReloadCode(
          scopeId,
          JSON.stringify(requestFor()),
          binding,
          name,
          "import.meta.turbopackHot",
          component,
        ),
      );
    } else imports.push(`import ${JSON.stringify(request)};`);
  }
  if (modules.length) emitted.append(`\n${component}.__cssModules = { ${modules.join(", ")} };`);
  const assets = collectSfcTemplateAssetUrls(source).map((asset) => ({
    ...asset,
    varName: unique(asset.varName),
  }));
  if (assets.length) {
    emitted.edit(rewriteSfcTemplateAssetReferences(emitted.code, assets));
    for (const asset of assets) {
      const binding = unique("__vize_asset__");
      let request = asset.url.startsWith("~") ? asset.url.slice(1) : asset.url;
      request = request.split("#", 1)[0];
      imports.push(
        `import ${binding} from ${JSON.stringify(request)};\nconst ${asset.varName} = typeof ${binding} === "string" ? ${binding} : ${binding}.src;`,
      );
    }
  }
  emitted.prepend(`${imports.join("\n")}\n`);
  if (!options.isProduction)
    emitted.append(
      `\n${component}.__file = ${JSON.stringify(path.relative(root, filename).replace(/\\/g, "/"))};`,
    );
  if (!options.isProduction && !options.ssr && options.hotReload !== false) {
    const metadata = {
      source: hash(source),
      script: compiled.scriptHash,
      template: compiled.templateHash,
      options: hash(JSON.stringify(compilerOptions)),
      canRerender: true,
      // Native CSS module class maps depend on the SFC content. The host owns
      // those maps here, and the assignments above replace the native values.
      // Compare native script/template identities for that discarded metadata;
      // setup, imported values, options and style structure remain guarded by
      // the shared runtime. Missing native hashes retain the conservative path.
      module: hash(
        modules.length && compiled.scriptHash && compiled.templateHash
          ? JSON.stringify([compiled.scriptHash, compiled.templateHash])
          : emitted.code,
      ),
      styles: JSON.stringify(
        compiled.styles.map(({ content, ...style }) => ({
          ...style,
          active: Boolean(style.src || /\S/.test(content)),
        })),
      ),
    };
    // Structured generator arguments keep authored import snapshots untouched.
    emitted.append(moduleUpdates.join("\n"));
    emitted.append(
      genHotReloadCode(
        scopeId,
        metadata,
        hmrImportSnapshot(emitted.code) ?? "null",
        "import.meta.turbopackHot",
        component,
      ),
    );
  }
  emitted.append(`\nexport default ${component};`);
  const map = emitted.map;
  return {
    code: emitted.code,
    // Production scope hoisting joins input maps by their generated file. The
    // rule's `as: "*.js"` appends this suffix to the original SFC identity.
    map: map ? { ...map, file: `${filename}.js` } : null,
    warnings: compiled.warnings,
  };
}

export default function vizeLoader(this: LoaderContext<SfcLoaderOptions>, source: string): void {
  const callback = this.async();
  try {
    const result = compileTurbopackSfc(this.resourcePath, source, this.getOptions());
    for (const warning of result.warnings) this.emitWarning(new Error(`[vize] ${warning}`));
    callback(null, result.code, result.map);
  } catch (error) {
    callback(error instanceof Error ? error : new Error(String(error)));
  }
}
