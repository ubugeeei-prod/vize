import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { checkVersions, compiler } from "./native-sfc-ssr-reference.ts";

const fromUi = createRequire(
  process.env.VIZE_TEST_VUE_PACKAGE ?? new URL("../../../npm/ui/package.json", import.meta.url),
);
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const fromPlugin = process.env.VIZE_TEST_VUE_PLUGIN_PACKAGE
  ? createRequire(process.env.VIZE_TEST_VUE_PLUGIN_PACKAGE)
  : fromUi;
const pluginFactory = (await import(pathToFileURL(fromPlugin.resolve("@vitejs/plugin-vue")).href))
  .default;
const vite = await import(pathToFileURL(fromPlugin.resolve("vite")).href);
const fromSfc = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"));
const parser = fromSfc("@babel/parser");
const codec = createRequire(fromSfc.resolve("magic-string/package.json"))(
  "@jridgewell/sourcemap-codec",
);
const dataUrl = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
const declarations = (code: string) =>
  parser
    .parse(code, { sourceType: "module" })
    .program.body.filter(
      (node: any) =>
        node.type === "ImportDeclaration" ||
        node.type === "ExportAllDeclaration" ||
        (node.type === "ExportNamedDeclaration" && node.source),
    );
export type SetupGraph = {
  code: string;
  map?: any;
  helper?: string;
  dependencies?: any[];
};

export function primarySetupBindings(source: string, filename: string) {
  const parsed = compiler.parse(source, { filename });
  assert.deepEqual(parsed.errors, []);
  const script = compiler.compileScript(parsed.descriptor, { id: filename });
  const bindings = script.scriptSetupAst
    .filter((node: any) => node.type === "VariableDeclaration")
    .flatMap((node: any) =>
      node.declarations.map((declaration: any) => {
        assert.equal(declaration.id.type, "Identifier");
        return { name: declaration.id.name, kind: node.kind[0].toUpperCase() + node.kind.slice(1) };
      }),
    );
  assert.deepEqual(
    bindings.map((row: any) => row.name),
    Object.keys(script.bindings),
  );
  return bindings;
}

// Keep the real plugin's whole module, export helper, context hook and TS graph.
export async function stockSetupSfc(source: string, filename: string, external: boolean) {
  checkVersions();
  assert(filename.startsWith("/native-ssr/"));
  const plugin = pluginFactory({
    compiler,
    features: { prodDevtools: external },
    template: { compilerOptions: { comments: true } },
  });
  plugin.configResolved({
    root: "/native-ssr",
    command: "build",
    isProduction: true,
    build: { sourcemap: true },
    define: {},
    logger: {
      warn(message: string) {
        throw Error(message);
      },
    },
  });
  plugin.buildStart.call({});
  const context = {
    error(error: any) {
      throw Error(typeof error === "string" ? error : error.message);
    },
    warn(error: any) {
      throw Error(typeof error === "string" ? error : error.message);
    },
    addWatchFile() {},
  };
  const output = await plugin.transform.handler.call(context, source, filename, { ssr: true });
  assert(output && typeof output.code === "string");
  const helper = plugin.load.handler.call(context, "\0plugin-vue:export-helper", { ssr: true });
  assert.equal(typeof helper, "string");
  const dependencies: any[] = [];
  const visit = async (code: string) => {
    for (const node of declarations(code)) {
      const id = node.source.value;
      if (!id.startsWith("/native-ssr/")) continue;
      assert.equal(id, `${filename}?vue&type=script&setup=true&lang.ts`);
      if (dependencies.some((row) => row.id === id)) continue;
      const original = plugin.load.handler.call(context, id, { ssr: true });
      assert(original && typeof original.code === "string");
      const transformed = await vite.transformWithOxc(
        original.code,
        id,
        { lang: "ts", sourcemap: true },
        original.map,
      );
      dependencies.push({
        id,
        originalCode: original.code,
        originalMap: original.map,
        code: transformed.code,
        map: transformed.map,
      });
      await visit(transformed.code);
    }
  };
  await visit(output.code);
  return { code: output.code, map: output.map, helper, dependencies };
}

function runtimeUrl(name: string) {
  const runtime = fromVue(name);
  return dataUrl(
    `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve(name)).href)};\n` +
      Object.keys(runtime)
        .filter((key) => /^[A-Za-z_$][\w$]*$/.test(key) && key !== "default")
        .map((key) => `export const ${key} = runtime.${key};`)
        .join("\n"),
  );
}
const vueUrl = runtimeUrl("vue"),
  serverUrl = runtimeUrl("@vue/server-renderer");
