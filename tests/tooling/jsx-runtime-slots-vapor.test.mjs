import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { evaluateCompiledRender } from "./support/davinci-runtime-trace.mjs";
import {
  officialCompilerVapor,
  vueVaporBrowserRuntime,
  vueVaporVersion,
} from "./support/vue-vapor-release.mjs";

const required = process.env.VIZE_NATIVE_VAPOR_REQUIRE_CAPTURE === "1";
const capture = process.env.VIZE_JSX_VAPOR_SLOTS_CAPTURE;
const cases = [
  ["bare", "const A = () => <B v-slots={slots}/>;"],
  [
    "mixed",
    "const A = () => <B v-slots={getSlots()}><div>A</div><template v-slot:named><i>N</i></template></B>;",
  ],
  ["member", "const A = () => <B v-slots={owner.slots}/>;"],
  ["conditional", "const A = () => <B v-slots={choose ? left : right}/>;"],
  ["nested", "const A = () => <Outer><B v-slots={slots}/></Outer>;"],
  ["dynamic-component", "const A = () => <view.Component v-slots={slots}/>;"],
  ["alias", "const A = () => <B vSlots={slots}/>;"],
];

assert.ok(!required || capture, "mandatory fresh JSX client Vapor modules are required");

test(
  "seven fresh JSX client Vapor modules retain forwarded slots in the pinned runtime",
  { skip: !capture && !required },
  async () => {
    assert.ok(capture);
    const packets = JSON.parse(readFileSync(capture, "utf8"));
    assert.equal(packets.length, cases.length);
    assert.deepEqual(
      packets.map((packet) => [packet.id, packet.source]),
      cases,
    );

    const requireUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
    const { Window } = await import(requireUi.resolve("happy-dom"));
    const window = new Window({ url: "http://localhost/" });
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
    ]) {
      globalThis[key] = key === "window" ? window : window[key];
    }
    const dataUrl = (source) =>
      `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
    const vue = await import(dataUrl(readFileSync(vueVaporBrowserRuntime, "utf8")));
    assert.equal(vue.version, vueVaporVersion);

    const compileRender = async (source) => {
      const output = officialCompilerVapor.compile(source, { mode: "module" });
      return evaluateCompiledRender(output.code, vue);
    };
    const childRender = await compileRender('<main><slot/><slot name="named"/></main>');
    const outerRender = await compileRender("<section><slot/></section>");
    const Child = vue.defineVaporComponent({ render: childRender });
    const Outer = vue.defineVaporComponent({ render: outerRender });
    const defaultFactory = vue.template("<strong>forwarded</strong>");
    const namedFactory = vue.template("<b>forwarded named</b>");
    const wrongFactory = vue.template("<u>wrong</u>");
    const slots = {
      default: () => defaultFactory(),
      named: () => namedFactory(),
    };
    const wrong = { default: () => wrongFactory() };
    const outcomes = [];

    for (const packet of packets) {
      assert.deepEqual(Object.keys(packet).sort(), ["code", "id", "source"]);
      let evaluations = 0;
      const forwardedDefaultOnly = { default: slots.default };
      const context = {
        slots,
        getSlots: () => {
          evaluations++;
          return forwardedDefaultOnly;
        },
        owner: {
          get slots() {
            evaluations++;
            return slots;
          },
        },
        choose: true,
        left: slots,
        right: wrong,
        view: { Component: Child },
      };
      const render = await evaluateCompiledRender(packet.code, vue, context);
      const Parent = vue.defineVaporComponent({ render });
      const app = vue.createVaporApp(Parent);
      app.component("B", Child);
      app.component("Outer", Outer);
      const diagnostics = [];
      app.config.warnHandler = (message) => diagnostics.push(message);
      app.config.errorHandler = (error) => diagnostics.push(String(error));
      const host = window.document.createElement("div");
      window.document.body.append(host);
      try {
        app.mount(host);
        await vue.nextTick();
        const child =
          packet.id === "mixed"
            ? "<main><strong>forwarded</strong><!--slot--><i>N</i><!--slot--></main>"
            : "<main><strong>forwarded</strong><!--slot--><b>forwarded named</b><!--slot--></main>";
        const html =
          packet.id === "nested"
            ? `<section>${child}<!--slot--></section>`
            : packet.id === "dynamic-component"
              ? `${child}<!--dynamic-component-->`
              : child;
        assert.equal(host.innerHTML, html, packet.id);
        assert.equal(evaluations, ["mixed", "member"].includes(packet.id) ? 1 : 0, packet.id);
        assert.deepEqual(diagnostics, [], packet.id);
        outcomes.push({ ...packet, html, evaluations, diagnostics });
      } finally {
        app.unmount();
        host.remove();
      }
    }

    writeFileSync(
      capture + ".runtime.json",
      JSON.stringify({ runtime: `vue@${vueVaporVersion}`, outcomes }, null, 2) + "\n",
    );
    await window.happyDOM.close();
  },
);
