import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire, stripTypeScriptTypes } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import { compileFunction } from "node:vm";

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const uiRequire = createRequire(
  fileURLToPath(new URL("../../../npm/ui/package.json", import.meta.url)),
);
const vueEntry = uiRequire.resolve("vue");
const vueRequire = createRequire(vueEntry);
const rendererEntry = uiRequire.resolve("vue/server-renderer");
const rendererRequire = createRequire(rendererEntry);
const entries = {
  vue: vueEntry,
  compilerDom: vueRequire.resolve("@vue/compiler-dom"),
  compilerSfc: uiRequire.resolve("vue/compiler-sfc"),
  serverRenderer: rendererEntry,
  happyDom: uiRequire.resolve("happy-dom"),
};
const versions = {
  vue: vueRequire("vue/package.json").version,
  compilerDom: vueRequire("@vue/compiler-dom/package.json").version,
  compilerSfc: vueRequire("@vue/compiler-sfc/package.json").version,
  serverRenderer: vueRequire("@vue/server-renderer/package.json").version,
};
for (const version of Object.values(versions)) assert.equal(version, "3.5.35");
assert.equal(fs.realpathSync(rendererRequire.resolve("vue")), fs.realpathSync(vueEntry));
const rendererPackage = vueRequire.resolve("@vue/server-renderer");
assert.equal(
  fs.realpathSync(createRequire(rendererPackage).resolve("vue")),
  fs.realpathSync(vueEntry),
);
const happyManifest = JSON.parse(
  fs.readFileSync(new URL("../package.json", pathToFileURL(entries.happyDom))),
);
assert.equal(happyManifest.name, "happy-dom");
assert.equal(happyManifest.version, "20.11.2");
export const providerIdentities = Object.fromEntries(
  Object.entries(entries).map(([name, entry]) => [
    name,
    {
      entry,
      realEntry: fs.realpathSync(entry),
      version: name === "happyDom" ? happyManifest.version : versions[name],
      sha256: sha256(fs.readFileSync(entry)),
    },
  ]),
);

export function validateProviders(contract) {
  assert.deepEqual(contract.versions, versions);
  assert.deepEqual(Object.keys(contract.providers).sort(), Object.keys(entries).sort());
  for (const [name, expected] of Object.entries(contract.providers)) {
    assert.equal(providerIdentities[name].sha256, expected.sha256, `${name}: provider entry`);
  }
}

const { Window } = await import(pathToFileURL(entries.happyDom));
const window = new Window();
for (const name of [
  "window",
  "document",
  "Element",
  "HTMLElement",
  "SVGElement",
  "Node",
  "Event",
  "MouseEvent",
  "navigator",
]) {
  Object.defineProperty(globalThis, name, {
    configurable: true,
    value: name === "window" ? window : window[name],
  });
}
const Vue = vueRequire("vue");
const compiler = vueRequire("@vue/compiler-dom");
const { parse } = uiRequire("vue/compiler-sfc");
const { renderToString } = uiRequire("vue/server-renderer");

// Preserve the independently authored stock observer's complete call contract.
export async function observeSource(source, id, states) {
  assert.deepEqual(states, [false, true]);
  const { descriptor, errors } = parse(source, { filename: `${id}.vue` });
  assert.deepEqual(errors, []);
  assert(descriptor.template, "complete SFC template is required");
  const script = descriptor.scriptSetup?.content ?? "";
  const compilerOptions = { mode: "function", prefixIdentifiers: true };
  const domCode = compiler.compile(descriptor.template.content, compilerOptions).code;
  const render = compileFunction(domCode, ["Vue"])(Vue);
  const strippedScript = script ? stripTypeScriptTypes(script, { mode: "strip" }) : "";
  const observations = [];
  for (const state of states) {
    const calls = [];
    const makeData = () => {
      const data = {
        currentItem: {
          primaryValue: state ? "世界" : "primary",
          secondaryValue: state ? "Ω" : "secondary",
          extra: state ? "extra²" : "extra",
        },
        buildTooltipText: (...args) => {
          calls.push({ call: "tooltip", args });
          return args.join(" | ");
        },
        構築TooltipText: (...args) => {
          calls.push({ call: "unicode-tooltip", args });
          return args.join(" | ");
        },
      };
      if (script) {
        const functions = compileFunction(
          `${strippedScript}\nreturn {guard,openEditor,openNotice};`,
          ["Math"],
        )({ random: () => (state ? 0.75 : 0.25) });
        data.guard = (...args) => {
          calls.push({ call: "guard", arguments: args.length });
          return functions.guard(...args);
        };
        data.openEditor = () => {
          calls.push({ call: "editor" });
          return functions.openEditor();
        };
        data.openNotice = () => {
          calls.push({ call: "notice" });
          return functions.openNotice();
        };
      }
      return data;
    };
    const component = { data: makeData, render };
    const host = document.createElement("div");
    document.body.append(host);
    const app = Vue.createApp(component);
    try {
      app.mount(host);
      await Vue.nextTick();
      const beforeClick = {
        innerHTML: host.innerHTML,
        textContent: host.textContent,
        calls: structuredClone(calls),
      };
      const button = host.querySelector("button");
      if (button) {
        button.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
        await Vue.nextTick();
      }
      const afterClick = {
        innerHTML: host.innerHTML,
        textContent: host.textContent,
        calls: structuredClone(calls),
      };
      app.unmount();
      host.remove();
      calls.length = 0;
      const ssr = await renderToString(Vue.createSSRApp(component));
      observations.push({ state, beforeClick, afterClick, ssr, ssrCalls: structuredClone(calls) });
    } finally {
      if (host.isConnected) {
        app.unmount();
        host.remove();
      }
    }
  }
  return {
    id,
    sourceSha256: sha256(source),
    templateSha256: sha256(descriptor.template.content),
    domCode,
    strippedScript,
    compilerOptions,
    observations,
  };
}
