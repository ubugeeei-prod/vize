import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const fromUi = createRequire(
  process.env.VIZE_TEST_VUE_PACKAGE ?? new URL("../../../npm/ui/package.json", import.meta.url),
);
const fromPlugin = process.env.VIZE_TEST_VUE_PLUGIN_PACKAGE
  ? createRequire(process.env.VIZE_TEST_VUE_PLUGIN_PACKAGE)
  : fromUi;
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
export const compiler = fromVue("@vue/compiler-sfc");
export const core = fromVue("vue");
export const renderer = fromVue("@vue/server-renderer");
const pluginFactory = (await import(pathToFileURL(fromPlugin.resolve("@vitejs/plugin-vue")).href))
  .default;
export const hash = (text: string) => createHash("sha256").update(text).digest("hex");
const dataUrl = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;

export function checkVersions() {
  for (const name of ["vue", "@vue/compiler-sfc", "@vue/compiler-ssr", "@vue/server-renderer"]) {
    assert.equal(fromVue(`${name}/package.json`).version, "3.5.35", name);
  }
  assert.equal(fromPlugin("@vitejs/plugin-vue/package.json").version, "6.0.7");
}

// The official whole-SFC transform supplies both the component and attachment.
// Its export helper and Vite SSR-context bookkeeping are retained verbatim.
export async function stockSfc(source: string, filename: string) {
  checkVersions();
  assert(filename.startsWith("/native-ssr/"), "whole-SFC oracle uses a fixed absolute root");
  const plugin = pluginFactory({ compiler, template: { compilerOptions: { comments: true } } });
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
  return { code: output.code, map: output.map, helper };
}

export const contexts = [
  {},
  {
    id: "override",
    class: ["root", { active: true }],
    style: { color: "red" },
    title: '"<&>',
    disabled: true,
    onClick() {
      throw Error("SSR event executed");
    },
  },
  {
    id: null,
    class: ["雪", "🌸"],
    style: { display: "block" },
    title: "${untrusted}\\`",
    disabled: false,
    "data-x": "猫&",
  },
];

function runtimeUrl(specifier: string) {
  const runtime = fromVue(specifier);
  const names = Object.keys(runtime).filter(
    (name) => /^[A-Za-z_$][\w$]*$/.test(name) && name !== "default",
  );
  return dataUrl(
    `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve(specifier)).href)};\n` +
      names.map((name) => `export const ${name} = runtime.${name};`).join("\n"),
  );
}
const vueUrl = runtimeUrl("vue");
const serverUrl = runtimeUrl("@vue/server-renderer");
const fromSfc = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"));
const parser = fromSfc("@babel/parser");

async function loadModule(code: string, helper?: string) {
  const parsed = parser.parse(code, { sourceType: "module" });
  const imports = parsed.program.body.filter((node: any) => node.type === "ImportDeclaration");
  let resolved = code;
  for (const node of imports.reverse()) {
    const name = node.source.value;
    const location =
      name === "vue"
        ? vueUrl
        : name === "@vue/server-renderer" || name === "vue/server-renderer"
          ? serverUrl
          : name === "\0plugin-vue:export-helper" && helper !== undefined
            ? dataUrl(helper)
            : undefined;
    assert(location, `unexpected whole-SFC runtime import ${name}`);
    resolved =
      resolved.slice(0, node.source.start) +
      JSON.stringify(location) +
      resolved.slice(node.source.end);
  }
  return import(dataUrl(resolved));
}

export async function renderModule(code: string, helper?: string) {
  const loaded = await loadModule(code, helper);
  assert(loaded.default && typeof loaded.default.ssrRender === "function");
  const executions = [];
  for (const props of contexts) {
    const app = core.createSSRApp(loaded.default, props);
    const warnings: string[] = [];
    app.config.warnHandler = (message: string) => warnings.push(message);
    const context: any = {};
    const html = await renderer.renderToString(app, context);
    assert.deepEqual(warnings, []);
    executions.push({ html, modules: [...(context.modules ?? [])] });
  }
  return executions;
}

