import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { environment7502 } from "./support/native-attribute-values-7502-loader.ts";

interface Capture {
  name: string;
  template: string;
  code: string;
}

const capturePath = process.env.VIZE_RUNTIME_PATCH_FLAGS_CAPTURE;
const required = process.env.VIZE_RUNTIME_PATCH_FLAGS_REQUIRE_CAPTURE === "1";
process.env.NODE_ENV = "development";
const environment = await environment7502("development");
const ui = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const compiler = createRequire(ui.resolve("vue/package.json"))("@vue/compiler-dom");

async function execute(code: string) {
  const { component, runtime } = await environment.load(
    `${code}\nexport default { data: () => ({ label: "first" }), render }\n`,
    "dom",
  );
  const host = environment.window.document.createElement("div");
  const app = runtime.createApp(component);
  try {
    const proxy = app.mount(host) as { label: string };
    const before = host.textContent;
    proxy.label = "second";
    await runtime.nextTick();
    return { before, after: host.textContent };
  } finally {
    app.unmount();
  }
}

async function executeHydratedRoot(code: string) {
  const { component, runtime } = await environment.load(
    `${code}\nexport default { data: () => ({ label: "first" }), render }\n`,
    "dom",
  );
  const host = environment.window.document.createElement("div");
  host.innerHTML = "<!--[-->first <span>!</span><!--]-->";
  const app = runtime.createSSRApp(component);
  try {
    const proxy = app.mount(host) as { label: string };
    const before = host.textContent;
    proxy.label = "second";
    await runtime.nextTick();
    return { before, after: host.textContent };
  } finally {
    app.unmount();
  }
}

async function executeScopedSlot(code: string) {
  const { component, runtime } = await environment.load(
    `${code}\nexport default { data: () => ({ items: [{ id: 1, tone: "red", name: "Alpha" }] }), render }\n`,
    "dom",
  );
  const MyList = {
    props: ["items"],
    setup(props: { items: object[] }, { slots }: { slots: Record<string, Function> }) {
      return () =>
        runtime.h(
          "div",
          props.items.flatMap((item) => slots.default?.({ item }) ?? []),
        );
    },
  };
  const MyChip = {
    props: ["tone"],
    setup(props: { tone: string }, { slots }: { slots: Record<string, Function> }) {
      return () => runtime.h("b", { "data-tone": props.tone }, slots.default?.());
    },
  };
  const host = environment.window.document.createElement("div");
  const app = runtime.createApp(component);
  app.component("MyList", MyList);
  app.component("MyChip", MyChip);
  try {
    const proxy = app.mount(host) as { items: object[] };
    const before = host.textContent;
    proxy.items = [{ id: 2, tone: "red", name: "Beta" }];
    await runtime.nextTick();
    return { before, after: host.textContent };
  } finally {
    app.unmount();
  }
}

test(
  "runtime-sensitive patch flags refresh text and slot closures in Vue",
  {
    skip:
      !capturePath && !required ? "source capture is supplied by the native Actions lane" : false,
  },
  async () => {
    assert(capturePath, "a required source compiler capture must be provided");
    const captures: Capture[] = JSON.parse(readFileSync(capturePath, "utf8"));
    assert.equal(captures.length, 6);
    for (const capture of captures) {
      if (capture.name === "scoped-slot-closure") {
        assert.match(capture.code, /_: 2 \/\* DYNAMIC \*\//, capture.name);
        assert.match(capture.code, /1032 \/\* PROPS, DYNAMIC_SLOTS \*\//, capture.name);
        const expected = { before: "Alpha", after: "Beta" };
        assert.deepEqual(await executeScopedSlot(capture.code), expected, capture.name);
        continue;
      }
      assert.match(capture.code, /, 1 \/\* TEXT \*\/\)/, capture.name);
      const rootText =
        capture.name === "root-fragment-interpolation"
          ? '_createTextVNode(_toDisplayString(_ctx.label) + " ", 1 /* TEXT */)'
          : capture.name === "root-fragment-trailing-interpolation"
            ? '_createTextVNode(" " + _toDisplayString(_ctx.label), 1 /* TEXT */)'
            : capture.name === "root-fragment-comment-boundary"
              ? "_createTextVNode(_toDisplayString(_ctx.label), 1 /* TEXT */)"
              : undefined;
      if (rootText) {
        const upstream = compiler.compile(capture.template, {
          mode: "module",
          prefixIdentifiers: true,
        }).code;
        assert.ok(upstream.includes(rootText), upstream);
        assert.ok(capture.code.includes(rootText), capture.code);
      }
      const textAndInterpolation = capture.name.includes("text-and");
      const expectedRoot =
        capture.name === "root-fragment-interpolation"
          ? ["first !", "second !"]
          : capture.name === "root-fragment-trailing-interpolation"
            ? ["! first", "! second"]
            : capture.name === "root-fragment-comment-boundary"
              ? ["first!", "second!"]
              : undefined;
      const expected = {
        before: textAndInterpolation ? "Hi first" : (expectedRoot?.[0] ?? "first"),
        after: textAndInterpolation ? "Hi second" : (expectedRoot?.[1] ?? "second"),
      };
      assert.deepEqual(await execute(capture.code), expected, capture.name);
      if (capture.name === "root-fragment-interpolation")
        assert.deepEqual(await executeHydratedRoot(capture.code), expected, capture.name);
    }
  },
);
