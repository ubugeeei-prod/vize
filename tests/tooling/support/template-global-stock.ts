// Actual stock SFC compilation. Only target selection and public build configuration vary.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromPlugin = createRequire(fromUi.resolve("@vitejs/plugin-vue/package.json"));
const pluginFactory = (await import(pathToFileURL(fromUi.resolve("@vitejs/plugin-vue")).href))
  .default;
const { transformWithOxc } = await import(pathToFileURL(fromPlugin.resolve("vite")).href);
const fromBabel = createRequire(fromUi.resolve("@babel/core"));
const { parse } = fromBabel("@babel/parser");
const oraclePackage = process.env.VIZE_TEMPLATE_GLOBAL_ORACLE_PACKAGE;
assert.ok(oraclePackage);
const fromStable = createRequire(oraclePackage);
const fromVapor = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const serialized = (value: unknown): unknown => JSON.parse(JSON.stringify(value));
type Declaration = { type: string; source?: { value: string; start: number; end: number } };
export type Target = "dom" | "vapor" | "ssr";
export type VirtualModule = {
  id: string;
  loaded: { code: string; map: unknown };
  selected: { code: string; map: unknown };
  transformed: { code: string; map: unknown };
};
export type WholePacket = {
  code: string;
  errors?: unknown[];
  helperCode?: string;
  virtualModules?: VirtualModule[];
};
export type StockPacket = WholePacket & {
  source: string;
  filename: string;
  target: Target;
  inlineTemplate: boolean;
  compilerVersion: string;
  pluginVersion: string;
  map: unknown;
  helperCode: string;
  virtualModules: VirtualModule[];
  graph: Record<string, unknown>[];
};
export const declarations = (code: string): Declaration[] =>
  parse(code, { sourceType: "module", plugins: ["typescript"] }).program.body;
export const dataUrl = (code: string): string =>
  `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
export const stableProvider = fromStable;
export const vaporProvider = fromVapor;
export const pluginVersion = fromUi("@vitejs/plugin-vue/package.json").version;
assert.equal(pluginVersion, "6.0.7");

export function resolveModuleOwners(code: string, owners: ReadonlyMap<string, string>): string {
  const edits = declarations(code)
    .filter((node): node is Declaration & { source: NonNullable<Declaration["source"]> } =>
      Boolean(node.source),
    )
    .map((node) => {
      assert.ok(
        ["ImportDeclaration", "ExportNamedDeclaration", "ExportAllDeclaration"].includes(node.type),
      );
      assert.ok(owners.has(node.source.value), `unsupported module owner: ${node.source.value}`);
      return {
        start: node.source.start,
        end: node.source.end,
        text: JSON.stringify(owners.get(node.source.value)),
      };
    })
    .sort((left, right) => right.start - left.start);
  for (const edit of edits) code = code.slice(0, edit.start) + edit.text + code.slice(edit.end);
  return code;
}

export async function stockSfc(
  source: string,
  filename: string,
  target: Target,
  inlineTemplate: boolean,
): Promise<StockPacket> {
  const provider = target === "vapor" ? fromVapor : fromStable;
  const compiler = provider("vue/compiler-sfc");
  assert.equal(compiler.version, target === "vapor" ? "3.6.0-rc.9" : "3.5.41");
  const graph: Record<string, unknown>[] = [];
  const selectedCompiler = {
    ...compiler,
    parse(...args: [string, ...unknown[]]) {
      const result = compiler.parse(...args);
      assert.equal(result.descriptor.source, source);
      if (target === "vapor") result.descriptor.vapor = true;
      graph.push({
        api: "parse",
        source: args[0],
        vapor: target === "vapor",
        errors: result.errors,
      });
      return result;
    },
    compileScript(
      descriptor: unknown,
      options: { inlineTemplate: boolean; isProd?: boolean; templateOptions?: { ssr?: boolean } },
    ) {
      assert.equal(
        options.inlineTemplate,
        inlineTemplate,
        "stock plugin actually selected the requested inline branch",
      );
      const result = compiler.compileScript(descriptor, options);
      graph.push({
        api: "compileScript",
        inlineTemplate: options.inlineTemplate,
        isProd: options.isProd,
        ssr: options.templateOptions?.ssr ?? false,
        wholeCode: result.content,
        wholeMap: serialized(result.map),
        bindings: result.bindings,
        imports: result.imports,
      });
      return result;
    },
    compileTemplate(options: { source: string; ssr?: boolean }) {
      const result = compiler.compileTemplate(options);
      graph.push({
        api: "compileTemplate",
        wholeSource: options.source,
        ssr: options.ssr,
        wholeCode: result.code,
        wholeMap: result.map,
        errors: result.errors,
        tips: result.tips,
      });
      return result;
    },
  };
  const plugin = pluginFactory({
    compiler: selectedCompiler,
    template: { compilerOptions: { comments: true } },
  });
  const context = {
    error(value: string | { message: string }) {
      throw new Error(typeof value === "string" ? value : value.message);
    },
    warn(value: string | { message: string }) {
      throw new Error(typeof value === "string" ? value : value.message);
    },
    addWatchFile() {},
  };
  // In stock 6.0.7, devToolsEnabled disables inline setup; no compiler result is rewritten.
  plugin.configResolved({
    root: "/template-global-policy",
    command: "build",
    isProduction: inlineTemplate,
    build: { sourcemap: true },
    define: {},
    logger: { warn: (value: string | { message: string }) => context.warn(value) },
  });
  plugin.buildStart.call(context);
  const ssr = target === "ssr";
  const result = await plugin.transform.handler.call(context, source, filename, { ssr });
  assert.equal(typeof result?.code, "string");
  const helperCode = plugin.load.handler.call(context, "\0plugin-vue:export-helper", { ssr });
  assert.equal(typeof helperCode, "string");
  const virtualIds = [
    ...new Set(
      declarations(result.code)
        .filter((node) => node.source?.value.includes("?vue&"))
        .map((node) => {
          assert.ok(node.source);
          return node.source.value;
        }),
    ),
  ];
  const virtualModules: VirtualModule[] = [];
  for (const id of virtualIds) {
    assert.equal(plugin.resolveId.handler.call(context, id), id);
    const loaded = plugin.load.handler.call(context, id, { ssr });
    assert.equal(typeof loaded?.code, "string");
    const selected = /(?:[?&])type=template(?:&|$)/.test(id)
      ? await plugin.transform.handler.call(context, loaded.code, id, { ssr })
      : loaded;
    assert.equal(typeof selected?.code, "string");
    const transformed = await transformWithOxc(
      selected.code,
      id,
      { lang: "ts", sourcemap: true },
      selected.map,
    );
    virtualModules.push({
      id,
      loaded: { code: loaded.code, map: serialized(loaded.map) },
      selected: { code: selected.code, map: serialized(selected.map) },
      transformed: { code: transformed.code, map: serialized(transformed.map) },
    });
  }
  return {
    source,
    filename,
    target,
    inlineTemplate,
    compilerVersion: compiler.version,
    pluginVersion,
    code: result.code,
    map: serialized(result.map),
    helperCode,
    virtualModules,
    graph,
  };
}