export async function directModule(code: string, helper?: string) {
  const loaded = await loadModule(code, helper);
  return contexts.map((attrs) => {
    const forbidden = new Proxy(Object.create(null), {
      get(_, name) {
        throw Error(`static SSR read ${String(name)}`);
      },
    });
    const chunks: string[] = [];
    assert.equal(
      loaded.default.ssrRender(forbidden, (chunk: string) => chunks.push(chunk), forbidden, attrs),
      undefined,
    );
    assert(chunks.every((chunk) => typeof chunk === "string"));
    return chunks.join("");
  });
}

export function checkMap(row: any) {
  const map = row.map;
  assert.equal(map.version, 3);
  assert.equal(map.file, row.filename);
  assert.deepEqual(map.sources, [row.filename]);
  assert.deepEqual(map.sourcesContent, [row.source]);
  const fromSfc = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"));
  const codec = createRequire(fromSfc.resolve("magic-string/package.json"))(
    "@jridgewell/sourcemap-codec",
  );
  const generated = row.code.split(/\r\n|[\r\n\u2028\u2029]/);
  const original = row.source.split(/\r\n|[\r\n\u2028\u2029]/);
  const coordinate = (text: string, offset: number) => {
    const prefix = Buffer.from(text).subarray(0, offset).toString("utf8");
    assert(
      Buffer.from(prefix).equals(Buffer.from(text).subarray(0, offset)),
      "map endpoints must retain UTF-8 boundaries",
    );
    const lines = prefix.split(/\r\n|[\r\n\u2028\u2029]/);
    return [lines.length - 1, lines.at(-1)!.length];
  };
  const expectedNames: string[] = [];
  const expected = row.links
    .filter((link: any) => link.segment)
    .map((link: any) => {
      const [line, column] = coordinate(row.code, link.generated.start);
      const [sourceLine, sourceColumn] = coordinate(row.source, link.authored.start);
      const anchor = [line, column, 0, sourceLine, sourceColumn];
      if (link.name !== null) {
        let nameIndex = expectedNames.indexOf(link.name);
        if (nameIndex < 0) {
          nameIndex = expectedNames.length;
          expectedNames.push(link.name);
        }
        anchor.push(nameIndex);
      }
      return anchor;
    })
    .sort((left: number[], right: number[]) => left[0] - right[0] || left[1] - right[1]);
  assert.deepEqual(map.names, expectedNames);
  const decoded = codec.decode(map.mappings);
  assert.equal(codec.encode(decoded), map.mappings);
  const actual: number[][] = [];
  decoded.forEach((line: number[][], lineIndex: number) => {
    let previous = -1;
    for (const segment of line) {
      assert([4, 5].includes(segment.length));
      assert(segment.every(Number.isSafeInteger));
      const [column, source, sourceLine, sourceColumn, name] = segment;
      assert.equal(source, 0);
      assert(
        generated[lineIndex] !== undefined &&
          column >= previous &&
          column >= 0 &&
          column <= generated[lineIndex].length,
      );
      assert(
        original[sourceLine] !== undefined &&
          sourceColumn >= 0 &&
          sourceColumn <= original[sourceLine].length,
      );
      if (segment.length === 5) assert(name >= 0 && name < map.names.length);
      previous = column;
      actual.push([lineIndex, ...segment]);
    }
  });
  assert.deepEqual(
    actual,
    expected,
    "every actual original segment-bearing link contributes its complete start anchor",
  );
  for (const link of row.links) {
    assert.equal(typeof link.segment, "boolean");
    assert(link.name === null || typeof link.name === "string");
    for (const [text, range] of [
      [row.source, link.authored],
      [row.code, link.generated],
    ]) {
      assert(Number.isSafeInteger(range.start) && Number.isSafeInteger(range.end));
      assert(range.start >= 0 && range.start <= range.end && range.end <= Buffer.byteLength(text));
      coordinate(text, range.start);
      coordinate(text, range.end);
    }
  }
  return codec;
}
