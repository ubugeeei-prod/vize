import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const fromUi = createRequire(
  process.env.VIZE_TEST_VUE_PACKAGE ?? new URL("../../npm/ui/package.json", import.meta.url),
);
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const core = fromVue("vue");
const renderer = fromVue("@vue/server-renderer");
const capture = process.env.VIZE_JSX_SLOTS_SSR_CAPTURE;
const mandatory = process.env.VIZE_L4_SSR_REQUIRE_NATIVE === "1";
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
  ["compiled-ssr", "const A = () => <B v-slots={getSlots()}/>;"],
  [
    "dynamic-mixed",
    "const A = () => <view.Component v-slots={getSlots()}><div>A</div><template v-slot:named><i>N</i></template></view.Component>;",
  ],
];
const url = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
const hash = (source: string) => createHash("sha256").update(source).digest("hex");

async function load(code: string) {
  const relinked = code.replace(
    /^import \{ ([^\n]+) \} from "(vue|@vue\/server-renderer)"$/gm,
    (_line, imports: string, provider: string) => {
      const names = imports.split(", ").map((item) => {
        const name = item.split(" as ").at(0);
        assert.ok(name);
        assert.match(name, /^[A-Za-z_$][\w$]*$/);
        return name;
      });
      const bridge = url(
        `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve(provider)).href)};\n` +
          names.map((name) => `export const ${name} = runtime.${name};`).join("\n"),
      );
      return `import { ${imports} } from ${JSON.stringify(bridge)}`;
    },
  );
  assert.equal(code.split("function ssrRender(").length, 2);
  const loaded = await import(
    url(
      relinked.replace(
        "function ssrRender(",
        "export function create(slots, getSlots, owner, choose, left, right, view) { return function ssrRender(",
      ) + "\n}",
    )
  );
  return loaded.create;
}

test(
  "nine fresh JSX SSR modules preserve runtime slot ownership and whole HTML",
  {
    skip: !capture && !mandatory,
  },
  async () => {
    assert.ok(capture, "mandatory fresh JSX modules are required");
    assert.equal(fromVue("vue/package.json").version, "3.5.35");
    assert.equal(fromVue("@vue/server-renderer/package.json").version, "3.5.35");
    const packets = JSON.parse(fs.readFileSync(capture, "utf8"));
    assert.equal(packets.length, 9);
    assert.deepEqual(
      packets.map((packet: any) => [packet.id, packet.source]),
      cases,
    );
    const outcomes = [];
    for (const packet of packets) {
      assert.deepEqual(Object.keys(packet).sort(), ["code", "id", "source"]);
      const create = await load(packet.code);
      let evaluations = 0;
      const calls: string[] = [];
      const warnings: string[] = [];
      const Owned = { render: () => core.h("strong", "parent<&>") };
      const Wrong = { render: () => core.h("strong", "WRONG") };
      const Child = {
        components: { Owned: Wrong },
        ssrRender(ctx: any, push: any, parent: any) {
          renderer.ssrRenderSlot(
            ctx.$slots,
            "default",
            { label: '<&>"' },
            () => push("<u>fallback</u>"),
            push,
            parent,
            "data-v-slot",
          );
          renderer.ssrRenderSlot(
            ctx.$slots,
            "named",
            {},
            () => push("<u>named-fallback</u>"),
            push,
            parent,
          );
        },
      };
      const slots = {
        default: (props: any) => {
          calls.push("raw");
          return [core.h(core.resolveComponent("Owned")), core.h("span", props.label)];
        },
      };
      const getSlots = () => {
        evaluations++;
        return packet.id !== "compiled-ssr"
          ? slots
          : {
              default: core.withCtx((props: any, push: any, _parent: any, scope: string) => {
                calls.push("push:" + scope);
                if (push)
                  push("<em" + scope + ">" + renderer.ssrInterpolate(props.label) + "</em>");
                else return [core.h("em", props.label)];
              }),
            };
      };
      const owner = {
        get slots() {
          evaluations++;
          return slots;
        },
      };
      const Parent = {
        components: {
          B: Child,
          Owned,
          Outer: {
            render(this: any) {
              return core.h("section", this.$slots.default());
            },
          },
        },
        ssrRender: create(
          slots,
          getSlots,
          owner,
          false,
          { default: () => [core.h("u", "WRONG")] },
          slots,
          { Component: Child },
        ),
      };
      const app = core.createSSRApp(Parent);
      app.config.warnHandler = (message: string) => warnings.push(message);
      const html = await renderer.renderToString(app);
      const normal =
        "<!--[--><strong data-v-slot>parent&lt;&amp;&gt;</strong><span data-v-slot>&lt;&amp;&gt;&quot;</span><!--]-->";
      const body =
        packet.id === "compiled-ssr"
          ? "<!--[--><em data-v-slot>&lt;&amp;&gt;&quot;</em><!--]-->"
          : normal;
      const named = ["mixed", "dynamic-mixed"].includes(packet.id)
        ? "<!--[--><i>N</i><!--]-->"
        : "<!--[--><u>named-fallback</u><!--]-->";
      assert.equal(
        html,
        packet.id === "nested" ? `<section>${body}${named}</section>` : body + named,
        packet.id,
      );
      assert.equal(
        evaluations,
        ["mixed", "dynamic-mixed", "member", "compiled-ssr"].includes(packet.id) ? 1 : 0,
        packet.id,
      );
      assert.deepEqual(
        calls,
        [packet.id === "compiled-ssr" ? "push: data-v-slot" : "raw"],
        packet.id,
      );
      assert.deepEqual(warnings, [], packet.id);
      outcomes.push({
        ...packet,
        codeSha256: hash(packet.code),
        html,
        evaluations,
        calls,
        warnings,
      });
    }
    fs.writeFileSync(
      capture + ".runtime.json",
      JSON.stringify({ runtime: "vue@3.5.35", outcomes }, null, 2) + "\n",
    );
  },
);
