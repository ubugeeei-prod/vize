// Whole original #7970 SFCs and real default-slot DOM; existing pinned Vue only.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { Window } from "happy-dom";

const root = new URL("../../../", import.meta.url);
const corpus = new URL("tests/_fixtures/differential/compiler/component-slot-text-7970/", root);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const custody = JSON.parse(readFileSync(new URL("custody.json", corpus)));
for (const pin of [...custody.inputs, ...custody.authoredControls])
  assert.equal(hash(readFileSync(new URL(pin.path, corpus))), pin.sha256, pin.path);
const expected = JSON.parse(readFileSync(new URL("runtime.expected.json", corpus)));
const sourceCases = JSON.parse(readFileSync(new URL("cases.json", corpus)));
const originalApp = readFileSync(new URL("App.vue.txt", corpus), "utf8");
const originalChild = readFileSync(new URL("MyTitle.vue.txt", corpus), "utf8");
const evidence = {
  sourceRevision: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root }).toString().trim(),
  versions: null,
  cases: [],
  sfcs: [],
  qualifiedCases: 0,
  qualifiedOriginalSfcs: 0,
  nativeHandled: 0,
};
if (process.env.GITHUB_SHA) assert.equal(evidence.sourceRevision, process.env.GITHUB_SHA);
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.equal(input.cases.length, 64);
assert.equal(input.sfcs.length, 4);
const window = new Window();
for (const name of [
  "window",
  "document",
  "Document",
  "Node",
  "Text",
  "Comment",
  "Element",
  "HTMLElement",
  "SVGElement",
])
  globalThis[name] = name === "window" ? window : window[name];
