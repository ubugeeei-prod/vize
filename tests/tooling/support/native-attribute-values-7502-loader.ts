// Only real ImportDeclaration source literal ranges change during module loading.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const ui = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const data = (code: string) =>
  `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
let sequence = 0;
export async function environment7502(mode: "development" | "production") {
  assert.equal(process.env.NODE_ENV, mode, "fresh process selects the real runtime mode");
  console.info = (...args) => process.stderr.write(`${args.map(String).join(" ")}\n`);
  const happyDom = ui.resolve("happy-dom");
  assert.equal(
    JSON.parse(readFileSync(new URL("../package.json", pathToFileURL(happyDom)), "utf8")).version,
    "20.11.2",
  );
  const { Window } = await import(happyDom);
  const window = new Window();
  for (const key of [
    "window",
    "document",
    "Document",
    "Node",
    "Text",
    "Comment",
    "Element",
    "HTMLElement",
    "SVGElement",
    "Event",
    "ShadowRoot",
  ])
    (globalThis as any)[key] = key === "window" ? window : window[key];
  const dom = createRequire(ui.resolve("vue/package.json"));
  const vapor = createRequire(ui.resolve("vue-vapor-runtime/package.json"));
  assert.equal(dom("vue/package.json").version, "3.5.35");
  assert.equal(vapor("vue/package.json").version, "3.6.0-rc.9");
  assert.equal(dom("@vue/server-renderer/package.json").version, "3.5.35");
  const suffix = mode === "production" ? ".prod.js" : ".js";
  const domRuntime = await import(
    data(readFileSync(dom.resolve(`vue/dist/vue.runtime.esm-browser${suffix}`), "utf8"))
  );
  const vaporRuntime = await import(
    data(
      readFileSync(vapor.resolve(`vue/dist/vue.runtime-with-vapor.esm-browser${suffix}`), "utf8"),
    )
  );
  const ssrRuntime = dom("vue");
  const server = dom("vue/server-renderer");
  const vaporSsrRuntime = vapor("vue");
  const vaporServer = vapor("vue/server-renderer");
  assert.equal(vaporSsrRuntime.version, "3.6.0-rc.9");
  assert.equal(vapor("@vue/server-renderer/package.json").version, "3.6.0-rc.9");
  assert.equal(domRuntime.version, "3.5.35");
  assert.equal(vaporRuntime.version, "3.6.0-rc.9");
  assert.equal(ssrRuntime.version, "3.5.35");
  const { parse } = createRequire(dom.resolve("@vue/compiler-sfc"))("@babel/parser");
  async function load(
    code: string,
    target: string,
    helperCode: string | null = null,
    importFreeStaticSsr = false,
  ) {
    const imports = parse(code, { sourceType: "module" }).program.body.filter(
      (node: any) => node.type === "ImportDeclaration",
    );
    if (importFreeStaticSsr) assert.equal(target, "ssr");
    assert(
      imports.length > 0 || (importFreeStaticSsr && target === "ssr"),
      "whole module has actual runtime imports",
    );
    const runtime =
      target === "vapor"
        ? vaporRuntime
        : target === "vapor-ssr"
          ? vaporSsrRuntime
          : target === "ssr"
            ? ssrRuntime
            : domRuntime;
    const renderer = target === "vapor-ssr" ? vaporServer : server;
    const objectName = `__vize7502Runtime${sequence++}`;
    (globalThis as any)[objectName] = runtime;
    const serverName = `${objectName}Server`;
    (globalThis as any)[serverName] = renderer;
    const owners = new Map<string, string>();
    for (const owner of ["vue", "vue/server-renderer", "@vue/server-renderer"]) {
      const names: string[] = imports
        .filter((entry: any) => entry.source.value === owner)
        .flatMap((entry: any) =>
          entry.specifiers.map((specifier: any) => {
            assert.equal(specifier.type, "ImportSpecifier");
            const name = specifier.imported.name ?? specifier.imported.value;
            assert.equal(typeof name, "string");
            return name;
          }),
        );
      const object = owner === "vue" ? runtime : renderer;
      for (const name of names)
        assert.equal(
          typeof object[name],
          name === "Fragment" ? "symbol" : "function",
          `actual ${owner} export ${name}`,
        );
      owners.set(
        owner,
        data(
          [...new Set(names)]
            .map(
              (name) =>
                `export const ${name}=globalThis[${JSON.stringify(owner === "vue" ? objectName : serverName)}][${JSON.stringify(name)}];`,
            )
            .join("\n"),
        ),
      );
    }
    if (helperCode !== null) owners.set("\0plugin-vue:export-helper", data(helperCode));
    let resolved = code;
    for (const entry of imports.toReversed()) {
      const literal = entry.source;
      assert.equal(literal.type, "StringLiteral");
      assert(Number.isInteger(literal.start) && Number.isInteger(literal.end));
      assert(owners.has(literal.value), `unrecognized real module import ${literal.value}`);
      resolved =
        resolved.slice(0, literal.start) +
        JSON.stringify(owners.get(literal.value)) +
        resolved.slice(literal.end);
    }
    const component = (await import(data(resolved + `\n// fresh actual component ${sequence++}`)))
      .default;
    assert(component && typeof component === "object");
    assert.equal(typeof component[target.endsWith("ssr") ? "ssrRender" : "render"], "function");
    if (target.startsWith("vapor")) assert.equal(component.__vapor, true);
    return { component, runtime, renderer };
  }
  const tree = (node: any): any =>
    node.nodeType === 3 || node.nodeType === 8
      ? [node.nodeType, node.data]
      : [
          1,
          node.localName,
          node.namespaceURI,
          [...node.attributes]
            .map((attribute: any) => [attribute.name, attribute.value])
            .sort((left, right) => {
              const a = String(left);
              const b = String(right);
              return a < b ? -1 : a > b ? 1 : 0;
            }),
          [...node.childNodes].map(tree),
        ];
  const hostTree = (host: any) => [...host.childNodes].map(tree);
  const htmlTree = (html: string) => {
    const host = document.createElement("div");
    host.innerHTML = html;
    return hostTree(host);
  };
  return {
    load,
    dom,
    vapor,
    server,
    ssrRuntime,
    domRuntime,
    vaporRuntime,
    window,
    tree,
    hostTree,
    htmlTree,
  };
}
