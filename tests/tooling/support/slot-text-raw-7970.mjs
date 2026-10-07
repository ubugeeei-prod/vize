// Actual existing Vue runtime observes the retained Vue 1 source spelling.
// This is a coercion compatibility control, not a Vue 1 runtime oracle.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { Window } from "happy-dom";

const root = new URL("../../../", import.meta.url);
const fromUi = createRequire(new URL("npm/ui/package.json", root));
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
const vue = fromUi("vue");
const { transformSync } = fromUi("@babel/core");
assert.equal(vue.version, "3.5.35");
assert.equal(process.env.NODE_ENV, "production");
globalThis.__slotTextRawVue = vue;
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const rows = JSON.parse(Buffer.concat(chunks));
const fixtures = JSON.parse(
  readFileSync(
    new URL("tests/_fixtures/differential/compiler/component-slot-text-7970/raw-cases.json", root),
  ),
);
const evidence = {
  sourceRevision: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root }).toString().trim(),
  vue: vue.version,
  nativeHandled: 0,
  observations: [],
  qualified: 0,
};
if (process.env.GITHUB_SHA) assert.equal(evidence.sourceRevision, process.env.GITHUB_SHA);
function nodes(parent) {
  return [...parent.childNodes].map((node) =>
    node.nodeType === 3 || node.nodeType === 8
      ? { type: node.nodeType, data: node.data }
      : {
          type: node.nodeType,
          tag: node.localName,
          attributes: [...node.attributes].map((attr) => [attr.name, attr.value]),
          children: nodes(node),
        },
  );
}
try {
  assert.equal(rows.length, 5);
  for (const [index, row] of rows.entries()) {
    assert.deepEqual(row.case, fixtures[index]);
    assert.deepEqual(row.diagnostics, []);
    assert.equal(typeof row.map, "string");
    const code = transformSync(row.code, {
      configFile: false,
      babelrc: false,
      plugins: [
        ({ types: t }) => ({
          visitor: {
            ImportDeclaration(path) {
              assert.equal(path.node.source.value, "vue");
              path.replaceWith(
                t.variableDeclaration(
                  "const",
                  path.node.specifiers.map((specifier) => {
                    assert(t.isImportSpecifier(specifier));
                    assert(Object.hasOwn(vue, specifier.imported.name));
                    return t.variableDeclarator(
                      t.identifier(specifier.local.name),
                      t.memberExpression(
                        t.memberExpression(
                          t.identifier("globalThis"),
                          t.identifier("__slotTextRawVue"),
                        ),
                        t.identifier(specifier.imported.name),
                      ),
                    );
                  }),
                ),
              );
            },
          },
        }),
      ],
    }).code;
    const { render } = await import(
      `data:text/javascript;base64,${Buffer.from(code + `\n// ${index}`).toString("base64")}`
    );
    const diagnostics = [];
    const app = vue.createApp({ render, data: () => ({ left: 1, right: 2, show: true }) });
    app.component("MyTitle", {
      setup(_, { slots }) {
        return () => vue.h("h2", slots.default?.());
      },
    });
    app.config.warnHandler = (message) => diagnostics.push(message);
    app.config.errorHandler = (error) => diagnostics.push(String(error));
    const host = document.createElement("div");
    document.body.append(host);
    let mounted = false;
    try {
      const instance = app.mount(host);
      mounted = true;
      const snapshots = [];
      for (const expected of [row.case.before, row.case.after]) {
        await vue.nextTick();
        assert.deepEqual(diagnostics, []);
        const title = host.querySelector("h2");
        assert(title);
        assert.deepEqual(
          [...title.childNodes].map((node) =>
            node.nodeType === 3 ? node.data : `<${node.localName}>`,
          ),
          expected,
        );
        assert.equal(
          host.innerHTML,
          `<h2>${expected.map((value) => (value === "<i>" ? "<i></i>" : value)).join("")}</h2>`,
        );
        snapshots.push({ html: host.innerHTML, nodes: nodes(host), diagnostics: [...diagnostics] });
        Object.assign(instance, { left: 3, right: 4, show: false });
      }
      app.unmount();
      mounted = false;
      await vue.nextTick();
      assert.deepEqual(nodes(host), []);
      evidence.observations.push({ ...row, snapshots, afterUnmount: [], diagnostics });
      evidence.qualified++;
    } finally {
      if (mounted) app.unmount();
      host.remove();
    }
  }
  assert.equal(evidence.qualified, 5);
} catch (error) {
  evidence.failure = String(error.stack ?? error);
  process.exitCode = 1;
} finally {
  const profile = process.env.NEXTEST_PROFILE ?? "pr";
  assert(["pr", "full"].includes(profile));
  const directory = new URL(`target/nextest/${profile}/slot-text-raw-7970/`, root);
  mkdirSync(directory, { recursive: true });
  writeFileSync(new URL("observations.json", directory), JSON.stringify(evidence, null, 2) + "\n");
  process.stdout.write(
    JSON.stringify({
      qualified: evidence.qualified,
      nativeHandled: 0,
      failure: evidence.failure ?? null,
    }),
  );
  delete globalThis.__slotTextRawVue;
  await window.happyDOM.close();
}