const fromUi = createRequire(new URL("npm/ui/package.json", root));
const compiler = fromUi("vue/compiler-sfc");
const fromSfc = createRequire(fromUi.resolve("@vue/compiler-sfc/package.json"));
const codec = createRequire(fromSfc.resolve("magic-string/package.json"))(
  "@jridgewell/sourcemap-codec",
);
const vue = fromUi("vue");
const { transformSync } = fromUi("@babel/core");
assert.equal(vue.version, "3.5.35");
assert.equal(compiler.version, vue.version);
assert.equal(process.env.NODE_ENV, "production");
evidence.versions = {
  node: process.version,
  vue: vue.version,
  compiler: compiler.version,
  compilerManifestSha256: hash(readFileSync(fromUi.resolve("@vue/compiler-sfc/package.json"))),
  runtimeManifestSha256: hash(readFileSync(fromUi.resolve("vue/package.json"))),
  reportedVue: custody.reportedVersions.vue,
  reportedVueExecution: "unexecuted",
};
let sequence = 0;
function url(code) {
  return `data:text/javascript;base64,${Buffer.from(code + `\n// ${sequence++}`).toString("base64")}`;
}
async function load(code, child = null, module = false) {
  const registry = { vue, "./MyTitle.vue": { default: child } };
  globalThis.__componentSlotTextModules = registry;
  const transformed = transformSync(code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            const source = path.node.source.value;
            assert(Object.hasOwn(registry, source), source);
            const declarations = path.node.specifiers.map((specifier) => {
              const imported = t.isImportSpecifier(specifier)
                ? specifier.imported.name
                : t.isImportDefaultSpecifier(specifier)
                  ? "default"
                  : null;
              assert(
                imported && Object.hasOwn(registry[source], imported),
                `${source}:${imported}`,
              );
              return t.variableDeclarator(
                t.identifier(specifier.local.name),
                t.memberExpression(
                  t.memberExpression(
                    t.memberExpression(
                      t.identifier("globalThis"),
                      t.identifier("__componentSlotTextModules"),
                    ),
                    t.stringLiteral(source),
                    true,
                  ),
                  t.stringLiteral(imported),
                  true,
                ),
              );
            });
            path.replaceWith(t.variableDeclaration("const", declarations));
          },
        },
      }),
    ],
  }).code;
  const result = await import(url(transformed));
  return module ? result.default : result.render;
}
function stockTemplate(source, row, bindings) {
  const result = compiler.compileTemplate({
    source,
    filename: "App.vue",
    id: "slot-text",
    isProd: true,
    compilerOptions: {
      comments: row.comments,
      whitespace: row.whitespace,
      hoistStatic: false,
      bindingMetadata: bindings,
    },
  });
  assert.deepEqual(result.errors, []);
  assert.deepEqual(result.tips, []);
  return result;
}
function stockSfc(source, row, filename) {
  const parsed = compiler.parse(source, { filename });
  assert.deepEqual(parsed.errors, []);
  assert.equal(parsed.descriptor.source, source);
  const descriptor = parsed.descriptor;
  let code = "const __component = {};";
  let script = null;
  if (descriptor.scriptSetup || descriptor.script) {
    script = compiler.compileScript(descriptor, { id: "slot-text", genDefaultAs: "__component" });
    code = script.content;
  }
  const template = stockTemplate(descriptor.template.content, row, script?.bindings);
  code += `\n${template.code}\n__component.render = render;\nexport default __component;\n`;
  return { code, script, template };
}
function children(parent) {
  return [...parent.childNodes].map((node) =>
    node.nodeType === 3 || node.nodeType === 8
      ? { type: node.nodeType, data: node.data }
      : {
          type: node.nodeType,
          tag: node.localName,
          namespace: node.namespaceURI,
          attributes: [...node.attributes].map((attr) => [attr.name, attr.value]),
          children: children(node),
        },
  );
}
function mapGraph(code, map, source) {
  assert.equal(typeof map, "string");
  const value = JSON.parse(map);
  assert.equal(value.version, 3);
  assert.deepEqual(
    value.sourcesContent,
    value.sources.map(() => source),
  );
  const graph = codec.decode(value.mappings);
  assert.equal(codec.encode(graph), value.mappings);
  const generated = code.split(/\r\n|[\r\n\u2028\u2029]/u);
  const original = source.split(/\r\n|[\r\n\u2028\u2029]/u);
  let mapped = 0;
  for (const [line, segments] of graph.entries()) {
    for (const segment of segments) {
      assert([1, 4, 5].includes(segment.length));
      assert(segment.every(Number.isSafeInteger));
      assert(segment[0] >= 0 && segment[0] <= generated[line].length);
      if (segment.length === 1) continue;
      assert(segment[1] >= 0 && segment[1] < value.sources.length);
      assert(segment[2] >= 0 && segment[2] < original.length);
      assert(segment[3] >= 0 && segment[3] <= original[segment[2]].length);
      if (segment.length === 5) assert(segment[4] >= 0 && segment[4] < value.names.length);
      mapped++;
    }
  }
  assert(mapped > 0);
  return { raw: value, graph };
}
async function observe(component, child, updates) {
  const app = vue.createApp(component);
  app.component("MyTitle", child);
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  const host = document.createElement("div");
  document.body.append(host);
  const snapshots = [];
  let mounted = false;
  try {
    const instance = app.mount(host);
    mounted = true;
    for (let index = 0; index <= updates.length; index++) {
      if (index) Object.assign(instance, updates[index - 1]);
      await vue.nextTick();
      snapshots.push({
        html: host.innerHTML,
        children: children(host),
        titles: [...host.querySelectorAll(".my-title")].map((el) =>
          [...el.childNodes].map((n) =>
            n.nodeType === 3 ? n.data : `<${n.nodeName.toLowerCase()}>`,
          ),
        ),
        diagnostics: [...diagnostics],
      });
      assert.deepEqual(diagnostics, []);
    }
    app.unmount();
    mounted = false;
    await vue.nextTick();
    assert.deepEqual(children(host), []);
    return { snapshots, afterUnmount: [], diagnostics };
  } finally {
    if (mounted) app.unmount();
    host.remove();
  }
}
try {
  const seen = new Set();
  for (const row of input.cases) {
    const key = `${row.whitespace}/${row.comments}/${row.name}`;
    assert(!seen.has(key));
    seen.add(key);
    const fixture = sourceCases.find((entry) => entry.name === row.name);
    assert.equal(row.source, fixture.template);
    assert.deepEqual(row.current.errors, []);
    assert.equal(row.current.map, null);
    assert.equal(row.current.code, row.retained.code);
    const graph = mapGraph(row.retained.code, row.retained.map, row.source);
    const official = stockTemplate(row.source, row);
    const childSource =
      '<template><h2 class="my-title"><slot :label="\'owned\'"></slot><slot name="header" :label="\'owned\'"></slot></h2></template>';
    const child = await load(stockSfc(childSource, row, "MyTitle.vue").code, null, true);
    const actualRender = await load(row.current.code);
    const officialRender = await load(official.code);
    const component = (render) => ({ render, data: () => ({ ...expected.initial }) });
    const actual = await observe(component(actualRender), child, expected.updates);
    const reference = await observe(component(officialRender), child, expected.updates);
    const packet = { ...row, official, map: graph, actual, reference };
    evidence.cases.push(packet);
    assert.deepEqual(actual, reference, key);
    evidence.qualifiedCases++;
  }
  assert.equal(seen.size, 64);
  for (const row of input.sfcs) {
    assert.equal(row.appSource, originalApp);
    assert.equal(row.childSource, originalChild);
    for (const current of [row.app, row.child]) {
      assert.deepEqual(Object.keys(current).sort(), [
        "bindings",
        "code",
        "css",
        "errors",
        "macroArtifacts",
        "map",
        "warnings",
      ]);
      assert.deepEqual(current.errors, []);
      assert.deepEqual(current.warnings, []);
      assert.equal(current.css, null);
      assert.equal(current.map, null);
      assert.deepEqual(current.macroArtifacts, []);
    }
    const officialChild = stockSfc(originalChild, row, "MyTitle.vue");
    const officialApp = stockSfc(originalApp, row, "App.vue");
    const actualChild = await load(row.child.code, null, true);
    const referenceChild = await load(officialChild.code, null, true);
    const actual = await observe(await load(row.app.code, actualChild, true), actualChild, []);
    const reference = await observe(
      await load(officialApp.code, referenceChild, true),
      referenceChild,
      [],
    );
    evidence.sfcs.push({ ...row, officialApp, officialChild, actual, reference });
    assert.deepEqual(actual, reference);
    assert.deepEqual(actual.snapshots[0].titles, expected.reportedDom);
    if (row.whitespace === "condense")
      assert.equal(actual.snapshots[0].html, expected.reportedHtml);
    evidence.qualifiedOriginalSfcs++;
  }
  assert.equal(evidence.qualifiedCases, 64);
  assert.equal(evidence.qualifiedOriginalSfcs, 4);
} catch (error) {
  evidence.failure = String(error.stack ?? error);
  process.exitCode = 1;
} finally {
  const profile = process.env.NEXTEST_PROFILE ?? "pr";
  assert(["pr", "full"].includes(profile));
  const directory = new URL(`target/nextest/${profile}/component-slot-text-7970/`, root);
  mkdirSync(directory, { recursive: true });
  writeFileSync(new URL("observations.json", directory), JSON.stringify(evidence, null, 2) + "\n");
  process.stdout.write(
    JSON.stringify({
      qualifiedCases: evidence.qualifiedCases,
      qualifiedOriginalSfcs: evidence.qualifiedOriginalSfcs,
      nativeHandled: evidence.nativeHandled,
      evidenceSha256: hash(JSON.stringify(evidence)),
      failure: evidence.failure ?? null,
    }),
  );
  delete globalThis.__componentSlotTextModules;
  await window.happyDOM.close();
}