let loadIndex = 0;
export async function loadSetupModule(graph: SetupGraph) {
  const isolation = loadIndex++;
  const locations = new Map<string, string>();
  const rewrite = async (code: string): Promise<string> => {
    let output = code;
    for (const node of declarations(code).reverse()) {
      const name = node.source.value;
      let location =
        name === "vue"
          ? vueUrl
          : name === "@vue/server-renderer" || name === "vue/server-renderer"
            ? serverUrl
            : name === "\0plugin-vue:export-helper" && graph.helper !== undefined
              ? dataUrl(graph.helper)
              : locations.get(name);
      if (!location) {
        const dependency = graph.dependencies?.find((row) => row.id === name);
        assert(dependency, `unexpected whole setup SFC import ${name}`);
        location = dataUrl(
          `${await rewrite(dependency.code)}\n// isolated setup dependency ${isolation}\n`,
        );
        locations.set(name, location);
      }
      // Only parser-owned module-specifier ranges change. Authored strings stay intact.
      output =
        output.slice(0, node.source.start) +
        JSON.stringify(location) +
        output.slice(node.source.end);
    }
    return output;
  };
  const loaded = await import(
    dataUrl(`${await rewrite(graph.code)}\n// isolated setup oracle ${isolation}\n`)
  );
  assert(loaded.default && typeof loaded.default.setup === "function");
  return loaded.default;
}

export function removeSetupMarker(code: string) {
  const ranges: [number, number][] = [];
  const visit = (node: any) => {
    if (!node || typeof node !== "object") return;
    if (
      node.type === "ExpressionStatement" &&
      node.expression.type === "CallExpression" &&
      node.expression.callee.type === "MemberExpression" &&
      node.expression.callee.object.name === "Object" &&
      node.expression.callee.property.name === "defineProperty" &&
      node.expression.arguments[1]?.value === "__isScriptSetup"
    )
      ranges.push([node.start, node.end]);
    for (const value of Object.values(node)) {
      if (Array.isArray(value)) value.forEach(visit);
      else if (value && typeof value === "object") visit(value);
    }
  };
  visit(parser.parse(code, { sourceType: "module" }).program);
  assert.equal(ranges.length, 1);
  const [start, end] = ranges[0];
  return code.slice(0, start) + code.slice(end);
}

// Upstream maps have their actual shape; inline TS may genuinely have only mappings:"".
export function checkOfficialSetupMap(
  graph: SetupGraph,
  source: string,
  filename: string,
  external = false,
) {
  const check = (code: string, map: any, allowEmpty = false) => {
    if (allowEmpty && JSON.stringify(map) === '{"mappings":""}') return;
    assert.equal(map.version, 3);
    assert.deepEqual(map.sources, [filename]);
    assert.deepEqual(map.sourcesContent, [source]);
    // Real compiler-sfc/MagicString maps locate lines with LF only. Preserve
    // that upstream convention; native SpanLink maps use their separate judge.
    const generated = code.split("\n");
    const original = source.split("\n");
    const decoded = codec.decode(map.mappings);
    assert.equal(codec.encode(decoded), map.mappings);
    decoded.forEach((line: number[][], lineIndex: number) => {
      let previous = -1;
      for (const segment of line) {
        assert([1, 4, 5].includes(segment.length) && segment.every(Number.isSafeInteger));
        const [column, sourceIndex, sourceLine, sourceColumn, name] = segment;
        assert(
          generated[lineIndex] !== undefined &&
            column >= previous &&
            column >= 0 &&
            column <= generated[lineIndex].length,
        );
        previous = column;
        if (segment.length === 1) continue;
        assert.equal(sourceIndex, 0);
        assert(
          original[sourceLine] !== undefined &&
            sourceColumn >= 0 &&
            sourceColumn <= original[sourceLine].length,
        );
        if (segment.length === 5) assert(name >= 0 && name < map.names.length);
      }
    });
  };
  check(graph.code, graph.map, !external && (graph.dependencies?.length ?? 0) > 0);
  for (const row of graph.dependencies ?? []) {
    assert.equal(row.id, `${filename}?vue&type=script&setup=true&lang.ts`);
    check(row.originalCode, row.originalMap);
    check(row.code, row.map);
  }
}

export function insertSetupRead(code: string, name: string) {
  const entries = parser
    .parse(code, { sourceType: "module" })
    .program.body.filter(
      (node: any) => node.type === "FunctionDeclaration" && node.params[5]?.name === "$setup",
    );
  assert.equal(entries.length, 1);
  const offset = entries[0].body.start + 1;
  return code.slice(0, offset) + `$setup[${JSON.stringify(name)}];` + code.slice(offset);
}

export function insertConstSetter(code: string, name: string) {
  const matches: any[] = [];
  const visit = (node: any) => {
    if (!node || typeof node !== "object") return;
    if (node.type === "ObjectMethod" && node.kind === "get" && node.key.name === name)
      matches.push(node);
    for (const value of Object.values(node)) {
      if (Array.isArray(value)) value.forEach(visit);
      else if (value && typeof value === "object") visit(value);
    }
  };
  visit(parser.parse(code, { sourceType: "module" }).program);
  assert.equal(matches.length, 1);
  const offset = matches[0].end;
  return code.slice(0, offset) + `, set ${name}(__value) {}` + code.slice(offset);
}
